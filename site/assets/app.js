import { COMPONENTS, GROUPS, GUIDES, CHANGELOG, REPO, esc } from "./content.js";

const $ = (s, el = document) => el.querySelector(s);
const view = $("#view");

// ------------------------------------------------------------- theme
const root = document.documentElement;
const stored = (() => { try { return localStorage.getItem("lumen-theme"); } catch { return null; } })();
const sysDark = matchMedia("(prefers-color-scheme: dark)");
function setTheme(t, save) {
  // Swap without animating every color.
  root.classList.add("no-anim");
  root.dataset.theme = t;
  requestAnimationFrame(() => root.classList.remove("no-anim"));
  if (save) { try { localStorage.setItem("lumen-theme", t); } catch {} }
  $("#theme").setAttribute("aria-label", t === "dark" ? "Switch to light theme" : "Switch to dark theme");
  $("#theme").innerHTML = t === "dark" ? ICON.sun : ICON.moon;
  document.querySelectorAll("img[data-light]").forEach((i) => (i.src = t === "dark" ? i.dataset.dark : i.dataset.light));
  document.querySelectorAll("iframe[data-demo]").forEach(syncDemo);
}
const isDark = () => root.dataset.theme === "dark";

// -------------------------------------------------------------- icons
const svg = (d) => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${d}</svg>`;
const ICON = {
  moon: svg('<path d="M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a8.5 8.5 0 1 0 11 11Z"/>'),
  sun: svg('<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>'),
  github: '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 .5a11.5 11.5 0 0 0-3.64 22.41c.58.1.79-.25.79-.56v-2c-3.2.7-3.88-1.37-3.88-1.37-.52-1.33-1.28-1.69-1.28-1.69-1.05-.71.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.68 0-1.26.45-2.28 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.17 1.18a11 11 0 0 1 5.77 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.83 1.19 3.09 0 4.41-2.69 5.38-5.25 5.67.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .5Z"/></svg>',
  menu: svg('<path d="M4 7h16M4 12h16M4 17h16"/>'),
};

// ---------------------------------------------------------- highlight
const KW = new Set("as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while".split(" "));
function highlight(src) {
  const re = /(\/\/[^\n]*)|("(?:[^"\\]|\\.)*")|('(?:[^'\\]|\\.)')|(\b\d[\d_.]*(?:f32|f64|u8|u32|i64)?\b)|([A-Za-z_][A-Za-z0-9_]*!?)|([\s\S])/g;
  let out = "", m;
  while ((m = re.exec(src))) {
    const [t, c, s, ch, n, id] = m;
    if (c) out += `<span class="t-c">${esc(c)}</span>`;
    else if (s || ch) out += `<span class="t-s">${esc(s || ch)}</span>`;
    else if (n) out += `<span class="t-n">${n}</span>`;
    else if (id) {
      if (id.endsWith("!")) out += `<span class="t-m">${id}</span>`;
      else if (KW.has(id)) out += `<span class="t-k">${id}</span>`;
      else if (/^[A-Z]/.test(id)) out += `<span class="t-t">${id}</span>`;
      else if (src[re.lastIndex] === "(") out += `<span class="t-f">${id}</span>`;
      else out += id;
    } else out += esc(t);
  }
  return out;
}
function decorate(el) {
  el.querySelectorAll("pre > code").forEach((c) => {
    if (c.dataset.done) return;
    c.dataset.done = 1;
    if (c.classList.contains("lang-rust") || c.classList.contains("lang-toml") === false) c.innerHTML = highlight(c.textContent);
    const b = document.createElement("button");
    b.className = "copy"; b.type = "button"; b.textContent = "Copy";
    b.addEventListener("click", async () => {
      try { await navigator.clipboard.writeText(c.textContent); b.textContent = "Copied"; } catch { b.textContent = "Press ⌘C"; }
      setTimeout(() => (b.textContent = "Copy"), 1400);
    });
    c.parentElement.append(b);
  });
}
const pre = (s) => `<pre><code class="lang-rust">${esc(s)}</code></pre>`;

// -------------------------------------------------------------- demos
const HEIGHTS = { hero: 540, vault: 600, looks: 380, calendar: 400, "date-picker": 440, select: 340, menu: 360, popover: 340, dialog: 380, toast: 380, "chart-line": 360, "chart-area": 360, "chart-bar": 360, donut: 300, gauge: 240, sparkline: 220, card: 300, "text-input": 340 };
function demoSrc(name, o = {}) {
  const p = new URLSearchParams({ demo: name, dark: isDark() ? "1" : "0", embed: o.embed ? "1" : "0" });
  if (o.look && o.look !== "lumen") p.set("look", o.look);
  if (o.accent) p.set("accent", o.accent);
  return `demo/index.html?${p}`;
}
function syncDemo(f) {
  const src = demoSrc(f.dataset.demo, { look: f.dataset.look, accent: f.dataset.accent, embed: f.dataset.embed === "1" });
  if (!f.getAttribute("src") || !f.getAttribute("src").endsWith(src.split("?")[1])) {
    f.closest(".mock")?.classList.remove("ready");
    f.src = src;
  }
}
// The demo page posts "lumen-ready" after its first frame; then the poster fades.
addEventListener("message", (e) => {
  if (e.data === "lumen-ready") {
    document.querySelectorAll("iframe[data-demo]").forEach((f) => { if (f.contentWindow === e.source) f.closest(".mock")?.classList.add("ready"); });
  }
});
function frame(name, { embed = false, look, height, title, poster } = {}) {
  const h = height || HEIGHTS[name] || 320;
  const img = poster ? `<img class="poster" alt="" data-light="img/${poster}-light.png" data-dark="img/${poster}-dark.png" src="img/${poster}-${isDark() ? "dark" : "light"}.png">` : "";
  return `<div class="mock">${img}<iframe data-demo="${name}" data-embed="${embed ? 1 : 0}" ${look ? `data-look="${look}"` : ""} title="${esc(title || name)} — live lumen demo" loading="lazy" style="height:${h}px"></iframe></div>`;
}
function activateFrames(el) { el.querySelectorAll("iframe[data-demo]").forEach(syncDemo); }

// --------------------------------------------------------------- pages
function home() {
  return `
