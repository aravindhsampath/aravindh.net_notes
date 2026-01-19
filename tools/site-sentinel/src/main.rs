use anyhow::{anyhow, Context, Result};
use chrono::Local;
use log::{error, info, warn};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use serde::Deserialize;
use simplelog::*; // Import all from simplelog
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

// --- Configuration Structs ---
#[derive(Deserialize, Clone, Debug)]
struct Config {
    sentinel: SentinelConfig,
    deploy: DeployConfig,
}

#[derive(Deserialize, Clone, Debug)]
struct SentinelConfig {
    content_dir: String,
    log_file: String,
}

#[derive(Deserialize, Clone, Debug)]
struct DeployConfig {
    ssh_key: String,
    ssh_target: String,
    dest_dir: String,
}

// Defaults / Constants
const DEBOUNCE_TIME: Duration = Duration::from_millis(500);
const RETRY_DELAY: Duration = Duration::from_secs(5);
const MAX_RETRIES: u32 = 2;
const CONFIG_FILE: &str = "site.toml";

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Initial Config
    let config = Arc::new(RwLock::new(load_config().context("Initial config load failed")?));

    // 2. Setup Logging
    // We read log file path from config once. If log file changes in config, we don't switch loggers dynamically in this simplified version.
    let log_path = config.read().unwrap().sentinel.log_file.clone();
    setup_logging(&log_path)?;

    let content_dir = config.read().unwrap().sentinel.content_dir.clone();
    info!("Site Sentinel started. Watching '{}' and '{}'!", content_dir, CONFIG_FILE);

    // 3. Setup Watcher
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(DEBOUNCE_TIME, move |res: DebounceEventResult| {
        let _ = tx.blocking_send(res);
    })?;

    // Watch Content Dir
    debouncer
        .watcher()
        .watch(Path::new(&content_dir), RecursiveMode::Recursive)?;
    
    // Watch Config File
    debouncer
        .watcher()
        .watch(Path::new(CONFIG_FILE), RecursiveMode::NonRecursive)?;

    // 4. Event Loop
    while let Some(res) = rx.recv().await {
        match res {
            Ok(events) => {
                let mut needs_build = false;
                let mut config_reloaded = false;

                for event in events {
                    let path = event.path;
                    
                    // Handle Config Change
                    if path.ends_with(CONFIG_FILE) {
                        info!("Config file changed. Reloading...");
                        match load_config() {
                            Ok(new_config) => {
                                let mut w = config.write().unwrap();
                                *w = new_config;
                                info!("Config reloaded successfully.");
                                config_reloaded = true;
                            }
                            Err(e) => {
                                error!("Failed to reload config: {}", e);
                                notify("Config Reload Failed", &format!("{}", e));
                            }
                        }
                    } 
                    // Handle Content Change
                    // We check if it's inside content_dir (simple check: if it's not config file)
                    // and matches extension.
                    else if path.extension().map_or(false, |ext| ext == "md") {
                        if is_new_empty_file(&path) {
                            info!("Detected new empty file: {:?}", path);
                            if let Err(e) = inject_front_matter(&path) {
                                error!("Failed to inject front matter for {:?}: {}", path, e);
                                notify("Sentinel Error", &format!("Front Matter failed: {}", e));
                            }
                        } else {
                            needs_build = true;
                        }
                    }
                }

                // If config reloaded, we might want to update watcher if content_dir changed? 
                // For simplicity, we assume content_dir doesn't change often. 
                // If it does, a restart is safer. We'll just Log warning if it changed.
                if config_reloaded {
                   // Optional: Check if content dir changed and warn user to restart
                }

                if needs_build {
                    // Pass a snapshot of the current config
                    let current_config = config.read().unwrap().clone();
                    handle_build_and_deploy(current_config).await;
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn load_config() -> Result<Config> {
    let content = fs::read_to_string(CONFIG_FILE)
        .context(format!("Could not read {}", CONFIG_FILE))?;
    let config: Config = toml::from_str(&content)
        .context(format!("Failed to parse {}", CONFIG_FILE))?;
    Ok(config)
}

fn setup_logging(path: &str) -> Result<()> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("Failed to open log file")?;

    WriteLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        log_file,
    )
    .context("Failed to init logger")?;
    Ok(())
}

fn is_new_empty_file(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        return metadata.len() == 0;
    }
    false
}

fn inject_front_matter(path: &Path) -> Result<()> {
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");
    
    // Convert kebab-case to Title Case (simple approx)
    let title = filename.replace("-", " ");
    let date = Local::now().format("%Y-%m-%dT%H:%M:%S%:z");

    let content = format!(
        r###"+++\ntitle = "{}"\ndate = {}\ndraft = false\ntags = []\n+++\n\n"###,
        title, date
    );

    fs::write(path, content)?;
    info!("Injected front matter into {:?}", path);
    Ok(())
}

async fn handle_build_and_deploy(config: Config) {
    info!("Change detected. Starting pipeline...");

    // Step 1: Hugo Build
    match run_command("hugo", &["--minify"])
        .await {
        Ok(_) => {
            info!("Hugo build successful.");
        }
        Err(e) => {
            error!("Hugo build failed: {}", e);
            notify("Hugo Build Failed", "Check log for details.");
            return; // Stop pipeline
        }
    }

    // Step 2: Concurrent Deploy (Git + Rsync)
    // We clone config for the closures
    let git_task = tokio::spawn(async move {
        retry_op("Git Push", || async {
            // Check for changes first
            let status_output = run_command_output("git", &["status", "--porcelain"])
                .await?;
            if status_output.trim().is_empty() {
                info!("Git: Nothing to commit.");
                // Ensure we push if we are ahead (omitted for simplicity, we just return Ok)
                return Ok(())
            }

            run_command("git", &["add", "."])
                .await?;
            run_command("git", &["commit", "-m", "Auto-save by Site Sentinel"])
                .await?;
            run_command("git", &["push"])
                .await
        })
        .await
    });

    let deploy_config = config.clone();
    let deploy_task = tokio::spawn(async move {
        retry_op("Rsync Deploy", || async {
            let expanded_key = shellexpand::tilde(&deploy_config.deploy.ssh_key).to_string();
            let rsh_cmd = format!("ssh -i {}", expanded_key);
            let dest = format!("{}:{}", deploy_config.deploy.ssh_target, deploy_config.deploy.dest_dir);

            run_command("rsync", &[
                "-az", 
                "--delete", 
                "-e", &rsh_cmd, 
                "public/", 
                &dest
            ])
            .await
        })
        .await
    });

    let (git_res, deploy_res) = tokio::join!(git_task, deploy_task);

    let mut failed = false;

    // Handle Join Errors and Task Results
    match git_res {
        Ok(task_result) => {
            if let Err(e) = task_result {
                error!("Git task failed: {}", e);
                notify("Git Sync Failed", "Check log.");
                failed = true;
            }
        }
        Err(e) => { error!("Git task panicked: {}", e); failed = true; }
    }

    match deploy_res {
        Ok(task_result) => {
            if let Err(e) = task_result {
                error!("Deploy task failed: {}", e);
                notify("Deploy Failed", "Check log.");
                failed = true;
            }
        }
        Err(e) => { error!("Deploy task panicked: {}", e); failed = true; }
    }

    if !failed {
        info!("Pipeline completed successfully.");
    }
}

// Wrapper that returns output as string for checking
async fn run_command_output(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("Failed to execute {}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("{} failed: {}", cmd, stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Wrapper for commands where we only care about success
async fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    run_command_output(cmd, args).await.map(|_| ())
}

async fn retry_op<F, Fut, T>(name: &str, op: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempts = 0;
    loop {
        match op().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                attempts += 1;
                warn!("{} failed (attempt {}/{}): {}", name, attempts, MAX_RETRIES + 1, e);
                if attempts > MAX_RETRIES {
                    return Err(e);
                }
                sleep(RETRY_DELAY).await;
            }
        }
    }
}

