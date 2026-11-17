// Shared by the four concept pages: the settings panel, its state, and a
// bento of interactive stand-ins for lumen widgets. Each page styles them.

const ACCENTS = [
  ["graphite", "#8e8e93"], ["violet", "#8b5cf6"], ["blue", "#3b82f6"], ["green", "#22b07d"],
  ["amber", "#f2b01e"], ["orange", "#f97316"], ["rose", "#f43f5e"], ["pink", "#e66ac0"],
];
const LOOKS = { lumen: 10, macos: 7, windows: 4, gnome: 12 };
const KEY = "lumen-concept-settings";
const DEFAULTS = { accent: "blue", custom: "#3b82f6", theme: "system", look: "lumen", speed: 1, density: "regular", reduced: false, split: false };

let state = { ...DEFAULTS };
try { Object.assign(state, JSON.parse(localStorage.getItem(KEY) || "{}")); } catch (e) {}
const mq = matchMedia("(prefers-color-scheme: dark)");
const rmq = matchMedia("(prefers-reduced-motion: reduce)");

export const settings = () => state;
export const motionScale = () => (state.reduced || rmq.matches ? 0 : state.speed);

function accentHex() {
  if (state.accent === "custom") return state.custom;
  return (ACCENTS.find(([n]) => n === state.accent) || ACCENTS[2])[1];
}

function apply() {
  const root = document.documentElement;
  const dark = state.theme === "dark" || (state.theme === "system" && mq.matches);
  root.classList.add("no-anim");
  root.classList.toggle("t-dark", dark);
  root.classList.toggle("t-light", !dark);
  root.style.setProperty("--accent", accentHex());
  root.style.setProperty("--r", LOOKS[state.look] + "px");
  root.style.setProperty("--m", motionScale() ? 1 / state.speed : 0);
  root.dataset.look = state.look;
  root.dataset.density = state.density;
  root.dataset.reduced = String(state.reduced || rmq.matches);
  root.dataset.split = String(state.split);
  requestAnimationFrame(() => requestAnimationFrame(() => root.classList.remove("no-anim")));
  try { localStorage.setItem(KEY, JSON.stringify(state)); } catch (e) {}
  syncPanel();
  document.dispatchEvent(new CustomEvent("lumen:settings", { detail: state }));
}

function set(k, v) { state[k] = v; apply(); }

// ---------- panel ----------

let panel;
function seg(key, opts) {
  return `<div class="sp-seg" role="radiogroup" data-key="${key}">${opts.map(([v, l]) =>
    `<button type="button" role="radio" data-v="${v}">${l}</button>`).join("")}<span class="sp-thumb" aria-hidden="true"></span></div>`;
}
function toggle(key, label) {
  return `<label class="sp-row"><span>${label}</span><button type="button" class="sp-switch" role="switch" data-key="${key}" aria-label="${label}"><i></i></button></label>`;
}

