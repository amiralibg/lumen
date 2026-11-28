// lumen site: Ethereal (site/DESIGN.md). One hash router, a settings panel
// that every live demo follows, and the dithered sky behind it all.
import { COMPONENTS, GROUPS, GUIDES, CHANGELOG, REPO, esc } from "./content.js";

const $ = (s, el = document) => el.querySelector(s);
const $$ = (s, el = document) => [...el.querySelectorAll(s)];
const view = $("#view");
const root = document.documentElement;

// ------------------------------------------------------------------ icons
const svg = (d) => `<svg viewBox="0 0 24 24" aria-hidden="true">${d}</svg>`;
const I = {
  moon: svg('<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>'),
  sun: svg('<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>'),
  sliders: svg('<path d="M20 7h-9M14 17H5"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>'),
  menu: svg('<path d="M4 7h16M4 12h16M4 17h16"/>'),
  github: svg('<path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65S8.93 17.38 9 18v4"/><path d="M9 18c-4.51 2-5-2-7-2"/>'),
  eye: svg('<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/>'),
  code: svg('<path d="m16 18 6-6-6-6M8 6l-6 6 6 6"/>'),
  split: svg('<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/>'),
  reset: svg('<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>'),
  full: svg('<path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/>'),
  ext: svg('<path d="M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>'),
  chev: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4.5 6.5 8 10l3.5-3.5"/></svg>',
  check: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3.5 8.5 3 3 6-7"/></svg>',
  search: svg('<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>'),
  layers: svg('<path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5M3 17l9 5 9-5"/>'),
  grid: svg('<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>'),
  palette: svg('<circle cx="13.5" cy="6.5" r="1"/><circle cx="17.5" cy="10.5" r="1"/><circle cx="8.5" cy="7.5" r="1"/><circle cx="6.5" cy="12.5" r="1"/><path d="M12 2a10 10 0 0 0 0 20c1.1 0 2-.9 2-2 0-.5-.2-1-.5-1.3-.3-.4-.5-.8-.5-1.3 0-1.1.9-2 2-2h2.4A5.6 5.6 0 0 0 22 9.8C22 5.5 17.5 2 12 2Z"/>'),
  lock: svg('<rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>'),
  chart: svg('<path d="M3 3v18h18"/><path d="m7 15 4-4 3 3 6-6"/>'),
  list: svg('<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>'),
  cal: svg('<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>'),
  menuI: svg('<rect x="4" y="4" width="16" height="16" rx="2"/><path d="M8 9h8M8 13h8M8 17h5"/>'),
  win: svg('<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/>'),
  bell: svg('<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>'),
};

// --------------------------------------------------------------- settings
// Names match lumen's Accent so the demos can take them as-is.
const ACCENTS = [["Neutral", "#9aa0b2"], ["Violet", "#7f58ee"], ["Blue", "#3b82f6"], ["Green", "#22b07d"], ["Amber", "#f5af20"], ["Orange", "#f77c10"], ["Coral", "#f46c5f"], ["Rose", "#e0336f"]];
const LOOKS = [["lumen", "Lumen"], ["mac", "macOS"], ["windows", "Windows"], ["gnome", "GNOME"]];
const KEY = "lumen-site";
const DEFAULTS = { accent: "Blue", custom: "#3b82f6", theme: "system", look: "lumen", speed: 1, reduced: false, split: false };
let S = { ...DEFAULTS };
try { Object.assign(S, JSON.parse(localStorage.getItem(KEY) || "{}")); } catch {}
const sysDark = matchMedia("(prefers-color-scheme: dark)");
const sysReduce = matchMedia("(prefers-reduced-motion: reduce)");
const isDark = () => S.theme === "dark" || (S.theme === "system" && sysDark.matches);
const reduced = () => S.reduced || sysReduce.matches;
const motion = () => (reduced() ? 0 : S.speed);
const accentHex = () => (S.accent === "custom" ? S.custom : (ACCENTS.find((a) => a[0] === S.accent) || ACCENTS[2])[1]);

function apply() {
  root.classList.add("no-anim");
  root.classList.toggle("t-dark", isDark());
  root.classList.toggle("t-light", !isDark());
  root.style.setProperty("--accent", accentHex());
  root.style.setProperty("--m", reduced() ? 0.001 : 1 / S.speed);
  root.dataset.reduced = String(reduced());
  requestAnimationFrame(() => requestAnimationFrame(() => root.classList.remove("no-anim")));
  try { localStorage.setItem(KEY, JSON.stringify({ ...S, accentHex: accentHex() })); } catch {}
  $("#theme").innerHTML = isDark() ? I.sun : I.moon;
  $("#theme").setAttribute("aria-label", isDark() ? "Switch to light theme" : "Switch to dark theme");
  syncPanel();
  $$(".live").forEach(syncLive);
  $$(".look[data-look]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.look === S.look)));
  $$("img[data-l]").forEach((i) => (i.src = isDark() ? i.dataset.d : i.dataset.l));
  $$("[data-split-btn]").forEach((b) => b.setAttribute("aria-pressed", String(S.split)));
  $$(".stage").forEach(renderPanes);
}
function set(k, v) { S[k] = v; apply(); }

// ------------------------------------------------------------ settings panel
let panel;
const seg = (key, opts) => `<div class="sp-seg" role="radiogroup" data-key="${key}">${opts.map(([v, l]) => `<button type="button" role="radio" data-v="${v}">${l}</button>`).join("")}<span class="sp-thumb" aria-hidden="true"></span></div>`;
const sw = (key, label) => `<div class="sp-row"><span>${label}</span><button type="button" class="sw" role="switch" data-key="${key}" aria-label="${label}"><i></i></button></div>`;
function buildPanel() {
  const btn = $("#settings");
  btn.innerHTML = I.sliders;
  panel = document.createElement("div");
  panel.className = "sp"; panel.id = "settings-panel"; panel.hidden = true;
  panel.setAttribute("role", "dialog"); panel.setAttribute("aria-label", "Appearance");
  panel.innerHTML = `
    <div class="sp-row"><div><span>Accent</span><small>Every live demo follows it</small></div>
      <div class="sp-dots">${ACCENTS.map(([n, c]) => `<button type="button" class="sp-dot" data-accent="${n}" style="--c:${c}" aria-label="${n} accent"></button>`).join("")}
        <label class="sp-dot sp-custom" aria-label="Custom accent"><input type="color" value="${S.custom}"></label></div></div>
    <div class="sp-row"><span>Theme</span>${seg("theme", [["light", "Light"], ["dark", "Dark"], ["system", "System"]])}</div>
    <div class="sp-row"><span>Look</span>${seg("look", LOOKS)}</div>
    <div class="sp-row"><span>Motion speed</span>${seg("speed", [["1", "1×"], ["0.5", "½×"], ["0.25", "¼×"], ["0.1", "⅒×"]])}</div>
    ${sw("reduced", "Reduced motion")}
    ${sw("split", "Light and dark side by side")}
    <button type="button" class="sp-reset">Reset to defaults</button>`;
  document.body.append(panel);
  panel.addEventListener("click", (e) => {
    const t = e.target.closest("button");
    if (!t) return;
    if (t.dataset.accent) set("accent", t.dataset.accent);
    else if (t.parentElement.classList.contains("sp-seg")) { const k = t.parentElement.dataset.key; set(k, k === "speed" ? Number(t.dataset.v) : t.dataset.v); }
    else if (t.classList.contains("sw")) set(t.dataset.key, !S[t.dataset.key]);
    else if (t.classList.contains("sp-reset")) { S = { ...DEFAULTS }; apply(); }
  });
  panel.querySelector("input[type=color]").addEventListener("change", (e) => { S.custom = e.target.value; set("accent", "custom"); });
  const place = () => { const r = btn.getBoundingClientRect(); panel.style.top = r.bottom + 10 + "px"; panel.style.right = Math.max(16, innerWidth - r.right) + "px"; };
  const open = (v) => {
    if (v) { place(); panel.hidden = false; syncPanel(); requestAnimationFrame(() => panel.classList.add("open")); }
    else { panel.classList.remove("open"); setTimeout(() => { if (!panel.classList.contains("open")) panel.hidden = true; }, 240); }
    btn.setAttribute("aria-expanded", String(v));
  };
  btn.setAttribute("aria-controls", "settings-panel"); btn.setAttribute("aria-expanded", "false");
  btn.addEventListener("click", () => open(panel.hidden));
  document.addEventListener("pointerdown", (e) => { if (!panel.hidden && !panel.contains(e.target) && !btn.contains(e.target)) open(false); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !panel.hidden) { open(false); btn.focus(); } });
  addEventListener("resize", () => !panel.hidden && place());
}
function thumbTo(group, sel, thumbSel) {
  const b = group.querySelector(sel), th = group.querySelector(thumbSel);
  if (b && th && b.offsetWidth) { th.style.width = b.offsetWidth + "px"; th.style.transform = `translateX(${b.offsetLeft}px)`; th.style.opacity = 1; }
  else if (th) th.style.opacity = 0;
}
function syncPanel() {
  if (!panel) return;
  $$(".sp-dot", panel).forEach((d) => d.setAttribute("aria-pressed", String((d.dataset.accent || "custom") === S.accent)));
  $(".sp-custom", panel).style.setProperty("--c", S.custom);
  $$(".sp-seg", panel).forEach((g) => {
    $$("button", g).forEach((b) => b.setAttribute("aria-checked", String(String(S[g.dataset.key]) === b.dataset.v)));
    thumbTo(g, '[aria-checked="true"]', ".sp-thumb");
  });
  $$(".sw", panel).forEach((s) => s.setAttribute("aria-checked", String(!!S[s.dataset.key])));
}