fn notify(title: &str, message: &str) {
    let safe_title = escape_for_applescript(title);
    let safe_message = escape_for_applescript(message);
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!("display notification \"{}\" with title \"{}\"", safe_message, safe_title))
        .output();
}

fn escape_for_applescript(s: &str) -> String {
    s.replace("\\", "\\\\").replace("\"", "\\\"")
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_delivery_hugo_fail() {
        println!("Sending test notification for Hugo failure...");
        notify("TEST: Hugo Build Failed", "This is a test notification.");
    }

    #[test]
    fn test_notification_delivery_git_fail() {
        println!("Sending test notification for Git failure...");
        notify("TEST: Git Sync Failed", "This is a test notification.");
    }

    #[test]
    fn test_notification_delivery_deploy_fail() {
        println!("Sending test notification for Deploy failure...");
        notify("TEST: Deploy Failed", "This is a test notification.");
    }

    #[test]
    fn test_notification_with_special_chars() {
        println!("Sending test notification with special chars...");
        notify("TEST: Special Chars", "Quotes \" and Backslashes \\ should work.");
    }

    #[tokio::test]
    async fn test_retry_logic_failure_triggers_error() {
        let result: Result<()> = retry_op("Test Fail", || async {
            Err(anyhow!("Forced failure"))
        }).await;
        
        assert!(result.is_err());
    }
}
