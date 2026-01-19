use anyhow::{anyhow, Context, Result};
use chrono::Local;
use log::{error, info, warn};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use simplelog::*;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

// Configuration
const CONTENT_DIR: &str = "content";
const LOG_FILE: &str = "site-sentinel.log";
const DEBOUNCE_TIME: Duration = Duration::from_millis(500);
const RETRY_DELAY: Duration = Duration::from_secs(5);
const MAX_RETRIES: u32 = 2;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup Logging
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .context("Failed to open log file")?;

    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )
    .context("Failed to init logger")?;

    info!("Site Sentinel started. Watching '{}'...", CONTENT_DIR);

    // 2. Setup Watcher
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(DEBOUNCE_TIME, move |res: DebounceEventResult| {
        // Bridge blocking callback to async channel
        let _ = tx.blocking_send(res);
    })?;

    debouncer
        .watcher()
        .watch(Path::new(CONTENT_DIR), RecursiveMode::Recursive)?;

    // 3. Event Loop
    while let Some(res) = rx.recv().await {
        match res {
            Ok(events) => {
                let mut needs_build = false;

                for event in events {
                    let path = event.path;
                    // Check if it's a markdown file
                    if path.extension().map_or(false, |ext| ext == "md") {
                        if is_new_empty_file(&path) {
                            info!("Detected new empty file: {:?}", path);
                            if let Err(e) = inject_front_matter(&path) {
                                error!("Failed to inject front matter for {:?}: {}", path, e);
                                notify("Sentinel Error", &format!("Front Matter failed: {}", e));
                            }
                        } else {
                            // It's a modification or non-empty file
                            needs_build = true;
                        }
                    }
                }

                if needs_build {
                    handle_build_and_deploy().await;
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }

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
        "+++\n\
         title = \"{}\"\n\
         date = {}\n\
         draft = false\n\
         tags = []\n\
         +++\n\n",
        title, date
    );

    fs::write(path, content)?;
    info!("Injected front matter into {:?}", path);
    Ok(())
}

async fn handle_build_and_deploy() {
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
    let git_task = tokio::spawn(async {
        retry_op("Git Push", || async {
            run_command("git", &["add", "."])
                .await?;
            // Simple approach: try commit, ignore error if it says 'nothing to commit' 
            let _ = run_command("git", &["commit", "-m", "Auto-save by Site Sentinel"])
                .await;
            run_command("git", &["push"])
                .await
        })
        .await
    });

    let deploy_task = tokio::spawn(async {
        retry_op("Rsync Deploy", || async {
            let ssh_key = format!("{}/.ssh/id_rsa_aravindh.net", std::env::var("HOME").unwrap_or_default());
            let rsh_cmd = format!("ssh -i {}", ssh_key);
            
            run_command("rsync", &[
                "-az", 
                "--delete", 
                "-e", &rsh_cmd, 
                "public/", 
                "root@49.12.190.41:/home/caddy/www/notes/"
            ])
            .await
        })
        .await
    });

    let (git_res, deploy_res) = tokio::join!(git_task, deploy_task);

    let mut failed = false;

    if let Ok(res) = git_res {
        if let Err(e) = res {
            error!("Git task failed: {}", e);
            notify("Git Sync Failed", "Check log.");
            failed = true;
        }
    } else {
         error!("Git task panicked");
         failed = true;
    }

    if let Ok(res) = deploy_res {
        if let Err(e) = res {
            error!("Deploy task failed: {}", e);
            notify("Deploy Failed", "Check log.");
            failed = true;
        }
    } else {
        error!("Deploy task panicked");
        failed = true;
    }

    if !failed {
        info!("Pipeline completed successfully.");
    }
}

async fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
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
    Ok(())
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
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!("display notification \"{}\" with title \"{}\"", message, title))
        .output();
}