// ---------------------------------------------------------- custom dropdown
// A button plus a listbox popover. The site never opens the OS select.
function dropdown(id, label, value, opts) {
  const cur = opts.find((o) => o[0] === value) || opts[0];
  return `<div class="dd" data-dd="${id}"><button type="button" class="dd-btn" aria-haspopup="listbox" aria-expanded="false" aria-label="${esc(label)}"><span>${esc(cur[1])}</span>${I.chev}</button>
    <ul class="dd-menu" role="listbox" tabindex="-1" aria-label="${esc(label)}">${opts.map(([v, l]) => `<li role="option" data-v="${v}" aria-selected="${v === cur[0]}">${I.check}<span>${esc(l)}</span></li>`).join("")}</ul></div>`;
}
function wireDropdown(dd, onPick) {
  const btn = $(".dd-btn", dd), menu = $(".dd-menu", dd), items = [...menu.children];
  let active = 0;
  const mark = (i) => { active = (i + items.length) % items.length; items.forEach((x, k) => x.classList.toggle("active", k === active)); };
  const open = (v) => {
    dd.classList.toggle("open", v); btn.setAttribute("aria-expanded", String(v));
    if (v) { mark(Math.max(0, items.findIndex((x) => x.getAttribute("aria-selected") === "true"))); menu.focus({ preventScroll: true }); }
  };
  const pick = (li) => {
    items.forEach((x) => x.setAttribute("aria-selected", String(x === li)));
    const sp = $("span", btn); sp.textContent = $("span", li).textContent; sp.classList.remove("roll"); void sp.offsetWidth; sp.classList.add("roll");
    open(false); btn.focus(); onPick(li.dataset.v);
  };
  btn.addEventListener("click", () => open(!dd.classList.contains("open")));
  btn.addEventListener("keydown", (e) => { if (e.key === "ArrowDown" || e.key === "ArrowUp") { e.preventDefault(); open(true); } });
  items.forEach((li, i) => { li.addEventListener("pointermove", () => mark(i)); li.addEventListener("click", () => pick(li)); });
  menu.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") mark(active + 1); else if (e.key === "ArrowUp") mark(active - 1);
    else if (e.key === "Home") mark(0); else if (e.key === "End") mark(items.length - 1);
    else if (e.key === "Enter" || e.key === " ") pick(items[active]);
    else if (e.key === "Escape" || e.key === "Tab") { open(false); btn.focus(); }
    else return;
    e.preventDefault(); e.stopPropagation();
  });
  document.addEventListener("pointerdown", (e) => { if (!dd.contains(e.target)) open(false); });
}

// ------------------------------------------------------------- highlight
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
  $$("pre > code", el).forEach((c) => {
    if (c.dataset.done) return;
    c.dataset.done = 1;
    if (!c.classList.contains("lang-sh")) c.innerHTML = highlight(c.textContent);
    const b = document.createElement("button");
    b.className = "copy"; b.type = "button"; b.textContent = "Copy";
    b.addEventListener("click", async () => {
      try { await navigator.clipboard.writeText(c.textContent); b.textContent = "Copied"; } catch { b.textContent = "Press ⌘C"; }
      setTimeout(() => (b.textContent = "Copy"), 1400);
    });
    c.parentElement.append(b);
  });
}
const pre = (s, lang = "rust") => `<pre><code class="lang-${lang}">${esc(s)}</code></pre>`;