<div class="page">
  <section class="hero wrap">
    <h1>Rust interfaces that look <span class="lit">finished</span>.</h1>
    <p class="lead">lumen is an immediate-mode UI kit that draws its own components: real type, spring motion, charts, date pickers and overlays. It can look like itself, or like macOS, Windows or GNOME.</p>
    <div class="cta">
      <a class="btn primary" href="#/guide/getting-started">Get started</a>
      <a class="btn ghost" href="#/components">Browse components <span class="arrow" aria-hidden="true">→</span></a>
    </div>
    <p class="install">lumen = { git = "${REPO}" }</p>
    <div class="render">
      ${frame("hero", { title: "Dashboard", poster: "hero" })}
      <p class="caption">This dashboard is lumen running in your browser: Rust compiled to WebAssembly, drawing into one canvas. Hover the chart.</p>
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">The model</p>
    <h2>Your state is a struct. The interface is a function of it.</h2>
    <div class="split">
      <div>
        ${pre(`#[derive(Default)]
struct State { wifi: bool, volume: f32 }

lumen::App::new("Settings").run(State::default(), |ui, s| {
    ui.column().pad(24).gap(14).show(|ui| {
        ui.switch("Wi-Fi", &mut s.wifi);
        ui.slider("Volume", &mut s.volume, 0.0..=1.0);
        if ui.button("Save").primary().clicked() {
            save(s);
        }
    });
});`)}
        <ul class="points">
          <li><b>Widgets are builders.</b> Every option has a sensible default, and the widget is added when the builder drops.</li>
          <li><b>No animation code.</b> Toggles, presses, focus rings and values are springs, so they keep their velocity when interrupted.</li>
          <li><b>Nothing runs at rest.</b> An idle window requests no frames.</li>
        </ul>
      </div>
      <div class="panel peach">${frame("card", { embed: true, title: "Settings card" })}</div>
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">Looks</p>
    <h2>Write it once. Make it feel at home anywhere.</h2>
    <p class="sub">Four looks ship with lumen. Switching blends colors, corner radii, control sizes and motion together. Native builds use the system font and accent color.</p>
    <div class="panel midnight" style="margin-top:56px">
      ${frame("looks", { embed: true, title: "Looks" })}
      <p class="note">Use the switcher inside the demo. Every look comes from the same Rust code.</p>
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">Charts</p>
    <h2>Data that moves when it changes.</h2>
    <div class="split flip">
      <div class="panel marigold">${frame("chart-line", { embed: true, title: "Line chart" })}</div>
      <div>
        <ul class="points">
          <li><b>Line, area, bar, stacked, donut, gauge, sparkline.</b> Axes pick round numbers and labels thin out to fit.</li>
          <li><b>Values spring.</b> New data animates from where the old data was. Smoothing never overshoots the points.</li>
          <li><b>Read by hovering.</b> A crosshair glides between points and a tooltip lists every series.</li>
        </ul>
        <a class="more" href="#/components/line-chart">Read the chart docs <span class="arrow" aria-hidden="true">→</span></a>
      </div>
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">Overlays and dates</p>
    <h2>The parts that are hard to get right.</h2>
    <div class="cards">
      <a class="card" href="#/components/date-picker"><img alt="" data-light="img/thumb-date-light.png" data-dark="img/thumb-date-dark.png" src="img/thumb-date-${isDark() ? "dark" : "light"}.png"><h3>Date and range pickers</h3><p>Months slide as you page. Arrow keys walk the grid. Ranges join across weeks.</p><span class="go">Date picker →</span></a>
      <a class="card" href="#/components/menu"><img alt="" data-light="img/thumb-menu-light.png" data-dark="img/thumb-menu-dark.png" src="img/thumb-menu-${isDark() ? "dark" : "light"}.png"><h3>Menus, selects, popovers</h3><p>Anchored layers that flip to fit, close on Escape or an outside click, and return focus.</p><span class="go">Menu →</span></a>
      <a class="card" href="#/components/dialog"><img alt="" data-light="img/thumb-dialog-light.png" data-dark="img/thumb-dialog-dark.png" src="img/thumb-dialog-${isDark() ? "dark" : "light"}.png"><h3>Dialogs and toasts</h3><p>A modal that rises on a spring over a scrim, and notices that stack and leave on their own.</p><span class="go">Dialog →</span></a>
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">Components</p>
    <h2>${COMPONENTS.length} components, each with a live page.</h2>
    <div class="index">
      ${GROUPS.map((g) => {
        const items = COMPONENTS.filter((c) => c.group === g);
        return items.length ? `<div><h3>${g}</h3>${items.map((c) => `<a href="#/components/${c.slug}">${c.name}</a>`).join("")}</div>` : "";
      }).join("")}
    </div>
  </section>

  <section class="sec wrap">
    <p class="kicker">Where it fits</p>
    <h2>What lumen is, and what it isn't.</h2>
    <div class="compare"><table>
      <thead><tr><th></th><th>How it draws</th><th>Where lumen differs</th></tr></thead>
      <tbody>
        <tr><td>egui</td><td>Immediate mode, own widgets</td><td>Same model. lumen spends its effort on finish: OKLab theming, springs, real type, platform looks.</td></tr>
        <tr><td>GPUI</td><td>Retained elements, own renderer</td><td>Zed's framework, desktop only. lumen is immediate mode and runs in the browser too. The two don't combine.</td></tr>
        <tr><td>iced, Slint</td><td>Retained or declarative, own widgets</td><td>Elm-style messages or a markup language. lumen is plain Rust functions over your state.</td></tr>
        <tr><td>Tauri, Electron</td><td>A web view</td><td>Those ship HTML and a browser engine. lumen is one native binary drawing with the GPU.</td></tr>
        <tr><td>AppKit, WinUI, GTK</td><td>The OS's own widgets</td><td>lumen doesn't wrap them. Its platform looks follow their conventions, but they're lumen widgets.</td></tr>
      </tbody></table></div>
  </section>

  <section class="sec wrap">
    <div class="panel midnight">
      <div class="vault">
        <div>
          <span class="tag">In progress</span>
          <h2>Vault, a password manager built with lumen.</h2>
          <p class="sub">The first full app on lumen: a local-first vault with one-time codes, password health and native looks on every platform. It's being designed now.</p>
          <p style="margin-top:28px"><a class="btn" style="background:#fff;color:#02093a" href="#/apps">About Vault <span class="arrow" aria-hidden="true">→</span></a></p>
        </div>
        <div class="mock"><img alt="Vault preview: an item list, a password field with copy and reveal buttons, a one-time code and a password health gauge." data-light="img/vault-light.png" data-dark="img/vault-dark.png" src="img/vault-${isDark() ? "dark" : "light"}.png"></div>
      </div>
    </div>
  </section>