function buildPanel(btn) {
  panel = document.createElement("div");
  panel.className = "sp";
  panel.id = "settings-panel";
  panel.hidden = true;
  panel.setAttribute("role", "dialog");
  panel.setAttribute("aria-label", "Appearance");
  panel.innerHTML = `
    <div class="sp-row sp-accent"><div><span>Accent</span><small>Pick one, or mix your own</small></div>
      <div class="sp-dots">${ACCENTS.map(([n, c]) => `<button type="button" class="sp-dot" data-accent="${n}" style="--c:${c}" aria-label="${n} accent"></button>`).join("")}
        <label class="sp-dot sp-custom" aria-label="Custom accent"><input type="color" value="${state.custom}"></label></div></div>
    <div class="sp-row"><span>Theme</span>${seg("theme", [["light", "Light"], ["dark", "Dark"], ["system", "System"]])}</div>
    <div class="sp-row"><span>Look</span>${seg("look", [["lumen", "Lumen"], ["macos", "macOS"], ["windows", "Windows"], ["gnome", "GNOME"]])}</div>
    <div class="sp-row"><span>Density</span>${seg("density", [["compact", "Compact"], ["regular", "Regular"], ["roomy", "Roomy"]])}</div>
    <div class="sp-row"><span>Motion speed</span>${seg("speed", [["1", "1×"], ["0.5", "½×"], ["0.25", "¼×"], ["0.1", "⅒×"]])}</div>
    ${toggle("reduced", "Reduced motion")}
    ${toggle("split", "Light and dark side by side")}
    <button type="button" class="sp-reset">Reset to defaults</button>`;
  document.body.append(panel);

  panel.addEventListener("click", (e) => {
    const t = e.target.closest("button");
    if (!t) return;
    if (t.dataset.accent) set("accent", t.dataset.accent);
    else if (t.parentElement.classList.contains("sp-seg")) {
      const k = t.parentElement.dataset.key;
      set(k, k === "speed" ? Number(t.dataset.v) : t.dataset.v);
    } else if (t.classList.contains("sp-switch")) set(t.dataset.key, !state[t.dataset.key]);
    else if (t.classList.contains("sp-reset")) { state = { ...DEFAULTS }; apply(); }
  });
  panel.querySelector("input[type=color]").addEventListener("input", (e) => { state.custom = e.target.value; set("accent", "custom"); });

  const place = () => {
    const r = btn.getBoundingClientRect();
    panel.style.top = r.bottom + 10 + "px";
    panel.style.right = Math.max(16, innerWidth - r.right) + "px";
  };
  const open = (v) => {
    if (v) { place(); panel.hidden = false; requestAnimationFrame(() => panel.classList.add("open")); panel.querySelector("button").focus({ preventScroll: true }); }
    else { panel.classList.remove("open"); setTimeout(() => { if (!panel.classList.contains("open")) panel.hidden = true; }, 220 * (state.speed ? 1 / state.speed : 1)); }
    btn.setAttribute("aria-expanded", String(v));
  };
  btn.setAttribute("aria-controls", "settings-panel");
  btn.setAttribute("aria-expanded", "false");
  btn.addEventListener("click", () => open(panel.hidden));
  document.addEventListener("pointerdown", (e) => { if (!panel.hidden && !panel.contains(e.target) && !btn.contains(e.target)) open(false); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !panel.hidden) { open(false); btn.focus(); } });
  addEventListener("resize", () => !panel.hidden && place());
}

function syncSeg(el) {
  const k = el.dataset.key;
  let active = null;
  el.querySelectorAll("button").forEach((b) => {
    const on = String(state[k]) === b.dataset.v;
    b.setAttribute("aria-checked", String(on));
    if (on) active = b;
  });
  const th = el.querySelector(".sp-thumb, .lm-thumb");
  if (active && th) { th.style.width = active.offsetWidth + "px"; th.style.transform = `translateX(${active.offsetLeft}px)`; }
}

function syncPanel() {
  if (!panel) return;
  panel.querySelectorAll(".sp-dot").forEach((d) => d.setAttribute("aria-pressed", String((d.dataset.accent || "custom") === state.accent)));
  panel.querySelector(".sp-custom").style.setProperty("--c", state.custom);
  panel.querySelectorAll(".sp-seg").forEach(syncSeg);
  panel.querySelectorAll(".sp-switch").forEach((s) => s.setAttribute("aria-checked", String(!!state[s.dataset.key])));
}

// ---------- bento of widget stand-ins ----------

const DAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
function calendar() {
  // November 2026 starts on a Sunday.
  let cells = "";
  for (let i = 0; i < 6; i++) cells += `<span class="lm-day out">${26 + i}</span>`;
  for (let d = 1; d <= 30; d++) cells += `<button type="button" class="lm-day${d === 17 ? " today" : ""}${d >= 18 && d <= 22 ? " in" : ""}${d === 18 || d === 22 ? " edge" : ""}" data-d="${d}">${d}</button>`;
  for (let i = 1; i <= 6; i++) cells += `<span class="lm-day out">${i}</span>`;
  return `<div class="lm-cal-head"><b>November 2026</b><span><button type="button" class="lm-icon" aria-label="Previous month">‹</button><button type="button" class="lm-icon" aria-label="Next month">›</button></span></div>
    <div class="lm-cal">${DAYS.map((d) => `<span class="lm-dow">${d}</span>`).join("")}${cells}</div>`;
}