// -------------------------------------------------------------- live demos
const HEIGHTS = { "app-mail": 760, "app-music": 760, "app-tracker": 760, "app-notes": 760, "app-assistant": 760, "app-finance": 760, "app-todo": 760, "app-settings": 760, "code-editor": 460, "file-browser": 460, "emoji-picker": 460, "wheel-picker": 400, "signature-pad": 440, "avatar-upload": 400, "time-dial": 520, "aspect-ratio": 420, "ruler-slider": 420, "hex-viewer": 420, "tree-table": 420, "problems-panel": 460, "stack-trace": 480, "blame-view": 440, "coverage-view": 440, "query-builder": 460, "regex-tester": 520, "feature-flags": 480, "dependency-graph": 460, "veil": 340, "onboarding-checklist": 520, "swipe-actions": 460, "docking-panels": 460, "bubble-chart": 460, "chart-annotations": 420, "dot-matrix": 360, "calendar-heatmap": 400, tour: 460, "list-view": 460, bento: 780, hero: 560, vault: 600, looks: 440, "scene-ide": 640, "scene-observability": 640, "scene-shader": 640, "scene-release": 760, "scene-finance": 640, "scene-collab": 640, "scene-governance": 640, "scene-schedule": 640, calendar: 420, "date-picker": 460, "date-range-picker": 480, "time-picker": 420, "color-picker": 460, combobox: 400, "multi-select": 400, select: 340, menu: 380, popover: 340, dialog: 400, sheet: 460, "command-palette": 460, "hover-card": 340, "context-menu": 380, "confirm-popover": 340, toast: 340, "chart-line": 380, "chart-area": 380, "chart-bar": 380, donut: 300, gauge: 260, sparkline: 260, "progress-ring": 280, "bar-list": 340, heatmap: 300, "radar-chart": 380, "scatter-chart": 360, "funnel-chart": 340, candlestick: 360, kanban: 360, "ai-reasoning": 340, "ai-composer": 320, "ai-chat": 340, "diff-viewer": 320, waterfall: 300, "uptime-bars": 340, "rolling-number": 300, dock: 260, waveform: 260, "window-chrome": 320, "split-pane": 320, "menu-bar": 320, terminal: 320, gantt: 320, treemap: 340, markdown: 400, "property-grid": 340, "flame-graph": 300, "notification-list": 340, "spotlight-card": 280, "editable-text": 280, "money-field": 340, "masked-input": 320, "mention-input": 340, "step-slider": 320, "transfer-list": 360, "reorderable-list": 360, "editor-tabs": 320, "table-of-contents": 300, "nav-bar": 300, carousel: 340, callout: 300, "undo-toast": 300, "image-frame": 340, "sankey-chart": 360, "contribution-graph": 300, histogram: 340, "kpi-strip": 260, "day-schedule": 360, "node-editor": 340, "commit-graph": 340, "test-explorer": 340, "data-grid": 340, "filter-bar": 320, "ai-citations": 360, "text-effects": 300, "rotary-knob": 300, "compare-slider": 320, countdown: 260, "checklist-card": 360, "action-bar": 320, "speed-dial": 320, "numeric-pad": 380, "dimension-input": 300, "stacked-meter": 260, "announcement-bar": 220, "quantity-picker": 280, "swatch-picker": 260, "date-strip": 320, "world-clocks": 260, "bullet-chart": 320, "metric-table": 320, "ai-model-picker": 340, "context-meter": 280, "api-inspector": 340, "permission-matrix": 340, "description-list": 320, "masonry-grid": 380, "collapsible-card": 340, marquee: 260, "presence-pile": 260, "comment-thread": 360, "reaction-bar": 280, "attachment-list": 320, "progress-steps": 320, "shortcut-bar": 240, "view-switcher": 260, "gradient-slider": 300, "schedule-picker": 320, "quarter-picker": 300, "win-loss-chart": 340, "waffle-chart": 300, "schema-viewer": 340, "env-switcher": 280, "memory-profiler": 340, "ai-suggestion": 360, "prompt-templates": 340, "pricing-cards": 400, "ab-comparison": 320, equalizer: 380, card: 340, "text-input": 360, "text-area": 340, "number-field": 340, "tag-input": 280, "otp-input": 300, "password-input": 360, "search-field": 360, alert: 320, skeleton: 340, "empty-state": 340, breadcrumb: 280, pagination: 300, steps: 300, kbd: 300, "radio-cards": 400, rating: 260, "range-slider": 280, "shortcut-recorder": 260, "split-button": 360, "hold-button": 260, dropzone: 380, toolbar: 240, "status-bar": 220, meter: 260, accordion: 360, table: 340, tree: 340, timeline: 360, "code-block": 300, "json-viewer": 340, "key-value": 340, "log-viewer": 360, stepper: 260, chips: 220, "toggle-group": 220, avatar: 240, checkbox: 320, switch: 280, radio: 260, slider: 260, tooltip: 240, badge: 260, progress: 260, tabs: 260, segmented: 200, button: 260 };
const FULL = new Set(["app-mail", "app-music", "app-tracker", "app-notes", "app-assistant", "app-finance", "app-todo", "app-settings", "bento", "hero", "vault", "looks", "scene-ide", "scene-observability", "scene-shader", "scene-release", "scene-finance", "scene-collab", "scene-governance", "scene-schedule"]); // demos with their own background
function demoSrc(name, { dark = isDark(), look = S.look } = {}) {
  const p = new URLSearchParams({ demo: name, dark: dark ? "1" : "0" });
  if (!FULL.has(name)) p.set("embed", "1");
  if (look !== "lumen") p.set("look", look);
  p.set("accent", S.accent === "custom" ? S.custom : S.accent);
  if (S.speed !== 1) p.set("speed", String(S.speed));
  if (reduced()) p.set("reduced", "1");
  return `demo/index.html?${p}`;
}
const poster = (name, dark) => `img/c-${name}-${dark ? "dark" : "light"}.png`;
// One live demo. It loads when it nears the viewport and unloads when it
// leaves, so a page never runs more than a few WebAssembly instances.
// Desktop compositions (scenes, the bento) don't reflow like apps do: on a
// narrow screen they render at this width and scale down as a miniature.
const minWidth = (name) => (/^(bento|hero)$/.test(name) ? 960 : /^(looks|vault|scene-)/.test(name) ? 760 : 0);
const fitRO = new ResizeObserver((es) => es.forEach((e) => fitLive(e.target)));
function fitLive(el) {
  const minw = +el.dataset.minw, h = +el.dataset.h, f = $("iframe", el);
  // Only on narrow screens: a slight scale on desktop would just blur text.
  const s = el.clientWidth < 720 ? Math.min(1, el.clientWidth / minw) : 1;
  if (s < 1 && !document.fullscreenElement) {
    el.style.height = Math.round(h * s) + "px";
    Object.assign(f.style, { width: minw + "px", height: h + "px", transform: `scale(${s})`, transformOrigin: "0 0" });
  } else {
    el.style.height = document.fullscreenElement ? "" : h + "px";
    Object.assign(f.style, { width: "", height: "", transform: "", transformOrigin: "" });
  }
}
function live(name, { height, dark, look } = {}) {
  const h = height || HEIGHTS[name] || 320;
  const minw = minWidth(name);
  return `<div class="live" data-demo="${name}" data-h="${h}" ${minw ? `data-minw="${minw}"` : ""} ${dark !== undefined ? `data-dark="${dark ? 1 : 0}"` : ""} ${look ? `data-look="${look}"` : ""} style="height:${h}px">
    <img class="poster" alt="" src="${poster(name, dark ?? isDark())}" onerror="this.style.display='none'">
    <span class="loading">Loading live demo</span>
    <iframe title="${esc(name)}: live lumen demo" tabindex="0"></iframe></div>`;
}
function liveSrc(el) {
  const dark = el.dataset.dark !== undefined ? el.dataset.dark === "1" : isDark();
  return demoSrc(el.dataset.demo, { dark, look: el.dataset.look || S.look });
}
function syncLive(el) {
  const dark = el.dataset.dark !== undefined ? el.dataset.dark === "1" : isDark();
  const p = $(".poster", el); if (p) p.src = poster(el.dataset.demo, dark);
  if (!el._on) return;
  const src = liveSrc(el), f = $("iframe", el);
  if (f.getAttribute("src") !== src) { el.classList.remove("ready"); f.src = src; }
}
const io = new IntersectionObserver((es) => es.forEach((e) => {
  const el = e.target, f = $("iframe", el);
  if (e.isIntersecting && !el._on) { el._on = true; f.src = liveSrc(el); }
  else if (!e.isIntersecting && el._on) { el._on = false; el.classList.remove("ready"); f.removeAttribute("src"); }
}), { rootMargin: "300px 0px" });
addEventListener("message", (e) => {
  if (e.data === "lumen-lost") {
    // Out of GPU contexts: back to the poster, then try again if still visible.
    $$(".live iframe").forEach((f) => {
      if (f.contentWindow !== e.source) return;
      const l = f.closest(".live");
      l.classList.remove("ready"); l._on = false; f.removeAttribute("src");
      setTimeout(() => { io.unobserve(l); io.observe(l); }, 800);
    });
    return;
  }
  if (e.data !== "lumen-ready") return;
  $$(".live iframe").forEach((f) => { if (f.contentWindow === e.source) f.closest(".live").classList.add("ready"); });
});
function activate(el) { $$(".live", el).forEach((l) => { io.observe(l); if (l.dataset.minw) fitRO.observe(l); }); }

// A stage: a framed live demo with a toolbar (preview/code, look, side by
// side, reset, full screen, open in a window).
let stageN = 0;
function stage(demo, { height, code, title = "" } = {}) {
  const id = "st" + ++stageN;
  return `<div class="stage" id="${id}" data-demo="${demo}" data-h="${height || HEIGHTS[demo] || 320}" data-look="${S.look}" data-view="preview">
    <div class="bar">
      ${code ? `<div class="tabs2 glass" role="tablist"><button role="tab" aria-selected="true" data-view="preview">${I.eye}Preview</button><button role="tab" aria-selected="false" data-view="code">${I.code}Code</button><span class="seg-thumb"></span></div>` : ""}
      <span class="grow"></span>
      ${dropdown(id + "-look", "Look", S.look, LOOKS)}
      <button class="ib" type="button" data-split-btn aria-pressed="${S.split}" aria-label="Light and dark side by side" title="Light and dark side by side">${I.split}</button>
      <button class="ib" type="button" data-act="reset" aria-label="Restart the demo" title="Restart">${I.reset}</button>
      <button class="ib" type="button" data-act="full" aria-label="Full screen" title="Full screen">${I.full}</button>
      <a class="ib" href="${demoSrc(demo)}" data-act="open" target="_blank" rel="noopener" aria-label="Open ${esc(title || demo)} in a new tab" title="Open in a new tab">${I.ext}</a>
    </div>
    <div class="frame"><div class="panes"></div>${code ? `<div class="code-pane" hidden>${pre(code)}</div>` : ""}</div>
  </div>`;
}
function renderPanes(st) {
  const panes = $(".panes", st), h = +st.dataset.h, look = st.dataset.look, demo = st.dataset.demo;
  const want = S.split ? `two:${demo}:${look}` : `one:${demo}:${look}`;
  if (panes.dataset.k === want) return;
  panes.dataset.k = want;
  $$(".live", panes).forEach((l) => io.unobserve(l));
  panes.classList.toggle("two", S.split);
  panes.innerHTML = S.split ? live(demo, { height: h, dark: false, look }) + live(demo, { height: h, dark: true, look }) : live(demo, { height: h, look });
  activate(panes);
  const o = $('[data-act="open"]', st); if (o) o.href = demoSrc(demo, { look });
}
function wireStage(st) {
  renderPanes(st);
  const dd = $(".dd", st);
  if (dd) wireDropdown(dd, (v) => { st.dataset.look = v; renderPanes(st); });
  $$('[data-split-btn]', st).forEach((b) => b.addEventListener("click", () => set("split", !S.split)));
  $('[data-act="reset"]', st).addEventListener("click", () => $$(".live", st).forEach((l) => { if (l._on) { l.classList.remove("ready"); const f = $("iframe", l); f.src = "about:blank"; setTimeout(() => (f.src = liveSrc(l)), 30); } }));
  $('[data-act="full"]', st).addEventListener("click", () => { const f = $(".frame", st); document.fullscreenElement ? document.exitFullscreen() : f.requestFullscreen?.(); });
  const tabs = $(".tabs2", st);
  if (tabs) {
    const sync = () => thumbTo(tabs, '[aria-selected="true"]', ".seg-thumb");
    requestAnimationFrame(sync);
    tabs.addEventListener("click", (e) => {
      const b = e.target.closest("button"); if (!b) return;
      $$("button", tabs).forEach((x) => x.setAttribute("aria-selected", String(x === b)));
      const code = b.dataset.view === "code";
      $(".panes", st).hidden = code; $(".code-pane", st).hidden = !code;
      sync();
    });
  }
}