</div>`;
}

const guideNav = () => `<h4>Guide</h4>${GUIDES.map((g) => `<a href="#/guide/${g.slug}">${g.title}</a>`).join("")}`;
const compNav = () => GROUPS.map((g) => {
  const it = COMPONENTS.filter((c) => c.group === g);
  return it.length ? `<h4>${g}</h4>${it.map((c) => `<a href="#/components/${c.slug}">${c.name}</a>`).join("")}` : "";
}).join("");
const docsShell = (side, body) => `<div class="docs page"><aside class="side" id="side" aria-label="Section navigation">${side}</aside><article class="doc">${body}</article></div>`;

function guide(slug) {
  const i = GUIDES.findIndex((g) => g.slug === slug);
  const g = GUIDES[i] || GUIDES[0];
  const prev = GUIDES[i - 1], next = GUIDES[i + 1];
  return docsShell(guideNav() + `<h4>Reference</h4><a href="#/components">Components</a><a href="#/changelog">Changelog</a>`, `
    <p class="crumb"><a href="#/guide/getting-started">Guide</a></p>
    <h1>${g.title}</h1><p class="lead">${g.lead}</p>
    <div class="prose">${g.body}</div>
    <nav class="next" aria-label="Pagination">${prev ? `<a href="#/guide/${prev.slug}"><small>Previous</small>${prev.title}</a>` : "<span></span>"}${next ? `<a href="#/guide/${next.slug}"><small>Next</small>${next.title}</a>` : ""}</nav>`);
}

function components() {
  return docsShell(compNav(), `
    <h1>Components</h1>
    <p class="lead">Every component follows the current look and animates its own state. Each page has a live demo you can switch between looks and themes.</p>
    ${GROUPS.map((g) => {
      const it = COMPONENTS.filter((c) => c.group === g);
      return it.length ? `<h2>${g}</h2><div class="comp-list">${it.map((c) => `<a href="#/components/${c.slug}"><span>${c.name}</span><small>${c.summary.split(". ")[0]}.</small></a>`).join("")}</div>` : "";
    }).join("")}`);
}

function component(slug) {
  const i = COMPONENTS.findIndex((c) => c.slug === slug);
  if (i < 0) return notFound();
  const c = COMPONENTS[i];
  const prev = COMPONENTS[i - 1], next = COMPONENTS[i + 1];
  const looks = [["lumen", "Lumen"], ["mac", "macOS"], ["windows", "Windows"], ["gnome", "GNOME"]];
  return docsShell(compNav(), `
    <p class="crumb"><a href="#/components">Components</a> / ${c.group}</p>
    <h1>${c.name}</h1>
    <p class="lead">${c.summary}</p>
    <div class="demo">
      <div class="bar">
        <div class="seg" role="group" aria-label="Look">${looks.map(([k, n], j) => `<button type="button" data-look="${k}" aria-pressed="${j === 0}">${n}</button>`).join("")}</div>
        <a class="end" href="demo/index.html?demo=${c.demo}" target="_blank" rel="noopener">Open full window ↗</a>
      </div>
      <iframe data-demo="${c.demo}" data-embed="1" title="${c.name} — live lumen demo" style="height:${HEIGHTS[c.demo] || 300}px"></iframe>
    </div>
    <h2 id="usage">Usage</h2>
    ${pre(c.code)}
    <h2 id="options">Options</h2>
    <table class="options"><thead><tr><th>Method</th><th>Notes</th></tr></thead><tbody>${c.options.map(([m, d]) => `<tr><td>${esc(m)}</td><td>${esc(d) || "—"}</td></tr>`).join("")}</tbody></table>
    ${c.keys.length ? `<h2 id="keyboard">Keyboard</h2><table><tbody>${c.keys.map(([k, d]) => `<tr><td><kbd>${esc(k)}</kbd></td><td>${esc(d)}</td></tr>`).join("")}</tbody></table>` : ""}
    <nav class="next" aria-label="Pagination">${prev ? `<a href="#/components/${prev.slug}"><small>Previous</small>${prev.name}</a>` : "<span></span>"}${next ? `<a href="#/components/${next.slug}"><small>Next</small>${next.name}</a>` : ""}</nav>`);
}

function showcase() {
  const fig = (img, cap) => `<figure><img alt="${esc(cap)}" data-light="img/${img}-light.png" data-dark="img/${img}-dark.png" src="img/${img}-${isDark() ? "dark" : "light"}.png"><figcaption>${cap}</figcaption></figure>`;
  return docsShell(guideNav(), `
    <h1>Showcase</h1><p class="lead">Every image on this page was drawn by lumen, headless, from the same demos you can run.</p>
    <div class="shots">
      ${fig("hero", "Dashboard: sidebar, segmented control, date picker, stat tiles, area chart and donut")}
      ${fig("vault", "Vault preview")}
      ${fig("look-mac", "macOS look")}
      ${fig("look-windows", "Windows 11 look")}
      ${fig("look-gnome", "GNOME look")}
      ${fig("look-lumen", "Lumen look")}
      ${fig("chart-bar", "Stacked bar chart")}
      ${fig("calendar", "Calendar and range calendar")}
    </div>`);
}

function apps() {
  return docsShell(guideNav(), `
    <p class="crumb">Apps</p>
    <h1>Vault</h1>
    <p class="lead">A password manager built with lumen. It's the first full application on the kit, and it's being designed now.</p>
    <div style="margin-bottom:40px">${frame("vault", { title: "Vault preview", poster: "vault" })}</div>
    <h2>What it will be</h2>
    <ul>
      <li><b>Local first.</b> The vault is an encrypted file you own; sync is optional.</li>
      <li><b>Native on every platform.</b> macOS, Windows and GNOME looks with the system accent and fonts, from one codebase.</li>
      <li><b>One-time codes, password health and breach checks</b> in the same window.</li>
    </ul>
    <p>The feature set, cryptography and threat model are being researched separately before any code is written. This page will follow that work.</p>
    <h2>Try the preview</h2>
    ${pre("cargo run -p lumen-demos -- vault --look mac")}`);
}

function changelog() {
  return docsShell(guideNav(), `<h1>Changelog</h1><p class="lead">What changed, newest first.</p>
    ${CHANGELOG.map((r) => `<div class="release"><h2>${r.v}</h2><time datetime="${r.date}">${r.date}</time><ul>${r.items.map((i) => `<li>${esc(i)}</li>`).join("")}</ul></div>`).join("")}`);
}

function notFound() {
  return `<div class="wrap page" style="padding:160px 24px"><h1 style="font-size:48px">Nothing here.</h1><p style="color:var(--text-2)">That page doesn't exist. <a class="prose" href="#/" style="color:var(--action)">Go to the overview</a>.</p></div>`;
}

