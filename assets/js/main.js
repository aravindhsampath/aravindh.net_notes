const THEME_KEY = "theme";

function getSystemTheme() {
  return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function getStoredTheme() {
  const theme = window.localStorage.getItem(THEME_KEY);
  if (theme === "light" || theme === "dark") return theme;
  return null;
}

function applyTheme(theme) {
  const root = document.documentElement;
  if (theme === "light" || theme === "dark") {
    root.setAttribute("data-theme", theme);
  } else {
    root.removeAttribute("data-theme");
  }
}

function setTheme(theme) {
  if (theme === "light" || theme === "dark") {
    window.localStorage.setItem(THEME_KEY, theme);
    applyTheme(theme);
  } else {
    window.localStorage.removeItem(THEME_KEY);
    applyTheme(null);
  }
}

function getEffectiveTheme() {
  return getStoredTheme() ?? getSystemTheme();
}

function initThemeToggle() {
  applyTheme(getStoredTheme());

  const toggle = document.getElementById("theme-toggle");
  if (!toggle) return;

  toggle.addEventListener("click", () => {
    const next = getEffectiveTheme() === "dark" ? "light" : "dark";
    setTheme(next);
  });

  if (window.matchMedia) {
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (!getStoredTheme()) applyTheme(null);
    });
  }
}

function debounce(fn, ms) {
  let timeoutId;
  return (...args) => {
    window.clearTimeout(timeoutId);
    timeoutId = window.setTimeout(() => fn(...args), ms);
  };
}

function escapeHtml(input) {
  return input
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function stripHtml(input) {
  const el = document.createElement("div");
  el.innerHTML = input;
  return (el.textContent || el.innerText || "").trim();
}

function truncate(input, maxChars) {
  const normalized = input.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxChars) return normalized;
  return normalized.slice(0, maxChars - 1).trimEnd() + "…";
}

function initSearch() {
  const input = document.getElementById("search-input");
  const results = document.getElementById("search-results");
  if (!input || !results) return;

  let pagefindLoading = null;
  function getPagefindScriptUrl() {
    const meta = document.querySelector('meta[name="pagefind-script"]');
    const path = meta?.getAttribute("content") || "pagefind/pagefind.js";
    return new URL(path, window.location.origin).toString();
  }

  async function loadPagefind() {
    if (window.pagefind && typeof window.pagefind.search === "function") return window.pagefind;
    if (pagefindLoading) return pagefindLoading;

    const url = getPagefindScriptUrl();
    pagefindLoading = import(url)
      .then((mod) => {
        window.pagefind = mod;
        return mod;
      })
      .catch((err) => {
        pagefindLoading = null;
        throw err;
      });
    return pagefindLoading;
  }

  function closeResults() {
    results.hidden = true;
    results.innerHTML = "";
  }

  function openResults() {
    results.hidden = false;
  }

  function renderEmpty(message) {
    results.innerHTML = `<div class="empty">${escapeHtml(message)}</div>`;
    openResults();
  }

  async function doSearch(term) {
    const q = term.trim();
    if (!q) return closeResults();

    let pagefind;
    try {
      pagefind = await loadPagefind();
    } catch {
      return renderEmpty("Search index isn’t available yet. Run Pagefind once after building.");
    }

    const search = await pagefind.search(q);
    if (!search.results || search.results.length === 0) {
      return renderEmpty("No results.");
    }

    const top = search.results.slice(0, 8);
    const data = await Promise.all(top.map((r) => r.data()));

    results.innerHTML = data
      .map((item) => {
        const title = escapeHtml(item.meta?.title || item.title || "Untitled");
        const url = escapeHtml(item.url);
        const excerpt = truncate(stripHtml(item.excerpt || ""), 160);
        return `<a href="${url}"><div class="result-title">${title}</div><p class="result-excerpt">${escapeHtml(excerpt)}</p></a>`;
      })
      .join("");
    openResults();
  }

  const debounced = debounce(() => doSearch(input.value), 160);

  input.addEventListener("input", debounced);
  input.addEventListener("focus", () => {
    if (results.innerHTML.trim()) openResults();
  });
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      input.blur();
      closeResults();
    }
  });

  document.addEventListener("click", (e) => {
    if (!results.hidden && !results.contains(e.target) && e.target !== input) closeResults();
  });
}

document.addEventListener("DOMContentLoaded", () => {
  initThemeToggle();
  initSearch();
});