function spark(points, w = 220, h = 56) {
  const max = Math.max(...points), min = Math.min(...points);
  const xy = points.map((p, i) => [(i / (points.length - 1)) * w, h - 6 - ((p - min) / (max - min || 1)) * (h - 12)]);
  let d = `M${xy[0][0]},${xy[0][1]}`;
  for (let i = 1; i < xy.length; i++) {
    const [x0, y0] = xy[i - 1], [x1, y1] = xy[i], cx = (x0 + x1) / 2;
    d += ` C${cx},${y0} ${cx},${y1} ${x1},${y1}`;
  }
  const [lx, ly] = xy[xy.length - 1];
  return `<svg class="lm-spark" viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" aria-hidden="true">
    <path class="area" d="${d} L${w},${h} L0,${h}Z"/><path class="line" d="${d}" pathLength="1"/><circle cx="${lx}" cy="${ly}" r="3.5"/></svg>`;
}

function donut() {
  const parts = [[42, "var(--accent)", "Search"], [26, "var(--c-series-2)", "Direct"], [17, "var(--c-series-3)", "Social"], [15, "var(--c-series-4)", "Other"]];
  let off = 0;
  const rings = parts.map(([v, c]) => { const s = `<circle r="15.9" cx="20" cy="20" pathLength="100" stroke="${c}" stroke-dasharray="${v - 1.2} ${100 - v + 1.2}" stroke-dashoffset="${-off}"/>`; off += v; return s; }).join("");
  return `<div class="lm-donut"><svg viewBox="0 0 40 40" aria-hidden="true">${rings}</svg><div><b>12.4k</b><small>visits</small></div></div>
    <ul class="lm-legend">${parts.map(([v, c, n]) => `<li><i style="background:${c}"></i>${n}<span>${v}%</span></li>`).join("")}</ul>`;
}

// A custom select: a trigger button plus a listbox popover. Never the OS menu.
function dropdown(id, value, opts) {
  return `<div class="lm-dd" data-dd>
    <button type="button" class="lm-dd-btn" aria-haspopup="listbox" aria-expanded="false" aria-labelledby="${id}-l ${id}-v"><span id="${id}-v">${value}</span>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4.5 6.5 8 10l3.5-3.5"/></svg></button>
    <ul class="lm-dd-menu" role="listbox" tabindex="-1" aria-labelledby="${id}-l">${opts.map(([v, hint]) =>
      `<li role="option" data-v="${v}" aria-selected="${v === value}"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3.5 8.5 3 3 6-7"/></svg><span>${v}</span><small>${hint}</small></li>`).join("")}</ul></div>`;
}

