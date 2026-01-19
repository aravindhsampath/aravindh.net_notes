const THEME_KEY = "theme";

function getMetaScriptUrl(metaName, fallbackPath) {
  const meta = document.querySelector(`meta[name="${metaName}"]`);
  const path = meta?.getAttribute("content") || fallbackPath;
  return new URL(path, window.location.origin).toString();
}

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

function escapeRegExp(input) {
  return input.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
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

function parseQueryTerms(query) {
  return query
    .trim()
    .split(/\s+/)
    .map((t) => t.trim())
    .filter(Boolean);
}

function highlightTextToHtml(text, terms) {
  if (!text) return "";
  if (!terms || terms.length === 0) return escapeHtml(text);

  const unique = Array.from(new Set(terms))
    .filter(Boolean)
    .sort((a, b) => b.length - a.length);
  if (unique.length === 0) return escapeHtml(text);

  const re = new RegExp(`(${unique.map(escapeRegExp).join("|")})`, "gi");
  const parts = text.split(re);
  return parts
    .map((part, index) => {
      // Split by capturing group: odd indices are the matches
      if (index % 2 === 1) {
        return `<mark class="search-hit">${escapeHtml(part)}</mark>`;
      }
      return escapeHtml(part);
    })
    .join("");
}

function initSearch() {
  const input = document.getElementById("search-input");
  const results = document.getElementById("search-results");
  if (!input || !results) return;

  let pagefindLoading = null;
  let pagefindConfigured = false;
  function getPagefindScriptUrl() {
    return getMetaScriptUrl("pagefind-script", "pagefind/pagefind.js");
  }

  async function loadPagefind() {
    if (window.pagefind && typeof window.pagefind.search === "function") return window.pagefind;
    if (pagefindLoading) return pagefindLoading;

    const url = getPagefindScriptUrl();
    pagefindLoading = import(url)
      .then(async (mod) => {
        window.pagefind = mod;
        if (!pagefindConfigured && typeof mod.options === "function") {
          pagefindConfigured = true;
          await mod.options({ highlightParam: "pagefind-highlight" });
        }
        return mod;
      })
      .catch((err) => {
        pagefindLoading = null;
        throw err;
      });
    return pagefindLoading;
  }

  function addHighlightToUrl(urlString, query) {
    const url = new URL(urlString, window.location.origin);
    const terms = parseQueryTerms(query);

    for (const term of terms) {
      url.searchParams.append("pagefind-highlight", term);
    }
    return url.toString();
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

    if (typeof pagefind.preload === "function") {
      try {
        await pagefind.preload();
      } catch {
        // ignore
      }
    }

    const search = await pagefind.search(q);
    if (!search.results || search.results.length === 0) {
      return renderEmpty("No results.");
    }

    const top = search.results.slice(0, 8);
    const data = await Promise.all(top.map((r) => r.data()));

    const terms = parseQueryTerms(q);
    results.innerHTML = data
      .map((item) => {
        const rawTitle = item.meta?.title || item.title || "Untitled";
        const url = escapeHtml(addHighlightToUrl(item.url, q));
        // Use Pagefind's native excerpt (which has <mark> tags) if available.
        // Fallback to truncating content if excerpt is missing.
        const excerptHtml = item.excerpt
          ? item.excerpt
          : highlightTextToHtml(truncate(stripHtml(item.content || ""), 160), terms);
        const titleHtml = highlightTextToHtml(rawTitle, terms);
        
        return `<a href="${url}"><div class="result-title">${titleHtml}</div><p class="result-excerpt">${excerptHtml}</p></a>`;
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

function initMobileMenu() {
  const hamburger = document.querySelector(".hamburger");
  const menu = document.querySelector(".header-actions");

  if (!hamburger || !menu) return;

  hamburger.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = menu.classList.toggle("is-open");
    hamburger.setAttribute("aria-expanded", isOpen);
  });

  document.addEventListener("click", (e) => {
    if (menu.classList.contains("is-open") && !menu.contains(e.target) && !hamburger.contains(e.target)) {
      menu.classList.remove("is-open");
      hamburger.setAttribute("aria-expanded", "false");
    }
  });
}

function initScrollToTop() {
  const btn = document.getElementById("scroll-to-top");
  if (!btn) return;

  const toggleVisibility = () => {
    if (window.scrollY > 300) {
      btn.classList.add("is-visible");
    } else {
      btn.classList.remove("is-visible");
    }
  };

  window.addEventListener("scroll", debounce(toggleVisibility, 100));

  btn.addEventListener("click", () => {
    window.scrollTo({ top: 0, behavior: "smooth" });
  });
}

function initMediumZoom() {
  if (typeof mediumZoom === 'undefined') return;

  const getOverlayColor = () => {
    const isDark = document.documentElement.getAttribute('data-theme') === 'dark' || 
                   (!document.documentElement.hasAttribute('data-theme') && 
                    window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches);
    // Matches CSS var(--bg) approx
    return isDark ? 'rgba(10, 10, 10, 0.95)' : 'rgba(253, 251, 247, 0.95)'; 
  };

  const zoom = mediumZoom('.prose img', {
    margin: 24,
    background: getOverlayColor(),
    scrollOffset: 0,
  });

  // Update background on theme toggle
  const observer = new MutationObserver(() => {
    zoom.update({ background: getOverlayColor() });
  });
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  
  // Also listen for system preference changes
  if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      zoom.update({ background: getOverlayColor() });
    });
  }
}

document.addEventListener("DOMContentLoaded", () => {
  initThemeToggle();
  initSearch();
  initMobileMenu();
  initScrollToTop();
  initMediumZoom();

  const params = new URLSearchParams(window.location.search);
  if (params.getAll("pagefind-highlight").length > 0) {
    import(getMetaScriptUrl("pagefind-highlight-script", "pagefind/pagefind-highlight.js"))
      .then((mod) => {
        const Highlighter = mod?.default || window.PagefindHighlight;
        if (typeof Highlighter === "function") {
          new Highlighter({ addStyles: false });
        }
      })
      .catch(() => {});
  }
});