// ------------------------------------------------------------------ pages
const count = COMPONENTS.length;
const BENTO_CODE = `fn bento(ui: &mut Ui, s: &mut State) {
    ui.card().title("Add a runner").show(|ui| {
        ui.segmented(&mut s.os, &[(0, "Linux"), (1, "macOS"), (2, "Windows")]);
        ui.text_input(&mut s.runner).label("Name").fill_width();
        ui.select(&mut s.region, &REGIONS).label("Region").fill_width();
        ui.row().gap(6).show(|ui| {
            for (i, l) in ["arm64", "gpu", "docker", "nightly"].iter().enumerate() {
                ui.chip(l, &mut s.labels[i]);
            }
        });
        ui.stepper("jobs", &mut s.jobs, 1, 16);
        if ui.button("Add runner").primary().clicked() {
            ui.toast(Toast::new("Runner added").success());
        }
    });
    ui.card().show(|ui| ui.stat("Active users", "8,214", Some(4.1), Some(&trend)));
    ui.card().show(|ui| {
        ui.calendar_range(&mut s.span);
        ui.toggle_group("slots", &mut s.slot, &SLOTS);
    });
}`;
const SCENES = [
  ["bento", "Overview", I.layers, BENTO_CODE],
  ["scene-ide", "AI IDE", I.code],
  ["scene-observability", "Observability", I.chart],
  ["scene-shader", "Shader Studio", I.palette],
  ["scene-release", "Releases", I.layers],
  ["scene-finance", "Analytics", I.chart],
  ["looks", "Looks", I.palette],
];