function wireDropdown(dd) {
  const btn = dd.querySelector(".lm-dd-btn"), menu = dd.querySelector(".lm-dd-menu"), items = [...menu.children];
  let active = 0;
  const mark = (i) => { active = (i + items.length) % items.length; items.forEach((x, k) => x.classList.toggle("active", k === active)); };
  const open = (v) => {
    dd.classList.toggle("open", v); btn.setAttribute("aria-expanded", String(v));
    if (v) { mark(items.findIndex((x) => x.getAttribute("aria-selected") === "true")); menu.focus({ preventScroll: true }); }
  };
  const pick = (li) => {
    items.forEach((x) => x.setAttribute("aria-selected", String(x === li)));
    const v = btn.querySelector("span"); v.textContent = li.dataset.v; v.classList.remove("roll"); void v.offsetWidth; v.classList.add("roll");
    open(false); btn.focus();
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

export function bentoHTML() {
  return `
  <article class="lm-card lm-form" data-tag="Inputs">
    <h3>Add a runner</h3><p class="lm-sub">Runners pick up jobs for the platforms you choose.</p>
    <div class="lm-segmented" role="radiogroup" aria-label="Platform" data-key="os">
      <button type="button" role="radio" aria-checked="true">Linux</button><button type="button" role="radio" aria-checked="false">macOS</button><button type="button" role="radio" aria-checked="false">Windows</button><span class="lm-thumb" aria-hidden="true"></span></div>
    <label class="lm-field"><span>Name</span><input value="linux-arm64-02" spellcheck="false"></label>
    <div class="lm-field"><span id="rg-l">Region</span>${dropdown("rg", "Frankfurt", [["Frankfurt", "eu-central · 12 ms"], ["Virginia", "us-east · 88 ms"], ["Oregon", "us-west · 140 ms"], ["Tokyo", "ap-northeast · 210 ms"]])}</div>
    <div class="lm-field"><span>Labels</span><div class="lm-chips" role="group" aria-label="Labels">${["arm64", "gpu", "docker", "nightly", "macos-14"].map((c, i) =>
      `<button type="button" class="lm-chip" aria-pressed="${i < 2}">${c}</button>`).join("")}</div></div>
    <div class="lm-field lm-inline"><span>Concurrent jobs</span><div class="lm-stepper"><button type="button" aria-label="Fewer jobs">−</button><output>4</output><button type="button" aria-label="More jobs">+</button></div></div>
    <div class="lm-actions"><button type="button" class="lm-btn ghost">Cancel</button><button type="button" class="lm-btn primary" data-toast>Add runner</button></div>
  </article>
  <article class="lm-card lm-stat" data-tag="Charts">
    <p class="lm-sub">Median build</p><div class="lm-num"><b data-count="222">3m 42s</b><em class="down">−17.6%</em></div>
    ${spark([9, 8.6, 8.9, 7.8, 8, 7.1, 7.3, 6.4, 6.6, 5.8, 5.1])}
  </article>
  <article class="lm-card lm-stat" data-tag="Charts">
    <p class="lm-sub">Active users</p><div class="lm-num"><b>8,214</b><em class="up">+4.1%</em></div>
    ${spark([3, 3.4, 3.2, 4.1, 4, 4.8, 5.2, 5, 6.1, 6.4, 7.2])}
  </article>
  <article class="lm-card lm-toggles" data-tag="Selection">
    <h3>Quick settings</h3>
    <div class="lm-trow"><span>Wi-Fi</span><button type="button" class="lm-switch" role="switch" aria-checked="true" aria-label="Wi-Fi"><i></i></button></div>
    <div class="lm-trow"><span>Bluetooth</span><button type="button" class="lm-switch" role="switch" aria-checked="false" aria-label="Bluetooth"><i></i></button></div>
    <div class="lm-trow"><span>Focus</span><button type="button" class="lm-switch" role="switch" aria-checked="true" aria-label="Focus"><i></i></button></div>
    <label class="lm-slider"><span>Volume <output>64%</output></span><input type="range" min="0" max="100" value="64" style="--v:64%"></label>
  </article>
  <article class="lm-card lm-calendar" data-tag="Dates">${calendar()}
    <div class="lm-agenda"><b>Release 2.4</b><small>Pick a window while the freeze holds.</small>
      <div class="lm-slots" role="radiogroup" aria-label="Release window">${["09:00", "11:30", "14:00", "16:30"].map((t, i) =>
        `<button type="button" role="radio" class="lm-slot" aria-checked="${i === 1}">${t}</button>`).join("")}</div></div></article>
  <article class="lm-card lm-traffic" data-tag="Charts"><h3>Traffic</h3>${donut()}</article>
  <article class="lm-card lm-people" data-tag="Layout">
    <h3>Reviewers</h3>
    ${[["JB", "Jonas Berg", "Required"], ["PN", "Priya Nair", "Required"], ["TR", "Tomas Rivera", "Optional"]].map(([i, n, r], k) =>
      `<div class="lm-person"><span class="lm-avatar a${k}">${i}</span><div><b>${n}</b><small>${n.split(" ")[0].toLowerCase()}@northlight.io</small></div><span class="lm-badge">${r}</span></div>`).join("")}
    <div class="lm-progress" role="progressbar" aria-label="Review progress" aria-valuenow="66"><i style="--p:66%"></i></div>
  </article>
  <div class="lm-toast" role="status" aria-live="polite"></div>`;
}

function wireBento(root) {
  root.querySelectorAll(".lm-segmented").forEach((s) => {
    const sync = () => {
      const b = s.querySelector('[aria-checked="true"]'), th = s.querySelector(".lm-thumb");
      if (b && th) { th.style.width = b.offsetWidth + "px"; th.style.transform = `translateX(${b.offsetLeft}px)`; }
    };
    s.addEventListener("click", (e) => {
      const b = e.target.closest("button"); if (!b) return;
      s.querySelectorAll("button").forEach((x) => x.setAttribute("aria-checked", String(x === b)));
      sync();
    });
    requestAnimationFrame(sync);
    document.addEventListener("lumen:settings", () => requestAnimationFrame(sync));
    addEventListener("resize", sync);
  });
  root.querySelectorAll(".lm-switch").forEach((s) => s.addEventListener("click", () => s.setAttribute("aria-checked", String(s.getAttribute("aria-checked") !== "true"))));
  root.querySelectorAll(".lm-slider input").forEach((r) => r.addEventListener("input", () => {
    r.style.setProperty("--v", r.value + "%");
    r.closest(".lm-slider").querySelector("output").textContent = r.value + "%";
  }));
  root.querySelectorAll("[data-dd]").forEach(wireDropdown);
  root.querySelectorAll(".lm-chip").forEach((c) => c.addEventListener("click", () => c.setAttribute("aria-pressed", String(c.getAttribute("aria-pressed") !== "true"))));
  root.querySelectorAll(".lm-stepper").forEach((st) => st.addEventListener("click", (e) => {
    const b = e.target.closest("button"); if (!b) return;
    const o = st.querySelector("output"), v = Math.min(16, Math.max(1, +o.textContent + (b === st.firstElementChild ? -1 : 1)));
    o.textContent = v; o.classList.remove("roll"); void o.offsetWidth; o.classList.add("roll");
  }));
  root.querySelectorAll(".lm-slots").forEach((g) => g.addEventListener("click", (e) => {
    const b = e.target.closest("button"); if (!b) return;
    g.querySelectorAll("button").forEach((x) => x.setAttribute("aria-checked", String(x === b)));
  }));
  let start = null;
  root.querySelectorAll(".lm-day[data-d]").forEach((d) => d.addEventListener("click", () => {
    const n = +d.dataset.d;
    if (start === null) { start = n; root.querySelectorAll(".lm-day").forEach((x) => x.classList.remove("in", "edge")); d.classList.add("edge", "in"); }
    else {
      const [a, b] = [Math.min(start, n), Math.max(start, n)];
      root.querySelectorAll(".lm-day[data-d]").forEach((x) => { const v = +x.dataset.d; x.classList.toggle("in", v >= a && v <= b); x.classList.toggle("edge", v === a || v === b); });
      start = null;
    }
  }));
  const toast = root.querySelector(".lm-toast");
  root.querySelectorAll("[data-toast]").forEach((b) => b.addEventListener("click", () => {
    const name = root.querySelector(".lm-form input")?.value || "runner";
    toast.innerHTML = `<b>Runner added</b><span>${name.replace(/[<&]/g, "")} is online and taking jobs.</span>`;
    toast.classList.remove("show"); void toast.offsetWidth; toast.classList.add("show");
    clearTimeout(toast._t); toast._t = setTimeout(() => toast.classList.remove("show"), 2800 * (motionScale() ? 1 / state.speed : 1));
  }));
  // Draw sparklines in when they scroll into view.
  const io = new IntersectionObserver((es) => es.forEach((e) => e.isIntersecting && e.target.classList.add("drawn")), { threshold: 0.4 });
  root.querySelectorAll(".lm-spark, .lm-donut, .lm-progress").forEach((s) => io.observe(s));
}

// Renders the bento into every [data-bento]; with "side by side" on, renders a
// forced-light and a forced-dark copy.
export function mountBento() {
  document.querySelectorAll("[data-bento]").forEach((host) => {
    const render = () => {
      if (state.split) host.innerHTML = `<div class="split"><div class="bento t-light">${bentoHTML()}</div><div class="bento t-dark">${bentoHTML()}</div></div>`;
      else host.innerHTML = `<div class="bento">${bentoHTML()}</div>`;
      host.querySelectorAll(".bento").forEach(wireBento);
      host._split = state.split;
    };
    render();
    document.addEventListener("lumen:settings", () => host._split !== state.split && render());
  });
}

export function init() {
  const btn = document.getElementById("settings");
  if (btn) buildPanel(btn);
  mq.addEventListener("change", () => state.theme === "system" && apply());
  rmq.addEventListener("change", apply);
  const themeBtn = document.getElementById("theme");
  themeBtn?.addEventListener("click", () => set("theme", document.documentElement.classList.contains("t-dark") ? "light" : "dark"));
  apply();
  mountBento();
  // Copy buttons.
  document.querySelectorAll("[data-copy]").forEach((b) => b.addEventListener("click", async () => {
    try { await navigator.clipboard.writeText(b.dataset.copy); const t = b.textContent; b.dataset.done = "1"; b.textContent = "Copied"; setTimeout(() => { delete b.dataset.done; b.textContent = t; }, 1400); } catch (e) {}
  }));
}