// -------------------------------------------------------------- router
function route() {
  const h = location.hash.replace(/^#\/?/, "");
  const [a, b] = h.split("/");
  let html, title = "lumen — Rust UI that looks finished";
  if (!a) html = home();
  else if (a === "guide") { html = guide(b); title = `${(GUIDES.find((g) => g.slug === b) || GUIDES[0]).title} — lumen`; }
  else if (a === "components" && b) { html = component(b); title = `${(COMPONENTS.find((c) => c.slug === b) || {}).name || "Component"} — lumen`; }
  else if (a === "components") { html = components(); title = "Components — lumen"; }
  else if (a === "showcase") { html = showcase(); title = "Showcase — lumen"; }
  else if (a === "apps") { html = apps(); title = "Vault — lumen"; }
  else if (a === "changelog") { html = changelog(); title = "Changelog — lumen"; }
  else html = notFound();
  document.title = title;
  view.innerHTML = html;
  decorate(view);
  activateFrames(view);
  // Look switcher on component pages.
  view.querySelectorAll(".demo .seg button").forEach((btn) => btn.addEventListener("click", () => {
    btn.parentElement.querySelectorAll("button").forEach((x) => x.setAttribute("aria-pressed", x === btn));
    const f = btn.closest(".demo").querySelector("iframe");
    f.dataset.look = btn.dataset.look;
    syncDemo(f);
  }));
  // Current-page markers.
  const here = "#/" + h;
  document.querySelectorAll(".side a, .links a").forEach((l) => {
    const href = l.getAttribute("href");
    const on = href === here || (href !== "#/" && here.startsWith(href) && l.closest(".links"));
    on ? l.setAttribute("aria-current", "page") : l.removeAttribute("aria-current");
  });
  $("#links").classList.remove("open");
  if (!location.hash.includes("#", 1)) window.scrollTo(0, 0);
  view.focus({ preventScroll: true });
}

// ------------------------------------------------------------- palette
const pal = $("#pal"), palIn = $("#pal input"), palList = $("#pal ul");
const ENTRIES = [
  ...GUIDES.map((g) => ({ t: g.title, k: "Guide", h: `#/guide/${g.slug}` })),
  ...COMPONENTS.map((c) => ({ t: c.name, k: c.group, h: `#/components/${c.slug}` })),
  { t: "Showcase", k: "Page", h: "#/showcase" }, { t: "Vault", k: "App", h: "#/apps" }, { t: "Changelog", k: "Page", h: "#/changelog" },
];
let sel = 0, results = ENTRIES, lastFocus = null;
function renderPal() {
  const q = palIn.value.trim().toLowerCase();
  results = q ? ENTRIES.filter((e) => (e.t + " " + e.k).toLowerCase().includes(q)) : ENTRIES;
  sel = Math.min(sel, Math.max(0, results.length - 1));
  palList.innerHTML = results.length
    ? results.map((e, i) => `<li role="option" aria-selected="${i === sel}"><a href="${e.h}">${esc(e.t)}<small>${esc(e.k)}</small></a></li>`).join("")
    : `<li class="empty">No page matches “${esc(palIn.value)}”.</li>`;
}
function openPal() { lastFocus = document.activeElement; pal.hidden = false; view.inert = true; palIn.value = ""; sel = 0; renderPal(); palIn.focus(); }
function closePal() { pal.hidden = true; view.inert = false; lastFocus?.focus?.(); }
palIn.addEventListener("input", () => { sel = 0; renderPal(); });
palIn.addEventListener("keydown", (e) => {
  if (e.key === "ArrowDown") { sel = Math.min(sel + 1, results.length - 1); renderPal(); e.preventDefault(); }
  else if (e.key === "ArrowUp") { sel = Math.max(sel - 1, 0); renderPal(); e.preventDefault(); }
  else if (e.key === "Enter" && results[sel]) { location.hash = results[sel].h; closePal(); }
});
pal.addEventListener("click", (e) => { if (e.target === pal || e.target.closest("a")) closePal(); });
addEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") { e.preventDefault(); pal.hidden ? openPal() : closePal(); }
  else if (e.key === "Escape" && !pal.hidden) closePal();
  else if (e.key === "/" && pal.hidden && !/input|textarea/i.test(document.activeElement.tagName)) { e.preventDefault(); openPal(); }
});

// ---------------------------------------------------------------- boot
$("#search").addEventListener("click", openPal);
$("#theme").addEventListener("click", () => setTheme(isDark() ? "light" : "dark", true));
$("#burger").addEventListener("click", () => { $("#links").classList.toggle("open"); $("#side")?.classList.toggle("open"); });
$("#gh").innerHTML = ICON.github;
$("#burger").innerHTML = ICON.menu;
sysDark.addEventListener("change", (e) => { if (!stored) setTheme(e.matches ? "dark" : "light"); });
setTheme(stored || (sysDark.matches ? "dark" : "light"));
addEventListener("hashchange", route);
route();