function home() {
  const tile = (demo, title, desc, slug, icon, w = "", h) => `
    <article class="tile glass glow ${w}">
      <div class="tile-head"><span class="ic">${icon}</span><div><h3>${title}</h3><p>${desc}</p></div><a href="#/components/${slug}">Docs →</a></div>
      ${live(demo, { height: h })}
    </article>`;
  return `
  <header class="hero">
    <span class="eyebrow glass"><b>new</b>${count} live components across AI, data, charts, devtools and motion</span>
    <h1>Rust UI, <span class="lit">lit from within.</span></h1>
    <p class="lede">lumen draws every pixel itself: real type, spring physics, and looks that turn into macOS, Windows or GNOME on demand. One Rust codebase, native and on the web.</p>
    <div class="ctas"><a class="btn main" href="#/guide/getting-started">Get started <span class="arr">→</span></a><a class="btn" href="#/components">Browse components</a></div>
    <div><span class="clone glass">$ cargo add --git ${REPO.replace("https://", "")} lumen <button type="button" data-copy="cargo add --git ${REPO} lumen">Copy</button></span></div>
  </header>

  <section class="wrap reveal">
    <div class="scenes glass" role="tablist" aria-label="Scenes">${SCENES.map(([d, n, ic], i) => `<button role="tab" data-scene="${i}" aria-selected="${i === 0}">${ic}${n}</button>`).join("")}<span class="seg-thumb"></span></div>
    <div id="scene">${stage("bento", { code: BENTO_CODE, title: "Overview" })}</div>
    <ul class="glance glass" aria-label="At a glance">
      <li><b>${count}</b><span>components</span></li>
      <li><b>4</b><span>looks</span></li>
      <li><b>8 + any</b><span>accents</span></li>
      <li><b>macOS · Windows · Linux · Web</b><span>one Rust codebase</span></li>
      <li><b>0</b><span>webviews</span></li>
    </ul>
  </section>

  <section class="band wrap reveal">
    <div class="band-head">
      <div><span class="kicker"><b>01</b>Components</span><h2>Everything an app needs, already finished.</h2></div>
      <p>Not a box of primitives: complete components designed together, with every state drawn and every change animated. Each tile below is running; click into one and use it. <a class="more" href="#/components">All ${count} <span class="arr">→</span></a></p>
    </div>
    <div class="tiles">
      ${tile("select", "Select", "A custom menu, never the OS one. Rows arrive in turn.", "select", I.list, "w3", 320)}
      ${tile("date-picker", "Date picker", "Months slide as you page; ranges join across weeks.", "date-picker", I.cal, "w3", 320)}
      ${tile("chart-area", "Charts", "Lines draw themselves in; values spring to new data.", "area-chart", I.chart, "w4", 360)}
      ${tile("chips", "Filter chips", "A check springs in and pushes the label over.", "chips", I.grid, "w2", 360)}
      ${tile("menu", "Menu", "Anchored, flips to fit, returns focus on close.", "menu", I.menuI, "w2", 340)}
      ${tile("dialog", "Dialog", "Rises on a spring over a scrim.", "dialog", I.win, "w2", 340)}
      ${tile("toast", "Toast", "Stacks, pauses on hover, leaves on its own.", "toast", I.bell, "w2", 340)}
    </div>
  </section>

  <section class="band wrap reveal">
    <div class="split">
      <div>
        <span class="kicker"><b>02</b>The model</span>
        <h2 style="font:600 clamp(32px,4vw,52px)/1.04 var(--font-display);letter-spacing:-0.04em;margin:12px 0 0">Your state is a struct. The interface is a function of it.</h2>
        ${pre(`#[derive(Default)]
struct State { wifi: bool, volume: f32 }

lumen::App::new("Settings").run(State::default(), |ui, s| {
    ui.card().title("Quick settings").show(|ui| {
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
      <div class="frame">${live("card", { height: 420 })}</div>
    </div>
  </section>

  <section class="band wrap reveal">
    <div class="band-head">
      <div><span class="kicker"><b>03</b>Looks</span><h2>One app. Four native accents.</h2></div>
      <p>Switching blends colors, radii, control sizes and motion together. Pick a look here or in the settings panel, and every live demo on the site follows it.</p>
    </div>
    <div class="looks">${[["lumen", "Lumen", "Ethereal"], ["mac", "macOS", "Sonoma"], ["windows", "Windows 11", "Fluent"], ["gnome", "GNOME", "Adwaita"]].map(([k, n, s]) =>
      `<button class="look glass" type="button" data-look="${k}" aria-pressed="${S.look === k}"><img alt="${n} look" data-l="img/look-${k}-light.png" data-d="img/look-${k}-dark.png" src="img/look-${k}-${isDark() ? "dark" : "light"}.png"><span>${n}<small>${s}</small></span></button>`).join("")}</div>
    <div class="accents">Accent ${ACCENTS.map(([n, c]) => `<button type="button" class="sp-dot" data-accent="${n}" style="--c:${c}" aria-pressed="${S.accent === n}" aria-label="${n} accent"></button>`).join("")}</div>
  </section>

  <section class="band wrap reveal">
    <div class="band-head"><div><span class="kicker"><b>04</b>Where it fits</span><h2>What lumen is, and what it isn't.</h2></div><p>An immediate-mode kit that spends its effort on finish. It draws its own widgets; it doesn't wrap the OS's.</p></div>
    <div class="compare"><table>
      <thead><tr><th></th><th>How it draws</th><th>Where lumen differs</th></tr></thead>
      <tbody>
        <tr><td>egui</td><td>Immediate mode, own widgets</td><td>Same model. lumen spends its effort on finish: OKLab theming, springs, real type, platform looks.</td></tr>
        <tr><td>GPUI</td><td>Retained elements, own renderer</td><td>Zed's framework, desktop only. lumen is immediate mode, runs in the browser too, and can live inside a GPUI app through <a href="#/guide/gpui">lumen-gpui</a>.</td></tr>
        <tr><td>iced, Slint</td><td>Retained or declarative, own widgets</td><td>Elm-style messages or a markup language. lumen is plain Rust functions over your state.</td></tr>
        <tr><td>Tauri, Electron</td><td>A web view</td><td>Those ship HTML and a browser engine. lumen is one native binary drawing with the GPU.</td></tr>
        <tr><td>AppKit, WinUI, GTK</td><td>The OS's own widgets</td><td>lumen doesn't wrap them. Its looks follow their conventions, but they're lumen widgets.</td></tr>
      </tbody></table></div>
  </section>

  <section class="band wrap reveal">
    <div class="vault glass">
      <div>
        <span class="row-chips"><span class="chip acc">Early preview · 0.0.6</span><span class="chip">Built with GPUI + lumen</span></span>
        <h2 class="wl-title"><img class="wl-icon" src="assets/wardlight-icon.svg" alt="" width="56" height="56">Wardlight</h2>
        <p class="wl-sub">Passwords, one-time codes and SSH keys in one encrypted vault. A native desktop app for macOS, Windows and Linux.</p>
        <p><b>GPUI</b>, Zed's framework, runs the window, the title bar and the keyboard. <b>lumen</b> draws every screen inside it through <a href="#/guide/gpui">lumen-gpui</a>: lists, forms, dialogs, the ⌘K palette and the code countdown.</p>
        <p class="wl-acts"><a class="btn main" href="#/apps">About Wardlight <span class="arr">→</span></a><a class="btn" href="${WARDLIGHT}/releases/latest" target="_blank" rel="noopener">Download</a><a class="btn" href="${WARDLIGHT}" target="_blank" rel="noopener">Source</a></p>
      </div>
      <div class="frame wl-shot"><img alt="Wardlight's Passwords screen: logins on the left, GitHub's details on the right with its one-time code counting down" data-l="img/wardlight-passwords-light.png" data-d="img/wardlight-passwords-dark.png" src="img/wardlight-passwords-${isDark() ? "dark" : "light"}.png"></div>
    </div>
  </section>`;
}
function wireHome() {
  const tabs = $(".scenes"), sync = () => thumbTo(tabs, '[aria-selected="true"]', ".seg-thumb");
  requestAnimationFrame(sync); addEventListener("resize", sync);
  tabs.addEventListener("click", (e) => {
    const b = e.target.closest("button"); if (!b) return;
    $$("button", tabs).forEach((x) => x.setAttribute("aria-selected", String(x === b)));
    sync();
    const [demo, name, , code] = SCENES[+b.dataset.scene];
    $$("#scene .live").forEach((l) => io.unobserve(l));
    $("#scene").innerHTML = stage(demo, { code, title: name });
    wireStage($("#scene .stage"));
    decorate($("#scene"));
  });
  $$(".look[data-look]").forEach((b) => b.addEventListener("click", () => set("look", b.dataset.look)));
  $$(".accents .sp-dot").forEach((b) => b.addEventListener("click", () => { set("accent", b.dataset.accent); $$(".accents .sp-dot").forEach((x) => x.setAttribute("aria-pressed", String(x === b))); }));
}

const guideNav = () => `<h4>Guide</h4>${GUIDES.map((g) => `<a href="#/guide/${g.slug}">${g.title}</a>`).join("")}<h4>Reference</h4><a href="#/components">Components</a><a href="#/showcase">Showcase</a><a href="#/apps">Apps</a><a href="#/changelog">Changelog</a>`;
const compNav = () => `<h4>Overview</h4><a href="#/components">All components</a>` + GROUPS.map((g) => {
  const it = COMPONENTS.filter((c) => c.group === g);
  return it.length ? `<h4>${g}</h4>${it.map((c) => `<a href="#/components/${c.slug}">${c.name}</a>`).join("")}` : "";
}).join("");
const docsShell = (side, body, wide) => `<div class="docs"><aside class="side glass" id="side" aria-label="Section navigation">${side}</aside><article class="doc${wide ? " wide" : ""}">${body}</article></div>`;

function guide(slug) {
  const i = Math.max(0, GUIDES.findIndex((g) => g.slug === slug));
  const g = GUIDES[i], prev = GUIDES[i - 1], next = GUIDES[i + 1];
  return docsShell(guideNav(), `
    <p class="crumb"><a href="#/guide/getting-started">Guide</a> / ${String(i + 1).padStart(2, "0")}</p>
    <h1>${g.title}</h1><p class="lead">${g.lead}</p>
    <div class="prose">${g.body}</div>
    ${pager(prev && [`#/guide/${prev.slug}`, prev.title], next && [`#/guide/${next.slug}`, next.title])}`);
}
const pager = (prev, next) => `<nav class="next" aria-label="Pagination">${prev ? `<a class="glass" href="${prev[0]}"><small>Previous</small>${prev[1]}</a>` : "<span></span>"}${next ? `<a class="glass" href="${next[0]}"><small>Next</small>${next[1]}</a>` : ""}</nav>`;

function components() {
  return docsShell(compNav(), `
    <p class="crumb">Components</p>
    <h1>${count} components, each one live.</h1>
    <p class="lead">Every component follows the current look and animates its own state. The pictures are drawn by lumen; open one to use it, switch looks, or see light and dark side by side.</p>
    <div class="filters">
      <label class="filter-in glass">${I.search}<input type="search" placeholder="Filter components" aria-label="Filter components" spellcheck="false"><kbd>/</kbd></label>
      <div class="pills" role="group" aria-label="Groups"><button type="button" data-g="" aria-pressed="true">All</button>${GROUPS.map((g) => `<button type="button" data-g="${esc(g)}" aria-pressed="false">${g}</button>`).join("")}</div>
    </div>
    <div id="cgroups">${GROUPS.map((g) => {
      const it = COMPONENTS.filter((c) => c.group === g);
      return it.length ? `<section data-group="${esc(g)}"><div class="grid-h"><h2>${g}</h2><span>${it.length}</span></div><div class="cgrid">${it.map((c) => `
        <a class="ccard glass glow" href="#/components/${c.slug}" data-name="${esc((c.name + " " + c.summary).toLowerCase())}">
          <div class="thumb"><img loading="lazy" alt="" data-l="${poster(c.demo, false)}" data-d="${poster(c.demo, true)}" src="${poster(c.demo, isDark())}"></div>
          <div class="t"><b>${c.name}</b><p>${c.summary}</p></div></a>`).join("")}</div></section>` : "";
    }).join("")}</div>
    <p class="empty glass" hidden>No component matches. Try “date”, “chart” or “menu”.</p>`, true);
}
function wireComponents() {
  const q = $(".filter-in input"); let g = "";
  const run = () => {
    const t = q.value.trim().toLowerCase(); let any = false;
    $$("#cgroups section").forEach((sec) => {
      let n = 0;
      $$(".ccard", sec).forEach((c) => { const on = (!g || sec.dataset.group === g) && (!t || c.dataset.name.includes(t)); c.hidden = !on; n += on; });
      sec.hidden = !n; any ||= n > 0;
    });
    $(".empty").hidden = any;
  };
  q.addEventListener("input", run);
  $$(".pills button").forEach((b) => b.addEventListener("click", () => { g = b.dataset.g; $$(".pills button").forEach((x) => x.setAttribute("aria-pressed", String(x === b))); run(); }));
}

function component(slug) {
  const i = COMPONENTS.findIndex((c) => c.slug === slug);
  if (i < 0) return notFound();
  const c = COMPONENTS[i], prev = COMPONENTS[i - 1], next = COMPONENTS[i + 1];
  return docsShell(compNav(), `
    <p class="crumb"><a href="#/components">Components</a> / ${c.group}</p>
    <h1>${c.name}</h1>
    <p class="lead">${c.summary}</p>
    ${stage(c.demo, { code: c.code, title: c.name })}
    <h2 id="usage">Usage</h2>
    ${pre(c.code)}
    <h2 id="options">Options</h2>
    <table class="options"><thead><tr><th>Method</th><th>Notes</th></tr></thead><tbody>${c.options.map(([m, d]) => `<tr><td>${esc(m)}</td><td>${esc(d) || "—"}</td></tr>`).join("")}</tbody></table>
    ${c.keys.length ? `<h2 id="keyboard">Keyboard</h2><table><thead><tr><th>Key</th><th>Action</th></tr></thead><tbody>${c.keys.map(([k, d]) => `<tr><td><kbd>${esc(k)}</kbd></td><td>${esc(d)}</td></tr>`).join("")}</tbody></table>` : ""}
    ${pager(prev && [`#/components/${prev.slug}`, prev.name], next && [`#/components/${next.slug}`, next.name])}`);
}

const WARDLIGHT = "https://github.com/amiralibg/wardlight";
const SRC = `${REPO}/blob/main/crates/lumen-demos/src/main.rs`;
const SRC_DIR = `${REPO}/blob/main/crates/lumen-demos/src`;
const SHOWS = [
  ["bento", "Overview bento", "Forms, stats, settings, reviewers, a release window and regional uptime in one balanced three-column grid: segmented control, inputs, chips, stepper, switches, slider, avatars, range calendar, toggle group and uptime bars.", ["Forms", "Selection", "Dates", "Charts"], "scenes/bento.rs", BENTO_CODE],
  ["scene-ide", "AI coding workspace", "A resizable split-pane Rust + WGSL editor pairing a file explorer tree, closable editor tabs, inline AI patch diff review, a multi-model prompt composer with context budget, and a live LSP status bar.", ["AI", "Developer tools", "Navigation", "Data"], "scenes/ide.rs", `ui.split_pane("ide_split", &mut s.split_ratio).height(pane_h).show(\n    |ui| { ui.tree("ide_tree", &tree, &mut s.tree_open, &mut s.tree_sel); },\n    |ui| {\n        ui.editor_tabs("ide_tabs", &s.ed_tabs, &mut s.ed_active);\n        ui.code_suggestion("sdf_patch", "shader.wgsl", rationale, &removed, &added, &mut s.patch_decision);\n        ui.prompt_composer("ide_composer", &mut s.prompt, &mut s.ai_model)\n            .attachments(&mut s.ai_files).context_used(0.36).show();\n    },\n);`],
  ["scene-observability", "Observability & profiling console", "Executive KPI sparklines over an interactive click-to-zoom CPU/GPU icicle flame graph, a distributed request trace waterfall, and a live filterable log stream.", ["Developer tools", "Charts", "Data"], "scenes/observability.rs", `ui.kpi_strip("obs_kpis", &metrics);\nui.row().fill_width().gap(12).show(|ui| {\n    ui.card().title("CPU / GPU frame flame graph").show(|ui| ui.flame_graph("obs_flame", &frames));\n    ui.card().title("Distributed request waterfall").show(|ui| ui.waterfall("obs_trace", &spans));\n});\nui.log_viewer("obs_logs", &logs).height(144.0).show();`],
  ["scene-shader", "Visual shader studio", "A draggable WGSL node graph wired into a two-column uniform property inspector, a live wireframe-vs-SDF comparison split, and perceptual OKLab gradient sliders.", ["Planning", "Inputs", "Selection", "Actions"], "scenes/shader.rs", `ui.toolbar("shader_tb", |tb| { tb.toggle("grid", "Snap grid", &mut s.snap_grid); tb.primary("Compile WGSL"); });\nui.node_editor("shader_dag", &mut s.nodes, &wires, 224.0);\nui.property_inspector("shader_props", |insp| {\n    insp.number("Corner radius", &mut s.corner_radius, 0.0..=48.0, "px");\n    insp.number("Shadow blur", &mut s.blur_sigma, 0.0..=64.0, "px");\n});\nui.compare_split("sdf_ab", &mut s.compare_ratio, "Wireframe mesh", "SDF shaded");`],
  ["scene-release", "Release control plane", "Environment lock banner with promotion split-button, interactive Git commit DAG, 45-day regional SLO uptime strips, and a multi-stage rollout Kanban board.", ["Developer tools", "Planning", "Feedback"], "scenes/release.rs", `ui.env_banner("rel_env", &mut s.env_idx, "eu-central-1", &mut s.env_locked);\nui.row().fill_width().gap(12).show(|ui| {\n    ui.commit_graph("rel_commits", &commits, &mut s.commit_sel);\n    ui.card().title("Regional SLO & Uptime").show(|ui| ui.uptime_bars("u_edge", "Edge Ingress", &bars));\n});\nui.kanban("rel_kanban", &cols, &mut s.cards);`],
  ["scene-finance", "Financial & analytics terminal", "Rolling portfolio valuation, interactive OHLC candlestick chart, multi-asset sparkline watchlist, ARR-to-FCF Sankey flow diagram, and Q4 operating target bullet charts.", ["Charts", "Data", "Motion"], "scenes/finance.rs", `ui.rolling_number("nav_val", s.portfolio_usd).prefix("$").role(TextRole::Heading2).show();\nui.candlestick_chart("fin_ohlc", &candles).height(190.0).show();\nui.metric_table("fin_watch", &rows);\nui.sankey_chart("fin_sankey", &nodes, &links, 176.0);\nui.bullet_chart("fin_bullets", &bullets);`],
  ["scene-collab", "Collaborative spec & RFC studio", "Multiplayer presence pile with viewport follow, inline editable heading, rendered Markdown spec with emoji reactions and build attachments, threaded code review comments, and release sign-off gates.", ["Data", "Feedback", "Inputs", "Planning"], "scenes/collab.rs", `ui.presence_pile("rfc_peers", &peers, 4, &mut s.follow_peer);\nui.markdown("rfc_md", SPEC_DOC);\nui.reaction_bar("rfc_rx", &mut s.reactions);\nui.comment_thread("rfc_thread", "Inline review · paint.rs", &mut s.comments, &mut s.resolved);\nui.checklist_card("rfc_gates", "Sign-off checklist", &mut s.steps);`],
  ["scene-governance", "Cloud governance & FinOps", "Fiscal quarter and currency-switchable budget controls paired with an interactive RBAC permission matrix, cluster spend treemap, IAM scope shuttle list, and 100-cell compliance waffle chart.", ["Data", "Charts", "Inputs", "Selection"], "scenes/governance.rs", `ui.quarter_picker("gov_fy", &mut s.fy_year, &mut s.fy_quarter);\nui.permission_matrix("gov_rbac", &roles, &caps, &mut s.rbac_grid);\nui.treemap("gov_spend", &items).unit("$k").height(176.0).show();\nui.transfer_list("gov_scopes", &mut s.avail_scopes, &mut s.assigned_scopes);\nui.waffle_chart("gov_compliance", "Fleet Zero-Trust Compliance", &slices);`],
  ["scene-schedule", "Global on-call & launch room", "Four-timezone world clocks, interactive week date strip, time-blocked engineering rotation calendar, critical-path launch Gantt chart, and live handoff inbox.", ["Dates", "Planning", "Feedback"], "scenes/schedule.rs", `ui.world_clocks("ops_clocks", &clocks);\nui.date_strip("ops_strip", &days, &mut s.strip_day);\nui.day_schedule("ops_cal", &cal_days, 9, 17, Some((2, 13.3)), &events, 236.0);\nui.gantt_chart("ops_gantt", &weeks, &tasks);\nui.notification_list("ops_inbox", &mut s.notifs);`],
  ["hero", "Northwind analytics dashboard", "A collapsible workspace sidebar, segmented time range, date picker, KPI stat cards with sparklines, smooth revenue area chart, and traffic breakdown donut.", ["Navigation", "Charts", "Dates"], "main.rs", ""],
  ["looks", "Four native platform looks", "The exact same controls morphing live across Lumen Ethereal, macOS Sonoma, Windows 11 Fluent, and GNOME Adwaita with perceptual OKLab token blending.", ["Looks", "Motion"], "main.rs", ""],
];
function showcase() {
  return docsShell(guideNav(), `
    <p class="crumb">Showcase</p>
    <h1>${SHOWS.length} compositions, running live.</h1>
    <p class="lead">Multi-panel workspaces composed from lumen’s ${count} widgets and compiled to WebAssembly. Interact with any scene, toggle code, switch OS looks, or open full window.</p>
    <div class="show">${SHOWS.map(([d, n, p, tags, srcFile, code]) => `
      <section class="show-item">
        <div class="show-meta"><h2>${n}</h2>${tags.map((t) => `<span class="chip">${t}</span>`).join("")}<p>${p}</p></div>
        ${stage(d, { title: n, code })}
        <div class="show-meta"><span class="chip acc">Drawn by lumen</span><span class="chip">macOS · Windows · Linux · Web</span><span class="end"><a class="btn sm" href="${SRC_DIR}/${srcFile}" target="_blank" rel="noopener">View source</a></span></div>
      </section>`).join("")}</div>`, true);
}

const APPS = [
  ["app-mail", "Postbox", "Mail", "Folders, a searchable message list with unread dots and stars, a reading pane with a quick reply, and a compose dialog.", "apps/mail.rs"],
  ["app-music", "Cadence", "Music", "A library you can search and star, a now-playing panel with a scrubbable waveform, transport, volume and an 8-band equalizer.", "apps/music.rs"],
  ["app-tracker", "Relay", "Issue tracker", "Issues as a filterable list or a board, a detail sheet with status, priority and activity, and a new-issue dialog.", "apps/tracker.rs"],
  ["app-notes", "Margin", "Notes", "Markdown notes with write, preview and split modes, pinned notes, tags and a live word count.", "apps/notes.rs"],
  ["app-assistant", "Lumi", "AI assistant", "Chat history, replies that stream in word by word, a composer with model choice and attachments, and starter prompts.", "apps/assistant.rs"],
  ["app-finance", "Tally", "Finance", "Balance and spending stats, a cash-flow chart, budgets that fill as you spend, transactions and an add dialog.", "apps/finance.rs"],
  ["app-todo", "Today", "To-do", "The tutorial app: add tasks, tick them off, filter and clear, in about 70 lines.", "apps/todo.rs"],
  ["app-settings", "Preferences", "Settings", "A searchable sidebar of panes. Appearance changes the whole window live: theme, look, accent, motion and text size. Do Not Disturb runs on two time wheels.", "apps/settings.rs"],
];
function apps() {
  return docsShell(guideNav(), `
    <p class="crumb">Apps</p>
    <h1>${APPS.length + 1} apps built with lumen.</h1>
    <p class="lead">Complete windows, not snippets. Wardlight ships on its own; the other ${APPS.length} run live here and natively from the same source.</p>
    <h2 id="wardlight">Wardlight</h2>
    <p>A password and SSH manager, and the first lumen app that ships on its own (<a href="${WARDLIGHT}/releases/latest" target="_blank" rel="noopener">download</a>, <a href="${WARDLIGHT}" target="_blank" rel="noopener">source</a>). It's an early preview: everything works, but it hasn't had a security review yet.</p>
    <figure class="wl-fig"><img loading="lazy" alt="Wardlight's Passwords screen" data-l="img/wardlight-passwords-light.png" data-d="img/wardlight-passwords-dark.png" src="img/wardlight-passwords-${isDark() ? "dark" : "light"}.png"></figure>
    <h3>Built with GPUI and lumen</h3>
    <p>Wardlight is a <a href="https://gpui.rs" target="_blank" rel="noopener">GPUI</a> app. GPUI owns the window, the custom title bar, menus, keyboard shortcuts and input methods. Every screen inside is a lumen view hosted through <a href="#/guide/gpui">lumen-gpui</a>, so the lists, detail panes, dialogs, the ⌘K palette and the one-time-code ring are the same widgets you see on this site, with the same looks and motion. The two share state both ways: GPUI actions update the lumen screens, and lumen buttons trigger GPUI commands.</p>
    <h3>What it does</h3>
    <ul>
      <li><b>Passwords</b> with folders, tags, favorites, history, a strength meter and a reused-password warning, and one-time codes (TOTP) next to them.</li>
      <li><b>SSH keys</b>: generate ed25519 or RSA 4096, import existing OpenSSH keys, copy the public key, export encrypted.</li>
      <li><b>SSH hosts</b>: aliases, jump hosts and options linked to a vault key; import <code>~/.ssh/config</code> and write hosts back, previewed as a diff.</li>
      <li><b>Backups</b>: scheduled encrypted backups, exports, and imports from Bitwarden or CSV with a review before anything is written.</li>
      <li><b>Careful with secrets</b>: the clipboard clears after 30 seconds, the vault locks when idle or asleep, and Touch ID, Windows Hello or fprintd can unlock it.</li>
    </ul>
    <div class="wl-pair">
      <figure class="wl-fig"><img loading="lazy" alt="SSH Keys: keys with fingerprints, and one key's public key and the hosts that use it" data-l="img/wardlight-ssh-keys-light.png" data-d="img/wardlight-ssh-keys-dark.png" src="img/wardlight-ssh-keys-${isDark() ? "dark" : "light"}.png"><figcaption>SSH keys</figcaption></figure>
      <figure class="wl-fig"><img loading="lazy" alt="SSH Hosts: hosts marked new or edited, and one host's details with its ssh command" data-l="img/wardlight-ssh-hosts-light.png" data-d="img/wardlight-ssh-hosts-dark.png" src="img/wardlight-ssh-hosts-${isDark() ? "dark" : "light"}.png"><figcaption>SSH hosts</figcaption></figure>
    </div>
    <h2>Demo apps</h2>
    <div class="apps">${APPS.map(([d, n, tag, p, src]) => `
      <article class="app glass glow">
        <div class="thumb"><img loading="lazy" alt="" data-l="${poster(d, false)}" data-d="${poster(d, true)}" src="${poster(d, isDark())}"></div>
        <div class="t">
          <div class="row"><h3>${n}</h3><span class="chip ${tag === "In progress" ? "acc" : ""}">${tag}</span></div>
          <p>${p}</p>
          <div class="row"><span class="chip">macOS</span><span class="chip">Windows</span><span class="chip">Linux</span><span class="chip">Web</span></div>
          <div class="acts"><a class="btn sm main" href="${demoSrc(d)}" target="_blank" rel="noopener">Open live ↗</a><a class="btn sm" href="${SRC_DIR}/${src}" target="_blank" rel="noopener">View source</a></div>
        </div>
      </article>`).join("")}</div>
    <h2>Run them natively</h2>
    ${pre("cargo run -p lumen-demos -- app-mail          # or app-music, app-tracker, app-notes,\n                                              #    app-assistant, app-finance, app-todo\ncargo run -p lumen-demos -- app-notes --look mac", "sh")}
`, true);
}

function changelog() {
  return docsShell(guideNav(), `<p class="crumb">Changelog</p><h1>What changed.</h1><p class="lead">Newest first.</p>
    ${CHANGELOG.map((r) => `<div class="release"><h2>${r.v}</h2><time datetime="${r.date}">${r.date}</time><ul>${r.items.map((i) => `<li>${esc(i)}</li>`).join("")}</ul></div>`).join("")}`);
}
function notFound() {
  return `<div class="wrap" style="padding:160px 16px"><h1 style="font:600 56px var(--font-display);letter-spacing:-0.04em;margin:0">Nothing here.</h1><p style="color:var(--text-2)">That page doesn't exist. <a class="more" href="#/">Go to the overview <span class="arr">→</span></a></p></div>`;
}

// ------------------------------------------------------------------ router
function route() {
  const h = location.hash.replace(/^#\/?/, "");
  const [a, b] = h.split("/");
  $$(".live").forEach((l) => io.unobserve(l));
  let html, title = "lumen: Rust UI, lit from within", wire;
  if (!a) { html = home(); wire = wireHome; }
  else if (a === "guide") { html = guide(b); title = `${(GUIDES.find((g) => g.slug === b) || GUIDES[0]).title} · lumen`; }
  else if (a === "components" && b) { html = component(b); title = `${(COMPONENTS.find((c) => c.slug === b) || {}).name || "Component"} · lumen`; }
  else if (a === "components") { html = components(); title = "Components · lumen"; wire = wireComponents; }
  else if (a === "showcase") { html = showcase(); title = "Showcase · lumen"; }
  else if (a === "apps") { html = apps(); title = "Apps · lumen"; }
  else if (a === "changelog") { html = changelog(); title = "Changelog · lumen"; }
  else html = notFound();
  document.title = title;
  document.body.classList.toggle("docs-page", !!a);
  view.innerHTML = html;
  decorate(view);
  $$(".stage", view).forEach(wireStage);
  activate(view);
  wire?.();
  $$("[data-copy]", view).forEach((btn) => btn.addEventListener("click", async () => {
    try { await navigator.clipboard.writeText(btn.dataset.copy); btn.dataset.done = 1; btn.textContent = "Copied"; setTimeout(() => { delete btn.dataset.done; btn.textContent = "Copy"; }, 1400); } catch {}
  }));
  const rio = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { e.target.classList.add("in"); rio.unobserve(e.target); } }), { threshold: 0.08 });
  $$(".reveal", view).forEach((el) => rio.observe(el));
  const here = "#/" + h;
  $$(".side a, .links a").forEach((l) => {
    const href = l.getAttribute("href");
    const seg = href.replace(/^#\/?/, "").split("/")[0];
    const on = l.closest(".links") ? seg === (a || "") : href === here;
    on ? l.setAttribute("aria-current", "page") : l.removeAttribute("aria-current");
  });
  sideIndicator();
  $(".side a[aria-current]")?.scrollIntoView({ block: "nearest" });
  $("#links").classList.remove("open");
  scrollTo(0, 0);
  view.focus({ preventScroll: true });
}

// The sidebar re-renders on every route, so the pill starts where the last
// page left it (same nav only) and springs to the new item.
let lastInd = null;
function sideIndicator() {
  const side = $("#side"), cur = side && $("a[aria-current]", side);
  if (!side || !cur) { lastInd = null; return; }
  const ind = document.createElement("span");
  ind.className = "ind"; ind.setAttribute("aria-hidden", "true");
  side.prepend(ind); side.classList.add("has-ind");
  const nav = $("h4", side)?.textContent, to = { nav, y: cur.offsetTop, h: cur.offsetHeight };
  const place = (p) => { ind.style.transform = `translateY(${p.y}px)`; ind.style.height = p.h + "px"; ind.style.opacity = 1; };
  if (lastInd && lastInd.nav === nav && lastInd.y !== to.y) {
    ind.style.transition = "none"; place(lastInd); ind.offsetWidth; ind.style.transition = ""; place(to);
  } else place(to);
  lastInd = to;
}

// ----------------------------------------------------------------- palette
const pal = $("#pal"), palIn = $("#pal input"), palList = $("#pal ul");
const ENTRIES = [
  ...GUIDES.map((g) => ({ t: g.title, k: "Guide", h: `#/guide/${g.slug}` })),
  ...COMPONENTS.map((c) => ({ t: c.name, k: c.group, h: `#/components/${c.slug}` })),
  { t: "Showcase", k: "Page", h: "#/showcase" }, { t: "Apps", k: "Page", h: "#/apps" }, { t: "Changelog", k: "Page", h: "#/changelog" },
];
let sel = 0, results = ENTRIES, lastFocus = null;
function renderPal() {
  const q = palIn.value.trim().toLowerCase();
  results = q ? ENTRIES.filter((e) => (e.t + " " + e.k).toLowerCase().includes(q)) : ENTRIES;
  sel = Math.min(sel, Math.max(0, results.length - 1));
  palList.innerHTML = results.length
    ? results.map((e, i) => `<li role="option" aria-selected="${i === sel}"><a href="${e.h}">${esc(e.t)}<small>${esc(e.k)}</small></a></li>`).join("")
    : `<li class="empty">No page matches “${esc(palIn.value)}”.</li>`;
  palList.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
}
function openPal() { lastFocus = document.activeElement; pal.hidden = false; $(".page-root").inert = true; palIn.value = ""; sel = 0; renderPal(); palIn.focus(); }
function closePal() { pal.hidden = true; $(".page-root").inert = false; lastFocus?.focus?.(); }
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
  else if (e.key === "/" && pal.hidden && !/input|textarea/i.test(document.activeElement.tagName)) {
    e.preventDefault();
    const f = $(".filter-in input"); f ? f.focus() : openPal();
  }
});

// --------------------------------------------------------------------- sky
// An ordered-dither aurora: three drifting lights plus one that follows the
// pointer, sampled at a quarter resolution and thresholded with a 4×4 Bayer
// matrix. Static when motion is reduced.
function sky() {
  const cv = $("#sky"), cx = cv.getContext("2d");
  const B = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5].map((v) => (v + 0.5) / 16);
  const CELL = 4; let W, H, img, t = 0, last = performance.now(), px = 0.5, py = 0.3, tx = 0.5, ty = 0.3, drawn = false;
  const size = () => { W = Math.ceil(innerWidth / CELL); H = Math.ceil(innerHeight / CELL); cv.width = W; cv.height = H; img = cx.createImageData(W, H); drawn = false; };
  size(); addEventListener("resize", size);
  addEventListener("pointermove", (e) => { tx = e.clientX / innerWidth; ty = e.clientY / innerHeight; });
  const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
  let lastKey = "";
  function frame(now) {
    requestAnimationFrame(frame);
    const m = motion();
    const key = accentHex() + isDark();
    if (!m && drawn && key === lastKey) return;
    if (document.hidden) return;
    lastKey = key;
    const dt = Math.min(64, now - last); last = now; t += dt * 0.00018 * m;
    px += (tx - px) * 0.04 * (m ? 1 : 0); py += (ty - py) * 0.04 * (m ? 1 : 0);
    const dark = isDark(), acc = hex(accentHex()), ink = dark ? [235, 240, 255] : [30, 40, 80];
    const d = img.data, asp = W / H;
    const s = [[0.25 + 0.15 * Math.sin(t * 1.3), 0.35 + 0.12 * Math.cos(t), 0.34], [0.7 + 0.18 * Math.cos(t * 0.8), 0.25 + 0.1 * Math.sin(t * 1.7), 0.3], [0.5 + 0.3 * Math.sin(t * 0.5), 0.8, 0.42]];
    for (let y = 0; y < H; y++) {
      const ny = y / H;
      for (let x = 0; x < W; x++) {
        const nx = x / W;
        const band = Math.exp(-(((ny - 0.32 - 0.08 * Math.sin(nx * 5 + t * 3) - 0.05 * Math.sin(nx * 11 - t * 2)) * 7) ** 2));
        let v = band * 0.55;
        for (const [sx, sy, r] of s) { const dx = (nx - sx) * asp, dy = ny - sy; v += 0.32 * Math.exp(-(dx * dx + dy * dy) / (r * r * 0.18)); }
        const dx = (nx - px) * asp, dy = ny - py; v += 0.45 * Math.exp(-(dx * dx + dy * dy) / 0.012);
        v *= 1 - ny * 0.55;
        const i = (y * W + x) * 4;
        if (v > B[(y & 3) * 4 + (x & 3)]) {
          const mix = Math.min(1, band * 1.4 + 0.15);
          d[i] = acc[0] * mix + ink[0] * (1 - mix); d[i + 1] = acc[1] * mix + ink[1] * (1 - mix); d[i + 2] = acc[2] * mix + ink[2] * (1 - mix);
          d[i + 3] = (dark ? 120 : 80) * Math.min(1, v);
        } else d[i + 3] = 0;
      }
    }
    cx.putImageData(img, 0, 0);
    drawn = true;
  }
  requestAnimationFrame(frame);
}

// ---------------------------------------------------------------- boot
if ("scrollRestoration" in history) history.scrollRestoration = "manual";
$("#gh").innerHTML = I.github;
$("#burger").innerHTML = I.menu;
$("#search").addEventListener("click", openPal);
$("#theme").addEventListener("click", () => set("theme", isDark() ? "light" : "dark"));
$("#burger").addEventListener("click", () => { $("#links").classList.toggle("open"); $("#side")?.classList.toggle("open"); });
sysDark.addEventListener("change", () => S.theme === "system" && apply());
sysReduce.addEventListener("change", apply);
// Card glow follows the pointer.
addEventListener("pointermove", (e) => $$(".glow").forEach((c) => {
  const r = c.getBoundingClientRect();
  if (e.clientY < r.top - 240 || e.clientY > r.bottom + 240) return;
  c.style.setProperty("--mx", e.clientX - r.left + "px"); c.style.setProperty("--my", e.clientY - r.top + "px");
}), { passive: true });
buildPanel();
addEventListener("hashchange", route);
route();
apply();
sky();
