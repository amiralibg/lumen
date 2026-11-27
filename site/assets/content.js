// Site content: components and guides. Rendered by app.js.

export const REPO = "https://github.com/amiralibg/lumen";

export const GROUPS = [
  "Actions",
  "Inputs",
  "Selection",
  "Navigation",
  "Overlays",
  "Feedback",
  "Data",
  "Planning",
  "AI",
  "Developer tools",
  "Dates",
  "Charts",
  "Motion",
  "Layout and surfaces",
];

// demo: the lumen-demos name shown live on the page.
export const COMPONENTS = [
  {
    slug: "button", name: "Button", group: "Actions", demo: "button", icon: "M5 12h14",
    summary: "Primary, secondary, ghost and danger buttons in three sizes, with icons, tooltips and a spring press.",
    code: `if ui.button("Deploy").primary().icon("rocket").clicked() {
    deploy();
}
ui.button("Cancel").ghost();
ui.button("Delete").danger().disabled(!confirmed);
ui.icon_button("copy").tooltip_shortcut("Copy", "⌘C");`,
    options: [
      [".primary() .secondary() .ghost() .danger()", "Variant. Secondary is the default. Use one primary per view."],
      [".small() .large() .size(ControlSize)", "Height, padding and type size come from the current look."],
      [".icon(name) .trailing_icon(name)", "Any Lucide icon name."],
      [".disabled(bool) .loading(bool) .full_width()", "Loading shows an inline spinner and prevents double activation."],
      [".tooltip(text) .tooltip_shortcut(text, keys)", "Shown after a delay; instant while moving between tooltips."],
      [".clicked() .show()", "Add the button and read its Response."],
    ],
    keys: [["Space / Enter", "Activate"], ["Tab", "Move focus; the ring shows only for keyboard focus"]],
  },
  {
    slug: "split-button", name: "Split button", group: "Actions", demo: "split-button",
    summary: "A main action joined seamlessly with a chevron trigger that opens a menu of alternate actions, plus inline loading buttons.",
    code: `if ui.split_button("Deploy production").primary().icon("rocket").show(|ui| {
    ui.menu_item("zap", "Deploy canary (10%)").shortcut("⌘⇧C");
    ui.menu_item("play", "Dry-run validation");
}).clicked() {
    deploy();
}
ui.button("Deploying").primary().loading(true);`,
    options: [
      [".primary() .secondary() .danger() .small()", "Variant and size for both joined halves."],
      [".icon(name) .disabled(bool)", "Leading icon on the main action half."],
      [".show(|ui| ..) -> Response", "Builds the dropdown menu and returns the main half's Response."],
    ],
    keys: [["Tab", "Focus main action, then chevron trigger"], ["Space / Enter", "Fire main action or open menu"]],
  },
  {
    slug: "hold-button", name: "Hold and copy buttons", group: "Actions", demo: "hold-button",
    summary: "A destructive button that sweeps full while held to prevent accidental clicks without a modal, and a clipboard button that confirms with a check.",
    code: `if ui.hold_button("Hold to purge cache", 0.9).icon("trash-2").confirmed() {
    purge_cache();
}
ui.copy_button("Copy token", "rk_live_9d2f4b8c1e7a");`,
    options: [
      ["ui.hold_button(label, secs)", ".danger() (default), .primary(), .secondary(), .icon(name), .confirmed()"],
      ["ui.copy_button(label, text)", "Copies text and switches its icon to a check for 1.4 s."],
    ],
    keys: [["Hold Space / Enter", "Fills and confirms; releasing early springs back to zero"]],
  },
  {
    slug: "text-input", name: "Text input", group: "Inputs", demo: "text-input",
    summary: "Single-line field with label, icon, placeholder, helper text and an animated error message.",
    code: `ui.text_input(&mut s.email)
    .label("Email")
    .icon("mail")
    .placeholder("you@example.com")
    .error((!s.email.contains('@')).then_some("Enter a valid email."))
    .helper("We never share it.")
    .fill_width();`,
    options: [
      [".label(t) .placeholder(t) .icon(name)", "Chrome around the field."],
      [".helper(t) .error(Option<&str>)", "The error slides in with a spring and replaces the helper."],
      [".mono() .disabled(b) .width(px) .fill_width()", ""],
    ],
    keys: [["⌘/Ctrl + A, C, X, V, Z", "Select all, copy, cut, paste, undo"], ["Alt/Ctrl + ← →", "Jump by word"], ["Double / triple click", "Select word / all"]],
  },
  {
    slug: "text-area", name: "Text area", group: "Inputs", demo: "text-area",
    summary: "Multi-line field that springs in height between min and max rows, scrolls with an overlay thumb when full, and optionally tracks a character limit.",
    code: `ui.text_area(&mut s.notes)
    .label("Release notes")
    .placeholder("Summarize what shipped…")
    .rows(3, 6)
    .max_chars(240)
    .helper("Shown on the public changelog.")
    .fill_width();`,
    options: [
      [".rows(min, max)", "Visible line bounds before scrolling (default 3..=8); height animates on the smooth spring."],
      [".max_chars(n)", "Caps input and shows a live n / max counter that turns warning and danger near the limit."],
      [".label(t) .placeholder(t) .helper(t) .error(Option<&str>)", ""],
      [".mono() .disabled(b) .width(px) .fill_width()", ""],
    ],
    keys: [["Enter", "Insert a newline"], ["↑ ↓", "Move across wrapped lines while keeping the column"], ["⌘/Ctrl + A, C, X, V, Z", "Select all, copy, cut, paste, undo"]],
  },
  {
    slug: "number-field", name: "Number field", group: "Inputs", demo: "number-field",
    summary: "Numeric input with drag-to-scrub on the handle or label, inline text editing, prefix and suffix units, and − / + steppers.",
    code: `ui.number_field(&mut s.timeout_ms)
    .label("Timeout")
    .range(100.0..=30000.0)
    .step(50.0)
    .suffix("ms");
ui.number_field(&mut s.cap).label("Spend cap").prefix("$").step(10.0);`,
    options: [
      [".range(min..=max) .step(s) .decimals(n)", "Bounds, increment and resting precision."],
      [".prefix(t) .suffix(t) .icon(name)", "Units and leading scrub-handle badge."],
      [".scrub_pixels(px) .controls(bool)", "Pixels per step while dragging (default 4.0) and trailing − / + buttons."],
      [".label(t) .helper(t) .error(Option<&str>) .width(px) .fill_width() .disabled(b)", ""],
    ],
    keys: [["Drag handle or label", "Scrub value (Shift 10×, Alt 0.1×)"], ["Click value", "Select all and edit as text"], ["↑ ↓ (Shift 10×)", "Step up or down"], ["Enter / Escape", "Commit typed number / revert"]],
  },
  {
    slug: "tag-input", name: "Tag input", group: "Inputs", demo: "tag-input",
    summary: "Free-form tags inside a wrapping field whose height springs open as chips flow onto new lines. Enter or comma commits; duplicate tags bounce.",
    code: `ui.tag_input(&mut s.tags)
    .label("Runner labels")
    .lowercase(true)
    .max_tags(8)
    .helper("Press Enter or comma to add.")
    .fill_width();`,
    options: [
      [".max_tags(n) .lowercase(bool)", "Cap total tags (with live counter) and normalize casing."],
      [".label(t) .placeholder(t) .helper(t) .error(Option<&str>)", ""],
      [".width(px) .fill_width() .disabled(b)", ""],
    ],
    keys: [["Enter / ,", "Commit current text (or pasted comma-separated list) as tags"], ["Backspace (twice on empty)", "Arm, then remove the last tag"], ["Escape", "Clear uncommitted draft"]],
  },
  {
    slug: "otp-input", name: "One-time code", group: "Inputs", demo: "otp-input",
    summary: "Individual verification-code cells in a single Tab stop: typing auto-advances with a bouncy pop, and pasting fills the entire code at once.",
    code: `if ui.otp_input("2fa", &mut s.code).completed() {
    verify(&s.code);
}`,
    options: [
      [".digits(n) .group(n)", "Cell count (3..=8, default 6) and separator dash interval (default 3)."],
      [".alphanumeric() .masked(bool)", "Allow A–Z codes or mask digits as bullets for PINs."],
      [".completed()", "True on the frame the final slot is filled."],
      [".label(t) .helper(t) .error(Option<&str>) .disabled(b)", ""],
    ],
    keys: [["0–9 (or A–Z)", "Fill active cell and advance"], ["⌘/Ctrl + V", "Paste full code (ignores spaces and hyphens)"], ["Backspace / Delete", "Clear cell and step back"], ["← → Home End", "Move between cells"]],
  },
  {
    slug: "password-input", name: "Password input", group: "Inputs", demo: "password-input",
    summary: "Masked password field with a reveal toggle, optional copy button, and a four-segment strength meter that morphs in OKLab from weak to strong.",
    code: `ui.password_input(&mut s.password)
    .label("Master password")
    .icon("lock")
    .strength(true)
    .copyable(true)
    .fill_width();`,
    options: [
      [".strength(bool)", "Animated four-bar meter plus grade and actionable hint."],
      [".copyable(bool) .icon(name)", "Trailing clipboard button with confirmation check."],
      ["lumen::password_strength(pw)", "Returns the 0..=4 strength score directly."],
      [".label(t) .placeholder(t) .helper(t) .error(Option<&str>) .fill_width()", ""],
    ],
    keys: [["Tab to eye button + Space/Enter", "Reveal or mask password"], ["⌘/Ctrl + A, V, Z", "Select all, paste, undo (copy is blocked while masked)"]],
  },
  {
    slug: "search-field", name: "Search field", group: "Inputs", demo: "search-field",
    summary: "Filter and search input with an accent-tinted magnifier, inline loading spinner, shortcut hint chip, match counter and spring-in clear button.",
    code: `ui.search_field(&mut s.query)
    .placeholder("Filter deployments…")
    .shortcut("/")
    .focus_key('/')
    .results(Some(matches.len()))
    .fill_width();`,
    options: [
      [".shortcut(t) .focus_key(ch) .cmd_key(ch)", "Keycap hint when idle, plus plain-key or ⌘/Ctrl shortcut focus."],
      [".loading(bool) .results(Option<usize>)", "Swap icon for a spinner while fetching; show live match count."],
      [".label(t) .placeholder(t) .width(px) .fill_width() .disabled(b)", ""],
    ],
    keys: [["/ or ⌘F", "Focus and select query (when configured)"], ["Escape", "Clear query first; press again to blur"]],
  },
  {
    slug: "select", name: "Select", group: "Inputs", demo: "select",
    summary: "A dropdown that picks one value from a list, in a popover matched to the field's width.",
    code: `ui.select(&mut s.region, &[
    (Region::Eu, "Frankfurt"),
    (Region::Us, "Virginia"),
    (Region::Ap, "Tokyo"),
])
.label("Region")
.fill_width();`,
    options: [[".label(t) .placeholder(t)", ""], [".width(px) .fill_width() .disabled(b)", ""], [".changed()", "True on the frame the value changed."]],
    keys: [["Space / Enter / ↓", "Open, landing on the selected option"], ["↑ ↓ Home End", "Move through options"], ["Enter", "Choose"], ["Escape", "Close and return focus"]],
  },
  {
    slug: "combobox", name: "Combobox", group: "Inputs", demo: "combobox",
    summary: "Type to filter a list and pick one value. Matched spans highlight in the accent, rows stagger in, and a highlight pill slides between matches.",
    code: `let zones: Vec<(usize, &str)> = ZONES.iter().copied().enumerate().collect();
ui.combobox(&mut s.tz, &zones)
    .label("Time zone")
    .placeholder("Search time zones")
    .fill_width();`,
    options: [
      [".label(t) .placeholder(t)", "Clearing the query and leaving sets the value to None."],
      [".max_rows(n) .width(px) .fill_width() .disabled(b)", "Long lists scroll and keep the active row in view."],
      [".changed() .show()", ""],
    ],
    keys: [["Type", "Fuzzy-filter options and highlight matched letters"], ["↑ ↓ Home End", "Move the highlight pill"], ["Enter", "Pick the highlighted option"], ["Escape", "Close and revert to the current value"]],
  },
  {
    slug: "multi-select", name: "Multi-select", group: "Inputs", demo: "multi-select",
    summary: "Pick several options as removable chips inside a field that grows on a spring as chips wrap.",
    code: `let people: Vec<(usize, &str)> = PEOPLE.iter().copied().enumerate().collect();
ui.multi_select(&mut s.reviewers, &people)
    .label("Reviewers")
    .placeholder("Add reviewers")
    .fill_width();`,
    options: [
      [".label(t) .placeholder(t)", "Placeholder shows only while no values are chosen."],
      [".max_rows(n) .width(px) .fill_width() .disabled(b)", ""],
      [".changed() .show()", ""],
    ],
    keys: [["Type", "Filter options"], ["↑ ↓", "Move through matches"], ["Enter", "Toggle the highlighted option"], ["Backspace (twice on empty)", "Arm, then remove the last chip"], ["Escape", "Close the list"]],
  },
  {
    slug: "color-picker", name: "Color picker", group: "Inputs", demo: "color-picker",
    summary: "A swatch trigger that opens a lumen-drawn saturation–value canvas, hue and opacity sliders, preset swatches and a live hex field.",
    code: `ui.color_picker("Brand color", &mut s.brand);
ui.color_picker("Spotlight glow", &mut s.glow).alpha(true);`,
    options: [
      [".alpha(bool)", "Adds a checkerboard opacity slider and 8-digit #RRGGBBAA hex editing."],
      [".swatches(&[Color])", "Custom preset row; defaults to lumen's eight palette hues."],
      [".width(px) .fill_width() .disabled(b)", ""],
      ["ui.color_editor(key, &mut Color, alpha)", "Inline color editor without the popover trigger."],
    ],
    keys: [["Space / Enter / ↓", "Open with focus on the 2D canvas"], ["← → ↑ ↓ (Shift 5×)", "Nudge saturation/value, hue or opacity"], ["Enter in hex", "Apply and close"], ["Escape", "Close and return focus"]],
  },
  {
    slug: "slider", name: "Slider", group: "Inputs", demo: "slider",
    summary: "Continuous or stepped values. Clicking away from the thumb springs it there; dragging tracks 1:1.",
    code: `let pct = format!("{:.0}%", s.volume * 100.0);
ui.slider("Volume", &mut s.volume, 0.0..=1.0).readout(pct);
ui.slider("Price", &mut s.price, 0.0..=100.0).step(5.0);`,
    options: [[".step(s)", "Snap to multiples from the range start."], [".readout(text) .show_value()", ""], [".width(px) .disabled(b)", ""]],
    keys: [["← → ↑ ↓", "One step (Shift for ten)"], ["Home / End", "Minimum / maximum"]],
  },
  {
    slug: "range-slider", name: "Range slider", group: "Inputs", demo: "range-slider",
    summary: "Dual-thumb interval slider with an active accent band, independent keyboard-focusable thumbs and look-matched knobs.",
    code: `let txt = format!("\${:.0} – \${:.0}", s.band.0, s.band.1);
ui.range_slider("Instance budget", &mut s.band, 0.0..=120.0)
    .step(5.0)
    .readout(txt)
    .fill_width();`,
    options: [
      [".step(s) .readout(text)", "Quantize both thumbs and display the formatted span."],
      [".width(px) .fill_width() .disabled(b)", ""],
    ],
    keys: [["Tab", "Focus low thumb, then high thumb"], ["← → ↑ ↓ (Shift 10×)", "Nudge focused thumb within bounds"], ["Home / End", "Snap to bound"]],
  },
  {
    slug: "shortcut-recorder", name: "Shortcut recorder", group: "Inputs", demo: "shortcut-recorder",
    summary: "Click to arm recording and press any modifier + key chord to capture it as sculpted keycaps.",
    code: `ui.shortcut_recorder("deploy-key", &mut s.chord)
    .label("Deploy shortcut")
    .fill_width();`,
    options: [
      ["ui.shortcut_recorder(key, &mut Vec<String>)", ".label(t) .width(px) .fill_width() .disabled(b)"],
    ],
    keys: [["Enter / Space", "Arm recording"], ["Modifiers + Key", "Capture chord"], ["Backspace", "Clear shortcut"], ["Escape", "Cancel recording"]],
  },
  {
    slug: "dropzone", name: "Dropzone", group: "Inputs", demo: "dropzone",
    summary: "Upload target zone with a hover-lifted icon tile, format hint and removable file rows.",
    code: `if ui.dropzone("artifacts", "Drop build artifacts or click to add")
    .hint("WASM, dSYM or tar.zst up to 50 MB")
    .files(&mut s.uploads)
    .clicked()
{
    pick_files();
}`,
    options: [
      [".hint(t) .files(&mut Vec<(String, String)>)", "Format caption and mutable (name, size) list with remove buttons."],
      [".disabled(b) .clicked()", ""],
    ],
    keys: [["Space / Enter", "Trigger file picker"], ["Tab to ×", "Remove attached file"]],
  },
  {
    slug: "switch", name: "Switch", group: "Selection", demo: "switch",
    summary: "On/off toggle. The knob stretches while pressed and springs across. Shape follows the look.",
    code: `ui.switch("Wi-Fi", &mut s.wifi);
ui.switch("", &mut s.flag).disabled(locked); // no label in settings rows`,
    options: [[".disabled(b)", ""], [".changed()", ""]],
    keys: [["Space", "Toggle"]],
  },
  {
    slug: "checkbox", name: "Checkbox", group: "Selection", demo: "checkbox",
    summary: "Checked, unchecked and mixed states with a wiping check mark and optional description.",
    code: `let parent = ui.checkbox("All channels", &mut all)
    .indeterminate(some_but_not_all)
    .show();
ui.checkbox("Email", &mut s.email).description("Digests and receipts");`,
    options: [[".indeterminate(b)", "Mixed state for a parent; clicking makes it checked."], [".description(t) .disabled(b)", ""]],
    keys: [["Space", "Toggle"]],
  },
  {
    slug: "radio", name: "Radio", group: "Selection", demo: "radio",
    summary: "One choice from a few. Radios bound to the same value form a group.",
    code: `ui.radio(&mut s.plan, Plan::Hobby, "Hobby — free");
ui.radio(&mut s.plan, Plan::Pro, "Pro — $20 / month");
ui.radio(&mut s.plan, Plan::Team, "Team — $80 / month");`,
    options: [["ui.radio(&mut value, option, label)", "Selects `option` when clicked. Returns a Response."]],
    keys: [["Space / Enter", "Select"]],
  },
  {
    slug: "radio-cards", name: "Radio cards", group: "Selection", demo: "radio-cards",
    summary: "Selectable plan and configuration cards with leading icon tiles, price badges, descriptions and a spring-animated radio dot.",
    code: `ui.radio_card(&mut s.tier, 1, "Performance ARM64")
    .icon("zap")
    .badge("$32 / mo")
    .description("8 dedicated Ampere cores · 16 GB ECC.");`,
    options: [
      [".icon(name) .badge(text) .description(text)", "Card chrome; selected card tints with the accent."],
      [".disabled(bool) .changed()", ""],
    ],
    keys: [["Tab", "Move between cards"], ["Space / Enter", "Select card"]],
  },
  {
    slug: "rating", name: "Rating", group: "Selection", demo: "rating",
    summary: "Interactive star bar with live hover preview, bouncy selection pop and toggle-to-clear on the active star.",
    code: `ui.rating("release", &mut s.stars).size(22.0);`,
    options: [
      [".max(n) .size(px) .icon(name) .disabled(b)", "3..=10 stars (default 5) and any Lucide glyph."],
    ],
    keys: [["← → ↑ ↓", "Step rating"], ["Home / End", "Zero / maximum"]],
  },
  {
    slug: "segmented", name: "Segmented control", group: "Selection", demo: "segmented",
    summary: "A compact choice between a few views, with a pill that slides to the selection.",
    code: `ui.segmented(&mut s.range, &[
    (Range::Day, "Day"),
    (Range::Week, "Week"),
    (Range::Month, "Month"),
]);`,
    options: [["ui.segmented(&mut value, &[(value, label)])", "Returns a Response; .changed when the choice moves."]],
    keys: [["← →", "Move and select"]],
  },
  {
    slug: "tabs", name: "Tabs", group: "Navigation", demo: "tabs",
    summary: "A row of tabs with a sliding accent underline. Draw the selected panel yourself below.",
    code: `ui.tabs(&mut s.tab, &[(Tab::Overview, "Overview"), (Tab::Activity, "Activity")]);
match s.tab {
    Tab::Overview => overview(ui),
    Tab::Activity => activity(ui),
}`,
    options: [["ui.tabs(&mut value, &[(value, label)])", "Windows draws a short pill indicator; other looks a full underline."]],
    keys: [["← →", "Previous / next tab"]],
  },
  {
    slug: "breadcrumb", name: "Breadcrumb", group: "Navigation", demo: "breadcrumb",
    summary: "A path trail ending at the active page, with an optional root icon and middle truncation into an interactive … menu.",
    code: `const TRAIL: &[&str] = &["Northwind", "Infrastructure", "Frankfurt", "linux-arm64-02"];
if let Some(i) = ui.breadcrumb("path", TRAIL).root_icon("layers").max_items(4).show() {
    navigate(i);
}`,
    options: [
      [".root_icon(name) .slash()", "Optional leading icon and / separator instead of chevrons."],
      [".max_items(n)", "Collapses middle crumbs into an ellipsis button that opens a menu of hidden ancestors."],
      [".show() -> Option<usize>", "Returns the clicked ancestor index."],
    ],
    keys: [["← → Home End", "Move focus across ancestor crumbs"], ["Space / Enter", "Navigate or open the collapsed menu"]],
  },
  {
    slug: "pagination", name: "Pagination", group: "Navigation", demo: "pagination",
    summary: "Page bar with a sliding accent pill, stable ellipsis window, item-range counter and a compact rolling-number mode.",
    code: `ui.pagination("builds", &mut s.page, 18)
    .summary(174, 10);`,
    options: [
      [".siblings(n)", "Neighboring page buttons around the active page (default 1)."],
      [".summary(total_items, per_page)", "Shows “41–50 of 174” to the left of the controls."],
      [".compact()", "Compact ← Page n of total → bar with a vertically rolling number."],
    ],
    keys: [["← →", "Step previous / next page"], ["Home / End", "Jump to first / last page"], ["Space / Enter", "Activate focused button"]],
  },
  {
    slug: "steps", name: "Steps", group: "Navigation", demo: "steps",
    summary: "Multi-step wizard header or vertical checklist. Completed steps grow a bouncy check mark while connector lines fill on a smooth spring.",
    code: `const STEPS: &[(&str, &str)] = &[
    ("Build", "Image signed"),
    ("Canary", "10% traffic"),
    ("Promote", "All regions"),
];
ui.steps("rollout", &mut s.step, STEPS);`,
    options: [
      [".linear(bool)", "Restrict clicks to steps up to the furthest step reached so far."],
      [".vertical()", "Vertical step timeline with vertical spring-filled connectors."],
      [".changed() .show()", ""],
    ],
    keys: [["← → (or ↑ ↓ vertical)", "Move between reachable steps"], ["Space / Enter", "Jump to focused step"]],
  },
  {
    slug: "sidebar", name: "Sidebar", group: "Navigation", demo: "hero",
    summary: "Navigation with sections, badges and a selection pill that slides between items. Collapses to icons.",
    code: `ui.sidebar().collapsed(s.collapsed).show(|ui| {
    ui.nav_header("N", "Northwind");
    ui.nav_section("Workspace");
    ui.nav_item("layout-dashboard", "Overview").selected(true);
    ui.nav_item("users", "Customers").badge(4);
});`,
    options: [[".collapsed(b) .width(px)", "Width springs; labels fade."], ["nav_item(..).badge(n) .selected(b) .disabled(b)", ""], ["nav_header nav_section nav_footer", ""]],
    keys: [["↑ ↓", "Move between items"], ["Enter", "Choose"]],
  },
  {
    slug: "toolbar", name: "Toolbar", group: "Navigation", demo: "toolbar",
    summary: "A floating raised strip of icon toggles, momentary actions, separators and a trailing primary CTA.",
    code: `ui.toolbar("editor", |tb| {
    tb.toggle("bold", "Bold (⌘B)", &mut s.bold);
    tb.toggle("italic", "Italic (⌘I)", &mut s.italic);
    tb.separator();
    tb.action("align-left", "Align left");
    tb.primary("Share");
});`,
    options: [
      ["tb.toggle(icon, tip, &mut bool)", "Accent-tinted toggle button with press squash."],
      ["tb.action(icon, tip) tb.separator() tb.primary(label)", ""],
    ],
    keys: [["Tab + Space / Enter", "Toggle or trigger toolbar controls"]],
  },
  {
    slug: "status-bar", name: "Status bar", group: "Navigation", demo: "status-bar",
    summary: "Compact 28 px bottom chrome bar for live connection status, git branch, latency and tabular editor readouts.",
    code: `ui.status_bar(|bar| {
    bar.status("Connected", Tone::Success);
    bar.item("git-branch", "ethereal*");
    bar.spacer();
    bar.text("Ln 142, Col 18");
});`,
    options: [
      ["bar.status(label, Tone)", "Pulsed status dot and label."],
      ["bar.item(icon, label) -> bool", "Clickable segment with hover pill and focus ring."],
      ["bar.text(label) bar.spacer()", "Tabular read-only readout."],
    ],
    keys: [["Tab + Space / Enter", "Activate clickable status segments"]],
  },
  {
    slug: "menu", name: "Menu", group: "Overlays", demo: "menu",
    summary: "A popover of actions with icons, shortcuts, separators and danger items. macOS highlights rows in the accent.",
    code: `let r = ui.button("Actions").trailing_icon("chevron-down").show();
ui.menu(&r, |ui| {
    ui.menu_item("pencil", "Rename").shortcut("⌘R");
    ui.menu_separator();
    if ui.menu_item("trash-2", "Delete").danger().clicked() {
        delete();
    }
});`,
    options: [["menu_item(icon, label)", ".shortcut(t) .danger() .disabled(b) .checked(b)"], ["menu_separator() menu_label(t)", ""]],
    keys: [["Enter / Space on the button", "Open and focus the first item"], ["↑ ↓ Home End", "Move"], ["Escape", "Close"]],
  },
  {
    slug: "popover", name: "Popover", group: "Overlays", demo: "popover",
    summary: "A floating panel anchored to any widget. It flips when it doesn't fit and closes on Escape or an outside click.",
    code: `let r = ui.button("Share").icon("share-2").show();
ui.popover(&r).width(300.0).pad(16.0).show(|ui| {
    ui.label("Share this board").strong();
    if ui.button("Copy link").primary().clicked() {
        ui.close_layer();
    }
});`,
    options: [[".placement(Placement) .align(LayerAlign)", "Bottom-start by default."], [".match_width() .width(px) .pad(px) .gap(px)", ""], [".manual()", "Control with ui.set_popover_open(id, b)."]],
    keys: [["Escape", "Close and return focus to the anchor"]],
  },
  {
    slug: "dialog", name: "Dialog", group: "Overlays", demo: "dialog",
    summary: "A modal over a scrim that rises in with a spring. Escape and scrim clicks dismiss it.",
    code: `ui.dialog("delete", &mut s.confirm)
    .title("Delete “Northwind”?")
    .description("This can't be undone.")
    .footer(|ui| {
        if ui.button("Cancel").ghost().clicked() { ui.close_layer(); }
        if ui.button("Delete").danger().clicked() { delete(); ui.close_layer(); }
    })
    .show(|_| {});`,
    options: [[".title(t) .description(t) .width(px)", ""], [".footer(|ui| ..)", "Right-aligned buttons."], [".dismissible(b)", "Allow Escape and scrim clicks (default true)."]],
    keys: [["Escape", "Close"], ["Tab", "Moves through the dialog's controls"]],
  },
  {
    slug: "sheet", name: "Sheet", group: "Overlays", demo: "sheet",
    summary: "An edge-docked drawer over a scrim with a scrollable body and optional pinned footer. Slides in from any edge on the smooth spring.",
    code: `ui.sheet("runner", &mut s.open)
    .side(Placement::Right)
    .title("linux-arm64-02")
    .description("Frankfurt · 4 concurrent jobs")
    .footer(|ui| {
        if ui.button("Cancel").ghost().clicked() { ui.close_layer(); }
        if ui.button("Save").primary().clicked() { save(); ui.close_layer(); }
    })
    .show(|ui| {
        ui.text_input(&mut s.hostname).label("Hostname").fill_width();
    });`,
    options: [
      [".side(Placement)", "Right (default), Left, Bottom or Top."],
      [".title(t) .description(t) .size(px)", "Width for left/right drawers; height for top/bottom."],
      [".footer(|ui| ..) .dismissible(b)", "Scrollable body sits between the header and footer."],
    ],
    keys: [["Escape", "Close and return focus to the trigger"], ["Tab", "Moves through the drawer's controls"]],
  },
  {
    slug: "command-palette", name: "Command palette", group: "Overlays", demo: "command-palette",
    summary: "A modal ⌘K / Ctrl+K command runner that fuzzy-filters grouped actions, highlights matched characters and shows shortcut chips.",
    code: `const CMDS: &[Command] = &[
    Command::new("deploy", "Deploy production").group("Actions").icon("rocket").shortcut("⌘D"),
    Command::new("settings", "Workspace settings").group("Navigation").icon("settings").shortcut("⌘,"),
];
if let Some(id) = ui.command_palette("main", &mut s.palette, CMDS).show() {
    run(id);
}`,
    options: [
      ["Command::new(id, label)", ".group(g) .icon(name) .shortcut(keys) .keywords(extra)"],
      [".placeholder(t) .width(px)", ""],
      [".global_shortcut(bool)", "Toggles automatically on ⌘K / Ctrl+K (default true)."],
    ],
    keys: [["⌘/Ctrl + K", "Toggle the palette"], ["Type", "Fuzzy-filter labels, groups and keywords"], ["↑ ↓ Home End", "Move the highlight pill"], ["Enter", "Run the highlighted command"], ["Escape", "Close and return focus"]],
  },
  {
    slug: "hover-card", name: "Hover card", group: "Overlays", demo: "hover-card",
    summary: "A rich preview card that opens after a short hover or keyboard-focus delay and stays open while the pointer crosses into it.",
    code: `let r = ui.button("@jonas").ghost().small().show();
ui.hover_card(&r).width(276.0).show(|ui| {
    ui.row().gap(12).align(Align::Center).show(|ui| {
        ui.avatar("JB", 0, 38.0, Some(true));
        ui.label("Jonas Berg").strong();
    });
});`,
    options: [
      [".placement(Placement) .align(LayerAlign)", "Bottom-start by default; flips when near the window edge."],
      [".width(px) .pad(px) .delay(secs)", "Default delay 0.28 s with a 0.16 s pointer-bridge grace window."],
    ],
    keys: [["Tab to anchor", "Opens after the delay"], ["Escape", "Dismiss"]],
  },
  {
    slug: "context-menu", name: "Context menu", group: "Overlays", demo: "context-menu",
    summary: "Opens at the pointer on right-click (or Shift+Enter when focused), using the same items, checks, shortcuts and danger styling as Menu.",
    code: `let zone = ui.column().id("row").show(|ui| { /* content */ });
ui.context_menu(&zone.response(), |ui| {
    ui.menu_item("copy", "Copy commit SHA").shortcut("⌘C");
    ui.menu_separator();
    ui.menu_item("rotate-ccw", "Roll back…").danger();
});`,
    options: [
      ["ui.context_menu(&Response, |ui| ..)", "Uses menu_item, menu_separator and menu_label; flips to stay on screen."],
      ["ui.open_context_menu(id, pos)", "Open programmatically at a point."],
    ],
    keys: [["Shift + Enter", "Open on a focused target"], ["↑ ↓ Home End", "Move between items"], ["Enter / Space", "Activate"], ["Escape", "Close and return focus"]],
  },
  {
    slug: "confirm-popover", name: "Confirm popover", group: "Overlays", demo: "confirm-popover",
    summary: "An inline confirmation anchored right next to a destructive button — lighter than a modal dialog so your eyes stay in place.",
    code: `let r = ui.button("Revoke").danger().small().show();
if ui.confirm_popover(&r, "Revoke this API key?")
    .description("Clients using this key will fail immediately.")
    .confirm_label("Revoke key")
    .confirmed()
{
    revoke();
}`,
    options: [
      [".description(t) .confirm_label(t) .cancel_label(t)", ""],
      [".danger(bool)", "Destructive confirm button by default; false uses primary."],
      [".placement(Placement) .width(px)", ""],
    ],
    keys: [["Space / Enter on trigger", "Open and focus Cancel"], ["Tab", "Move between Cancel and Confirm"], ["Escape", "Cancel and return focus"]],
  },
  {
    slug: "toast", name: "Toast", group: "Feedback", demo: "toast",
    summary: "Short notices that stack in the corner, slide in with a spring and leave on their own.",
    code: `ui.toast(Toast::new("Changes saved").description("Your profile is up to date.").success());
ui.toast(Toast::new("Deploy failed").danger().duration(0.0)); // stays until closed`,
    options: [[".description(t) .tone(Tone) .success() .danger()", ""], [".duration(secs)", "0 keeps it until dismissed. Default 4 s."]],
    keys: [],
  },
  {
    slug: "tooltip", name: "Tooltip", group: "Feedback", demo: "tooltip",
    summary: "Hover and keyboard-focus tooltips with optional shortcut chips and a warm-up delay.",
    code: `ui.icon_button("bold").ghost().tooltip_shortcut("Bold", "⌘B");
ui.tooltip_for(id, TooltipSpec { text: "Details".into(), shortcut: None, placement: Placement::Right, gap: 8.0 });`,
    options: [[".tooltip(t) .tooltip_shortcut(t, keys)", "On buttons."], ["ui.tooltip_for(id, spec)", "On anything with an id."]],
    keys: [["Tab to a control", "Shows its tooltip after the delay"]],
  },
  {
    slug: "badge", name: "Badge and status", group: "Feedback", demo: "badge",
    summary: "Pill labels and status dots in six tones.",
    code: `ui.badge("Live", Tone::Success);
ui.badge("Beta", Tone::Info);
ui.status("Partial outage", Tone::Warning);`,
    options: [["Tone", "Neutral, Accent, Success, Warning, Danger, Info"]],
    keys: [],
  },
  {
    slug: "progress", name: "Progress and spinner", group: "Feedback", demo: "progress",
    summary: "Determinate bars that spring to new values, an indeterminate sweep, and a spinner.",
    code: `ui.progress(Some(s.uploaded));
ui.progress(None);   // indeterminate
ui.spinner(20.0);`,
    options: [["progress(Option<f32>)", "0..1, or None for indeterminate."], ["spinner(size)", ""]],
    keys: [],
  },
  {
    slug: "meter", name: "Segmented meter", group: "Feedback", demo: "meter",
    summary: "Multi-part quota and storage bar whose segments spring to their shares and highlight when you hover the legend.",
    code: `ui.meter("storage", 100.0, &[
    ("Artifacts", 42.0),
    ("Build cache", 21.5),
    ("Logs", 12.0),
])
.label("Workspace storage")
.unit("GB");`,
    options: [
      [".label(t) .unit(u)", "Header and tabular used / capacity readout."],
    ],
    keys: [],
  },
  {
    slug: "alert", name: "Alert and banner", group: "Feedback", demo: "alert",
    summary: "Inline callouts and edge-accented banners across all six tones, with optional action buttons and spring-collapsed dismissal.",
    code: `ui.alert("Scheduled maintenance")
    .description("Frankfurt upgrades tonight at 02:00 UTC.")
    .warning()
    .banner()
    .dismissible(&mut s.banner_open);

if ui.alert("Renewal delayed").danger().action("Retry").clicked() {
    retry();
}`,
    options: [
      [".info() .success() .warning() .danger() .accent() .tone(Tone)", "Status tint, border and icon."],
      [".banner()", "Compact horizontal strip with a leading 3.5 px accent bar."],
      [".action(label) .dismissible(&mut bool) .icon(name)", "Action button and spring collapse on close."],
    ],
    keys: [["Tab + Space / Enter", "Trigger action or dismiss"]],
  },
  {
    slug: "skeleton", name: "Skeleton", group: "Feedback", demo: "skeleton",
    summary: "Placeholder blocks, circles and tapered paragraphs with a continuous spatial shimmer across SDF-clipped corners (static under reduced motion).",
    code: `ui.row().gap(12).align(Align::Center).show(|ui| {
    ui.skeleton_circle(40.0);
    ui.column().fill_width().gap(6).show(|ui| {
        ui.skeleton(140.0, 14.0);
        ui.skeleton(200.0, 11.0);
    });
});
ui.skeleton_lines(3);`,
    options: [
      ["ui.skeleton(w, h)", ".fill_width() .radius(px) .circle()"],
      ["ui.skeleton_circle(size)", "Circular avatar placeholder."],
      ["ui.skeleton_lines(n)", "Paragraph lines whose last line tapers to 62% width."],
    ],
    keys: [],
  },
  {
    slug: "empty-state", name: "Empty state", group: "Feedback", demo: "empty-state",
    summary: "Centered placeholder panel with a 20 px icon tile, heading, wrapped description and an optional action slot.",
    code: `ui.empty_state("inbox", "No self-hosted runners yet")
    .description("Install the daemon on a Linux or macOS machine to start.")
    .action(|ui| {
        ui.button("Copy install script").primary().icon("terminal");
        ui.button("Read docs").ghost();
    });`,
    options: [
      [".description(t) .bordered(bool) .compact()", "Card chrome and tighter padding for popovers."],
      [".action(|ui| ..) .show()", "Centered call-to-action controls."],
    ],
    keys: [],
  },
  {
    slug: "kbd", name: "Keycap", group: "Layout and surfaces", demo: "kbd",
    summary: "Sculpted monospace keycaps and multi-key shortcut chords that physically depress on click or while the matching keyboard key is held.",
    code: `ui.kbd("⌘K");
ui.shortcut_keys(&["⌘", "Shift", "D"]);
ui.keycap("↑").large().live_key(Key::ArrowUp);`,
    options: [
      ["ui.kbd(label) / ui.shortcut_keys(&[&str])", "Inline keycap or chord row."],
      ["ui.keycap(label)", ".large() .pressed(bool) .live_key(Key) .interactive()"],
    ],
    keys: [["Matching physical key", "Depresses any keycap configured with .live_key(..)"]],
  },
  {
    slug: "calendar", name: "Calendar", group: "Dates", demo: "calendar",
    summary: "Month grid for a single date or a range. Months slide when you page; today is marked.",
    code: `ui.calendar(&mut s.date);
ui.calendar_range(&mut s.stay); // Option<(Date, Date)>`,
    options: [["Date", "lumen::Date { year, month, day } with add_days, add_months, weekday, parsing and ISO display."]],
    keys: [["← → ↑ ↓", "Day / week"], ["PageUp / PageDown", "Month"], ["Enter", "Pick"]],
  },
  {
    slug: "date-picker", name: "Date picker", group: "Dates", demo: "date-picker",
    summary: "A field that opens a calendar popover; picking a date rolls the new value in and closes the panel.",
    code: `ui.date_picker("Due date", &mut s.due);`,
    options: [["date_picker(label, &mut Date)", "Returns a Response with .changed when a day is picked."]],
    keys: [["Space / Enter / ↓", "Open with focus in the grid"], ["← → ↑ ↓", "Move through days"], ["Escape", "Close"]],
  },
  {
    slug: "date-range-picker", name: "Date range picker", group: "Dates", demo: "date-range-picker",
    summary: "A range field with quick-pick presets, live hover-band preview while choosing the end date, and a night counter.",
    code: `ui.date_range_picker("Maintenance window", &mut s.span); // Option<(Date, Date)>`,
    options: [
      ["date_range_picker(label, &mut Option<(Date, Date)>)", "First click anchors the start; hovering previews the span; second click orders and closes."],
      ["Quick picks", "Today, Last 7 days, Last 30 days, This month and Clear."],
    ],
    keys: [["Space / Enter / ↓", "Open with focus in the calendar"], ["← → ↑ ↓", "Walk days"], ["Enter", "Set start, then end"], ["Escape", "Close"]],
  },
  {
    slug: "time-picker", name: "Time picker", group: "Dates", demo: "time-picker",
    summary: "Scrollable hour and minute columns (plus AM/PM in 12-hour mode) with sliding selection pills and a Now shortcut.",
    code: `ui.time_picker("Deploy at", &mut s.deploy_at);
ui.time_picker("Quiet hours end", &mut s.quiet_end)
    .twelve_hour(true)
    .step(15);`,
    options: [
      [".twelve_hour(bool)", "12-hour display and AM/PM column (default 24-hour)."],
      [".step(minutes)", "Minute interval in the list (default 5, clamped 1..=30)."],
      [".width(px) .fill_width() .disabled(b)", ""],
      ["lumen::Time", "Time::new(h, m), Time::now(), .format_24(), .format_12()."],
    ],
    keys: [["Space / Enter / ↓", "Open and focus the columns"], ["← →", "Switch between Hour, Minute and Period"], ["↑ ↓ Home End", "Step or jump within the column"], ["Enter / Escape", "Confirm or close"]],
  },
  {
    slug: "line-chart", name: "Line chart", group: "Charts", demo: "chart-line",
    summary: "One or more series with nice axes, monotone smoothing, a gliding crosshair and a tooltip. Points grow in from the baseline.",
    code: `ui.line_chart("revenue", &MONTHS, vec![
    Series::new("Revenue", revenue),
    Series::new("Costs", costs),
])
.smooth(true)
.height(260.0)
.format(|v| format!("\${}k", compact(v)));`,
    options: [[".smooth(b) .points(b) .area()", "Smoothing never overshoots the data."], [".height(px) .legend(b) .format(fn)", ""], ["Series::new(name, values).color(c)", "Colors default to the Series0..5 tokens (or the accent for one series)."]],
    keys: [],
  },
  {
    slug: "area-chart", name: "Area chart", group: "Charts", demo: "chart-area",
    summary: "Line charts with a gradient fill to the baseline.",
    code: `ui.area_chart("visitors", &days, vec![Series::new("Visitors", visits)]).smooth(true);`,
    options: [["Same options as the line chart", ""]],
    keys: [],
  },
  {
    slug: "bar-chart", name: "Bar chart", group: "Charts", demo: "chart-bar",
    summary: "Grouped or stacked bars. Hovering a group dims the others and shows every value.",
    code: `ui.bar_chart("platforms", &DAYS, vec![
    Series::new("Web", web),
    Series::new("iOS", ios),
    Series::new("Android", android),
])
.stacked();`,
    options: [[".stacked()", "Positive and negative values stack separately."], [".height(px) .legend(b) .format(fn)", ""]],
    keys: [],
  },
  {
    slug: "donut", name: "Donut chart", group: "Charts", demo: "donut",
    summary: "Parts of a whole. Segments sweep in, pop out on hover, and the center shows the hovered share.",
    code: `ui.donut("traffic", &[("Search", 4200.0), ("Direct", 2600.0), ("Social", 1500.0)]);`,
    options: [["donut_sized(key, parts, size)", "Smaller donuts tighten the legend."]],
    keys: [],
  },
  {
    slug: "gauge", name: "Gauge", group: "Charts", demo: "gauge",
    summary: "A semicircular meter that turns warning and danger as load rises, or stays one color for scores.",
    code: `ui.gauge("cpu", 0.38, "CPU");
ui.gauge_colored("health", 0.82, "Score", ui.color(Token::Success));`,
    options: [["gauge(key, value, caption)", "0..1; accent, then warning above 65%, danger above 85%."], ["gauge_colored(..)", "Fixed color."]],
    keys: [],
  },
  {
    slug: "sparkline", name: "Sparkline and stat", group: "Charts", demo: "sparkline",
    summary: "Inline trend lines, and stat tiles with a value, a delta badge and a trend.",
    code: `ui.stat("Revenue", "$118k", Some(12.4), Some(&trend));
ui.sparkline("cpu", &samples, None);`,
    options: [["stat(label, value, delta, trend)", "Delta color follows its sign."], ["sparkline_sized(key, values, color, width)", ""]],
    keys: [],
  },
  {
    slug: "card", name: "Card and settings rows", group: "Layout and surfaces", demo: "card",
    summary: "Surfaces with a title, description, header action and footer, and the settings-row pattern with dividers.",
    code: `ui.card().title("Notifications").description("Choose what reaches you.").show(|ui| {
    ui.settings(|rows| {
        rows.row("Deploy failures", "When a production deploy fails.", |ui| {
            ui.switch("", &mut s.failures);
        });
    });
});`,
    options: [[".title .description .action(|ui|) .footer(hint, |ui|) .pad(px)", "Radius, border and shadow come from the look."]],
    keys: [],
  },
  {
    slug: "looks", name: "Looks", group: "Layout and surfaces", demo: "looks",
    summary: "The same code in lumen's look, macOS, Windows 11 or GNOME. Switching blends colors, corners, sizes and motion together.",
    code: `ui.set_look(Look::mac().rounded(1.2).motion(Motion::playful()));
ui.set_accent(Accent::Native); // the OS accent color`,
    options: [["Look::lumen() mac() windows() gnome() native()", ""], [".rounded(f) .density(Density) .text_scale(s) .motion(m) .bundled_fonts()", ""], ["ui.override_color(mode, token, color)", "Replace any token."]],
    keys: [],
  },
  {
    slug: "chips", name: "Filter chips", group: "Selection", demo: "chips",
    summary: "Pills that toggle. When one turns on it tints with the accent and a check springs in ahead of the label, pushing the text over.",
    code: `for (i, label) in ["arm64", "gpu", "docker"].iter().enumerate() {
    ui.chip(label, &mut labels[i]);
}`,
    options: [["ui.chip(label, &mut bool) -> Response", "Press squashes the pill to 95%; the check uses the bouncy spring."]],
    keys: [["Space / Enter", "Toggle"], ["Tab", "Next chip"]],
  },
  {
    slug: "toggle-group", name: "Toggle group", group: "Selection", demo: "toggle-group",
    summary: "Equal cells where one is chosen, like time slots or alignment. The choice fills with the accent and arrow keys move between cells.",
    code: `ui.toggle_group("slots", &mut slot, &[(0, "09:00"), (1, "11:30"), (2, "14:00")]);`,
    options: [["ui.toggle_group(key, &mut T, &[(T, &str)])", "Cells share the row width. Uses the small control height from the look."]],
    keys: [["← →", "Move between cells"], ["Space / Enter", "Choose"]],
  },
  {
    slug: "stepper", name: "Stepper", group: "Inputs", demo: "stepper",
    summary: "A whole-number field with − and + buttons. The number rolls up or down the way it moved, and the buttons disable at the limits.",
    code: `ui.stepper("jobs", &mut jobs, 1, 16);`,
    options: [["ui.stepper(key, &mut i64, min, max) -> Response", "changed() is true on the frame the value moves."]],
    keys: [["↑ ↓", "Step while a button has focus"], ["Space / Enter", "Press the focused button"]],
  },
  {
    slug: "avatar", name: "Avatar", group: "Layout and surfaces", demo: "avatar",
    summary: "Initials on a tinted circle with an optional presence dot, and overlapping groups with a +n count.",
    code: `ui.avatar("JB", 0, 34.0, Some(true)); // online
ui.avatar_group(&["JB", "PN", "TR", "LF", "SW"], 4, 32.0);`,
    options: [["ui.avatar(initials, tone, size, presence)", "Tone picks a chart series color. presence: Some(true) online, Some(false) away."], ["ui.avatar_group(&[&str], max, size)", "Each avatar is ringed in the surface color."]],
    keys: [],
  },
  {
    slug: "accordion", name: "Accordion", group: "Data", demo: "accordion",
    summary: "Stacked disclosure sections and standalone collapsibles whose bodies expand on the smooth spring with rotating vector chevrons.",
    code: `ui.accordion("faq", &mut s.open_idx)
    .item_with("Zero-copy glyph atlas", Some("sparkles"), Some("GPU"), |ui| {
        ui.label("Shaped glyphs stream straight into an R8 coverage texture.").secondary();
    })
    .item("Spring-driven micro-motion", |ui| {
        ui.label("Every disclosure settles analytically.").secondary();
    })
    .show();`,
    options: [
      ["ui.accordion(key, &mut Option<usize>)", ".item(title, |ui| ..) and .item_with(title, icon, badge, |ui| ..)"],
      ["ui.collapsible(key, title, &mut bool)", "Independent disclosure section with .icon(name), .badge(text), .carded(bool)."],
    ],
    keys: [["↑ ↓ / Home / End", "Move focus across accordion headers"], ["Space / Enter", "Expand or collapse focused section"]],
  },
  {
    slug: "table", name: "Data table", group: "Data", demo: "table",
    summary: "Interactive table with sortable column headers, indeterminate header selection checkbox, right-aligned numeric columns and status badges.",
    code: `ui.table("services")
    .columns([
        TableCol::new("Service").fill(1.5).sortable(),
        TableCol::new("p95").width(104.0).sortable().right(),
        TableCol::new("Status").width(96.0).right(),
    ])
    .sort_by(&mut s.sort)
    .selection(&mut s.selected)
    .row(0, [TableCell::strong("edge-proxy"), TableCell::mono("4.2 ms"), TableCell::badge("Healthy", Tone::Success)])
    .show();`,
    options: [
      ["TableCol::new(label)", ".fill(weight), .width(px), .sortable(), .right()"],
      ["TableCell", "::text, ::strong, ::muted, ::mono, ::badge(label, Tone)"],
      [".sort_by(&mut Option<(usize, SortDir)>) .selection(&mut Vec<bool>)", "Optional column sorting and per-row selection."],
      [".lazy_rows(count, id_of, cells_of)", "Rows made on demand: cells are asked for only for rows in view, so 100k rows cost what a screenful does."],
      [".height(px)", "Scroll the body under a sticky header. Only visible rows are built either way."],
    ],
    keys: [["↑ ↓", "Move row focus"], ["Page Up / Page Down", "Move a page"], ["Home / End", "First / last row"], ["Space / Enter", "Toggle focused header sort or row selection"]],
  },
  {
    slug: "tree", name: "Tree view", group: "Data", demo: "tree",
    summary: "Hierarchical explorer with rotating branch chevrons, vertical indent guides, a spring-sliding selection pill and full arrow-key navigation.",
    code: `let nodes = [
    TreeNode::branch("src", "src", [
        TreeNode::leaf("lib.rs", "lib.rs").icon("file-code").meta("1.4 KB"),
        TreeNode::leaf("theme.rs", "theme.rs").icon("file-code").meta("24.8 KB"),
    ]),
];
ui.tree("workspace", &nodes, &mut s.expanded, &mut s.selected).show();`,
    options: [
      ["TreeNode::branch(id, label, children)", "Collapsible folder node; switches folder to folder-open when expanded."],
      ["TreeNode::leaf(id, label)", ".icon(name) and .meta(text) for trailing file sizes or counts."],
    ],
    keys: [["↑ ↓ / Home / End", "Move selection across visible nodes"], ["→ / ←", "Expand or enter branch / collapse or jump to parent"], ["Space / Enter", "Toggle branch"]],
  },
  {
    slug: "timeline", name: "Timeline", group: "Data", demo: "timeline",
    summary: "Vertical deploy and activity feed with semantic status badges, connecting rails, commit tags and timestamps.",
    code: `let events = [
    TimelineItem::new("Promoted v0.4.2 to production", "2m ago")
        .icon("rocket")
        .tone(Tone::Success)
        .tag("9f3c1a2")
        .description("All 14 health checks green across Frankfurt and Virginia."),
];
ui.timeline("deploys", &events);`,
    options: [
      ["TimelineItem::new(title, time)", ".icon(name), .tone(Tone), .tag(mono_pill), .description(text)"],
    ],
    keys: [],
  },
  {
    slug: "code-block", name: "Code block", group: "Data", demo: "code-block",
    summary: "Syntax-highlighted monospace code viewer for Rust, TOML, JSON and shell snippets with line numbers, accent line highlights and Copy.",
    code: `ui.code_block("rollout", snippet)
    .title("examples/rollout.rs")
    .language("rust")
    .highlight_lines(&[4, 5])
    .show();`,
    options: [
      [".language(lang) .title(path)", "Built-in tokenizer colors keywords, types, strings, numbers, macros and comments via theme tokens."],
      [".line_numbers(bool) .highlight_lines(&[usize]) .copyable(bool)", "1-based highlighted lines get an accent gutter bar and subtle tint."],
    ],
    keys: [["Tab + Enter on Copy", "Copies the full snippet and confirms with a check"]],
  },
  {
    slug: "json-viewer", name: "JSON viewer", group: "Data", demo: "json-viewer",
    summary: "Zero-dependency JSON parser and interactive collapsible tree with key/item count previews, indent guides and typed syntax colors.",
    code: `ui.json_viewer("manifest", payload)
    .title("release-manifest.json")
    .default_depth(2)
    .show();`,
    options: [
      [".default_depth(usize)", "Initial expansion depth before collapsing nested objects and arrays into { n keys } / [ n items ]."],
      [".title(label)", "Header title next to the Copy button."],
    ],
    keys: [["Space / Enter", "Expand or collapse focused object or array"]],
  },
  {
    slug: "key-value", name: "Key–value editor", group: "Data", demo: "key-value",
    summary: "Editable table of environment variables or HTTP headers with per-row enable checkboxes, secret bullet masking, reveal toggle and row add/remove.",
    code: `ui.key_value_editor("env", &mut s.env_vars)
    .placeholders("VARIABLE_NAME", "value")
    .add_label("Add variable")
    .show();`,
    options: [
      ["KvPair::new(key, value)", ".secret() masks the value with bullets until revealed; .disabled_row() starts unchecked."],
      [".placeholders(key, val) .add_label(label)", "Custom placeholders and footer button copy."],
    ],
    keys: [["Tab", "Move across checkbox, key field, value field and row actions"]],
  },
  {
    slug: "log-viewer", name: "Log viewer", group: "Data", demo: "log-viewer",
    summary: "Structured live console with Info / Warn / Error filter chips, instant fuzzy search highlighting and click-to-copy log lines.",
    code: `ui.log_viewer("console", &logs)
    .height(210.0)
    .show();`,
    options: [
      ["LogEntry::new(time, LogLevel, target, message)", "LogLevel::Info, Warn, or Error."],
      [".height(px)", "Viewport height; overflows scroll with an overlay thumb."],
    ],
    keys: [["Escape in filter", "Clear active search query"], ["Click log line", "Copy formatted log entry to clipboard"]],
  },
  {
    slug: "progress-ring", name: "Progress and activity rings", group: "Charts", demo: "progress-ring",
    summary: "Circular progress arcs and concentric Apple-style activity rings with rounded endcaps that sweep into place on the smooth spring.",
    code: `let rings = [
    RingSpec::new("Compute", 0.84, "67.2 / 80 vCPU"),
    RingSpec::new("Memory", 0.62, "99.2 / 160 GB"),
    RingSpec::new("SLO budget", 0.94, "99.96% uptime"),
];
ui.activity_rings("cluster", &rings, 132.0);
ui.progress_ring("cache_hit", 0.92).size(104.0).label("Cache hit").show();`,
    options: [
      ["ui.progress_ring(key, 0.0..=1.0)", ".size(px), .thickness(px), .label(text), .color(Color)"],
      ["ui.activity_rings(key, &[RingSpec], size)", "Concentric rings paired with a tabular series legend."],
    ],
    keys: [],
  },
  {
    slug: "bar-list", name: "Bar list", group: "Charts", demo: "bar-list",
    summary: "Ranked horizontal proportion bars for top routes, referrers or resource consumers with tabular counts and shares.",
    code: `let routes = [
    BarListItem::new("/v1/chat/completions", 48200.0),
    BarListItem::new("/v1/embeddings", 29400.0),
];
ui.bar_list("routes", &routes);`,
    options: [
      ["BarListItem::new(label, value)", ".formatted(text) overrides compact number formatting."],
    ],
    keys: [],
  },
  {
    slug: "heatmap", name: "Heatmap", group: "Charts", demo: "heatmap",
    summary: "2D density and contribution matrix colored across a perceptual OKLab accent ramp with row/column labels and hover tooltips.",
    code: `ui.heatmap("req_heat", 5, 12, &values)
    .row_labels(&["Mon", "Tue", "Wed", "Thu", "Fri"])
    .col_labels(&["00h", "02h", "04h", "06h", "08h", "10h", "12h", "14h", "16h", "18h", "20h", "22h"])
    .unit("req/s")
    .show();`,
    options: [
      ["ui.heatmap(key, rows, cols, &[f32])", ".row_labels(&[&str]), .col_labels(&[&str]), .unit(text)"],
    ],
    keys: [],
  },
  {
    slug: "radar-chart", name: "Radar chart", group: "Charts", demo: "radar-chart",
    summary: "Multi-axis polygon spider chart with concentric web rings, translucent mesh fills, vertex springs and axis tooltips.",
    code: `let axes = ["Throughput", "Latency", "Memory", "Startup", "Binary size", "Battery"];
ui.radar_chart("bench", &axes, vec![
    Series::new("lumen 0.4", vec![94.0, 92.0, 88.0, 95.0, 86.0, 90.0]),
    Series::new("WebView", vec![58.0, 52.0, 44.0, 49.0, 62.0, 55.0]),
]).show();`,
    options: [
      [".max_value(f32) .height(px)", "Axis maximum (default 100.0) and chart height."],
    ],
    keys: [],
  },
  {
    slug: "scatter-chart", name: "Scatter chart", group: "Charts", demo: "scatter-chart",
    summary: "2D scatter and bubble plot with nice X/Y ticks, dashed gridlines, series legend and nearest-point crosshair tooltips.",
    code: `let metal = ScatterSeries::new("Metal ARM64", vec![
    ScatterPoint::bubble(1.8, 142.0, 7.5, "M3 Max"),
    ScatterPoint::bubble(2.4, 118.0, 6.5, "M2 Pro"),
]);
ui.scatter_chart("gpu", vec![metal]).height(240.0).show();`,
    options: [
      ["ScatterPoint::new(x, y) / ::bubble(x, y, r, label)", "Fixed-radius point or named bubble."],
      ["ScatterSeries::new(name, points)", "Automatically colored from Series0..5."],
    ],
    keys: [],
  },
  {
    slug: "funnel-chart", name: "Funnel chart", group: "Charts", demo: "funnel-chart",
    summary: "Centered tapering conversion trapezoids with stage counts, cumulative conversion percentages and step-retention tooltips.",
    code: `let stages = [
    ("Visited docs", 18400.0),
    ("Ran quickstart", 11800.0),
    ("Built first view", 6400.0),
    ("Shipped release", 2950.0),
];
ui.funnel_chart("activation", &stages).height(205.0).show();`,
    options: [
      ["ui.funnel_chart(key, &[(&str, f32)])", ".height(px) sets total funnel height."],
    ],
    keys: [],
  },
  {
    slug: "candlestick", name: "Candlestick chart", group: "Charts", demo: "candlestick",
    summary: "OHLC financial and latency range chart with semantic Success/Danger wicks and bodies, nice Y ticks and crosshair readouts.",
    code: `let candles = [
    Candle::new("Nov 1", 142.0, 148.5, 139.0, 146.2),
    Candle::new("Nov 2", 146.2, 151.0, 144.5, 149.8),
];
ui.candlestick_chart("ohlc", &candles).height(250.0).show();`,
    options: [
      ["Candle::new(label, open, high, low, close)", "Bullish bars use Success; bearish bars use Danger."],
    ],
    keys: [],
  },
  {
    slug: "kanban", name: "Kanban board", group: "Planning", demo: "kanban",
    summary: "Multi-column status board with live column counts, semantic tags, avatars and pointer or arrow-key card moves.",
    code: `let cols = [
    KanbanCol::new("Backlog", Tone::Neutral),
    KanbanCol::new("In progress", Tone::Accent),
    KanbanCol::new("Shipped", Tone::Success),
];
ui.kanban("sprint", &cols, &mut s.cards);`,
    options: [
      ["KanbanCol::new(title, Tone)", "Column header with status dot and live card counter."],
      ["KanbanCard::new(id, col, title)", ".tag(label, Tone), .meta(id_text), .assignee(initials)"],
    ],
    keys: [["Tab", "Focus card"], ["← →", "Move focused card to previous or next column"]],
  },
  {
    slug: "ai-reasoning", name: "AI reasoning and tools", group: "AI", demo: "ai-reasoning",
    summary: "Collapsible chain-of-thought trace with an accent rail and structured tool-call status rows.",
    code: `ui.reasoning_block("cot", "Thought for 2.4s", Some("3 steps"), &mut s.open, |ui| {
    ui.label("Checked physical pixel snapping in Painter::snap_rect.").secondary();
    ui.tool_call("tc1", &ToolCall::new("read_file", "layout.rs:240-310", ToolStatus::Done).duration("18 ms"));
});`,
    options: [
      ["ui.reasoning_block(key, summary, duration, &mut open, |ui| ..)", "Rotating chevron + left accent guide bar."],
      ["ToolCall::new(name, args, ToolStatus)", "ToolStatus::Running, Done, or Error; .duration(text)."],
    ],
    keys: [["Space / Enter", "Expand or collapse reasoning trace"]],
  },
  {
    slug: "ai-composer", name: "Prompt composer", group: "AI", demo: "ai-composer",
    summary: "Multi-line auto-growing AI prompt box with removable attachment chips, model selector menu, context-window ring and Send / Stop button.",
    code: `let out = ui.prompt_composer("composer", &mut s.prompt, &mut s.model)
    .models(&["Claude 3.7 Sonnet", "Gemini 2.5 Pro", "o3-mini"])
    .attachments(&mut s.files)
    .context_used(0.34)
    .streaming(s.streaming)
    .show();`,
    options: [
      [".models(&[&str]) .attachments(&mut Vec<String>)", "Dropdown model selector and removable file chips."],
      [".context_used(0.0..=1.0) .streaming(bool)", "Mini arc gauge and Send-to-Stop state switch."],
      ["ComposerOut", "{ submitted, stopped, attach_clicked }"],
    ],
    keys: [["⌘/Ctrl + Enter", "Submit prompt"], ["Enter", "Insert newline"]],
  },
  {
    slug: "ai-chat", name: "AI chat thread", group: "AI", demo: "ai-chat",
    summary: "User and assistant conversation feed with numbered source citation pills and timestamps.",
    code: `let msgs = [
    ChatMessage::user("Why does lumen draw zero frames when idle?"),
    ChatMessage::assistant("lumen", "Springs settle analytically and park the thread.")
        .citations(&["anim.rs:84", "ui.rs:643"]),
];
ui.chat_thread("chat", &msgs);`,
    options: [
      ["ChatMessage::user(body) / ::assistant(author, body)", ".citations(&[&str]), .time(text)"],
    ],
    keys: [],
  },
  {
    slug: "diff-viewer", name: "Diff viewer", group: "Developer tools", demo: "diff-viewer",
    summary: "Unified diff card with file header, +added / -removed counters, hunk markers and dual old/new line numbers.",
    code: `ui.diff_viewer("patch", "crates/lumen-core/src/ui.rs", diff_str).show();`,
    options: [
      ["ui.diff_viewer(key, path, unified_diff)", "Colors + and - lines with Success and Danger subtle tints."],
    ],
    keys: [],
  },
  {
    slug: "waterfall", name: "Trace waterfall", group: "Developer tools", demo: "waterfall",
    summary: "Distributed trace and network span timeline with millisecond ruler ticks, nested span labels and hover timing tooltips.",
    code: `let spans = [
    TraceSpan::new("POST /v1/chat/completions", "edge-proxy", 0.0, 148.0, 0),
    TraceSpan::new("verify_session_jwt", "auth-svc", 3.5, 14.2, 1),
];
ui.waterfall("trace", &spans);`,
    options: [
      ["TraceSpan::new(name, service, start_ms, duration_ms, depth)", ".error() highlights failed spans in Danger."],
    ],
    keys: [],
  },
  {
    slug: "uptime-bars", name: "Uptime status", group: "Developer tools", demo: "uptime-bars",
    summary: "Service availability strip showing daily Operational, Degraded and Outage bars with overall uptime percentage and hover tooltips.",
    code: `ui.uptime_bars("api", "Inference API · eu-central", &statuses);`,
    options: [
      ["UptimeStatus", "::Operational, ::Degraded, ::Outage"],
    ],
    keys: [],
  },
  {
    slug: "rolling-number", name: "Rolling number", group: "Motion", demo: "rolling-number",
    summary: "Per-digit vertical odometer counter where each place-value column spins smoothly to its new digit and parks at zero wakeups.",
    code: `ui.rolling_number("mrr", s.mrr)
    .prefix("$")
    .suffix("USD")
    .role(TextRole::Display)
    .show();`,
    options: [
      [".prefix(t) .suffix(t) .role(TextRole)", "Automatic comma grouping and tabular digit columns."],
    ],
    keys: [],
  },
  {
    slug: "dock", name: "Magnifying dock", group: "Motion", demo: "dock",
    summary: "Floating glass icon dock whose tiles magnify along a Gaussian neighbor curve on bouncy springs as the pointer glides across.",
    code: `let items = [
    DockItem::new("compass", "Explorer").active(true),
    DockItem::new("terminal", "Terminal"),
    DockItem::new("code", "Editor"),
];
if let Some(i) = ui.dock("app_dock", &items) {
    open_app(i);
}`,
    options: [
      ["DockItem::new(icon, label).active(bool)", "Active apps draw an accent dot beneath their tile."],
    ],
    keys: [["Tab + Space / Enter", "Focus and launch dock item"]],
  },
  {
    slug: "waveform", name: "Audio waveform", group: "Motion", demo: "waveform",
    summary: "Scrubbable voice and audio bar spectrum with circular play/pause button, played/unplayed split and tabular mm:ss readout.",
    code: `ui.waveform("voice_note", &samples, &mut s.progress, &mut s.playing)
    .title("Architecture walkthrough")
    .duration(42.0)
    .show();`,
    options: [
      [".title(text) .duration(secs)", "Click or drag across bars to scrub playback position."],
    ],
    keys: [["Space", "Toggle play / pause"], ["← →", "Scrub backward or forward 5%"]],
  },
  {
    slug: "window-chrome", name: "Window chrome", group: "Layout and surfaces", demo: "window-chrome",
    summary: "A title bar that follows the look: macOS traffic lights, Windows 11 caption buttons, GNOME headerbar buttons or Lumen pills. As a preview card it reports clicks; with .window() it is your app's real title bar and moves, zooms, minimizes and closes the window.",
    code: `// Your app's own title bar (the OS one is hidden):
lumen::App::new("Notes").custom_chrome().run(state, |ui, s| {
    ui.window_chrome("main", "Notes").window().show(|ui| {
        notes_view(ui, s);
    });
});

// Or ask before closing:
let r = ui.window_chrome("main", "Notes").window().handle_buttons(false).show(body).response;
if r.close && confirm_quit(ui) { ui.request_window(WindowRequest::Close); }`,
    options: [
      [".window()", "Be the real title bar: fills the window, registers drag and button regions for the host, sends window requests."],
      [".handle_buttons(false)", "Report clicks in ChromeResponse (close, minimize, zoom, drag_started, double_clicked) without acting on them."],
      [".closable(b) .minimizable(b) .maximizable(b)", "Choose the buttons. Maximize shows restore while the window is maximized."],
      [".subtitle(text)", "A muted caption after the title. Inactive windows dim the title and, on macOS, gray the lights."],
      ["App::custom_chrome()", "lumen-app: hides the OS title bar (transparent on macOS, so corners and shadow stay), moves on drag, follows the OS double-click setting, and resizes from the edges on Windows and Linux."],
    ],
    keys: [],
  },
  {
    slug: "split-pane", name: "Split pane", group: "Layout and surfaces", demo: "split-pane",
    summary: "Draggable horizontal or vertical split view with an animated accent sash, double-click 50/50 reset and arrow-key resizing.",
    code: `ui.split_pane("ide_split", &mut s.ratio)
    .height(220.0)
    .show(|ui| { ui.label("Explorer"); }, |ui| { ui.label("Inspector"); });`,
    options: [
      [".vertical() .height(px)", "Horizontal left/right by default; .vertical() stacks top/bottom."],
    ],
    keys: [["← → / ↑ ↓ (Shift 2.5×)", "Resize focused sash"], ["Double-click / Home", "Spring back to 50 / 50"]],
  },
  {
    slug: "menu-bar", name: "Menu bar", group: "Navigation", demo: "menu-bar",
    summary: "Desktop application menu bar (File, Edit, View, Build) where hovering across menus while one is open switches menus seamlessly.",
    code: `ui.menu_bar("app_bar", |bar| {
    bar.menu("File", |ui| {
        ui.menu_item("plus", "New workspace").shortcut("⌘N");
    });
    bar.menu("Edit", |ui| {
        ui.menu_item("copy", "Copy").shortcut("⌘C");
    });
});`,
    options: [
      ["bar.menu(label, |ui| ..)", "Uses standard ui.menu_item, ui.menu_separator and ui.menu_label."],
    ],
    keys: [["← →", "Move across top-level triggers"], ["Space / Enter", "Open focused menu"]],
  },
  {
    slug: "terminal", name: "Terminal", group: "Developer tools", demo: "terminal",
    summary: "Interactive lumen-drawn command shell with semantic output tones, scrollable buffer, live prompt and ↑ / ↓ command history.",
    code: `if let Some(cmd) = ui.terminal("shell", &s.lines, &mut s.input)
    .title("zsh — lumen")
    .height(196.0)
    .show()
{
    run_command(&cmd, &mut s.lines);
}`,
    options: [
      ["TermLine", "::cmd, ::out, ::ok, ::warn, ::err"],
      [".title(text) .height(px)", "Header title and scrollable viewport height."],
    ],
    keys: [["Enter", "Submit command"], ["↑ ↓", "Cycle command history"]],
  },
  {
    slug: "gantt", name: "Gantt chart", group: "Planning", demo: "gantt",
    summary: "Roadmap and sprint timeline with completion-filled task bars, diamond milestones, owner tags and hover tooltips.",
    code: `let tasks = [
    GanttTask::new("Token & OKLab engine", 0.0, 1.8, 1.0).owner("JB").tone(Tone::Success),
    GanttTask::milestone("v0.5 RC freeze", 5.6),
];
ui.gantt_chart("roadmap", &["W1", "W2", "W3", "W4", "W5", "W6"], &tasks);`,
    options: [
      ["GanttTask::new(name, start, duration, progress)", ".owner(initials), .tone(Tone)"],
      ["GanttTask::milestone(name, at)", "Draws a diamond milestone marker at column coordinate at."],
    ],
    keys: [],
  },
  {
    slug: "treemap", name: "Treemap", group: "Charts", demo: "treemap",
    summary: "Slice-and-dice proportional area tiles for binary size, memory or storage breakdowns with share labels and hover tooltips.",
    code: `let items = [
    TreemapItem::new("lumen-core", 342.0).detail("Widgets & layout"),
    TreemapItem::new("cosmic-text", 218.0),
];
ui.treemap("wasm_size", &items).unit("KB").height(215.0).show();`,
    options: [
      ["TreemapItem::new(label, value).detail(text)", "Sorted largest-first and partitioned recursively by aspect ratio."],
    ],
    keys: [],
  },
  {
    slug: "markdown", name: "Markdown view", group: "Data", demo: "markdown",
    summary: "Zero-dependency immediate-mode Markdown renderer supporting headings, bold, inline code pills, lists, blockquotes and fenced code blocks.",
    code: `ui.markdown("doc", "# Heading\\nParagraph with \`inline_code\` and **bold**.");`,
    options: [
      ["ui.markdown(key, md_str)", "Delegates fenced code blocks to ui.code_block with syntax highlighting."],
    ],
    keys: [],
  },
  {
    slug: "property-grid", name: "Property inspector", group: "Developer tools", demo: "property-grid",
    summary: "Compact two-column Figma/Xcode-style inspector card with section headers, scrubbable X/Y numeric pairs and toggles.",
    code: `ui.property_inspector("props", |insp| {
    insp.section("Transform");
    insp.vec2_row("Position", &mut s.x, &mut s.y);
    insp.number("Opacity", &mut s.opacity, 0.0..=100.0, "%");
    insp.toggle("Clip sublayers", &mut s.clip);
});`,
    options: [
      ["insp.section(title) .row(label, |ui| ..)", "Custom or built-in .number, .vec2_row and .toggle rows."],
    ],
    keys: [["Drag field / ↑ ↓", "Scrub or step numeric properties"]],
  },
  {
    slug: "flame-graph", name: "Flame graph", group: "Developer tools", demo: "flame-graph",
    summary: "Interactive CPU and GPU icicle flame chart — click any stack frame to spring-zoom into its subtree and Reset zoom to pull back.",
    code: `let spans = [
    FlameSpan::new("Ui::run_frame", 0, 0.0, 4.80),
    FlameSpan::new("paint::tessellate", 1, 1.55, 1.92),
];
ui.flame_graph("profile", &spans);`,
    options: [
      ["FlameSpan::new(name, depth, start_ms, duration_ms)", "Click any span to zoom the window smoothly; hover for duration and % share."],
    ],
    keys: [],
  },
  {
    slug: "list-view", name: "List view", group: "Data", demo: "list-view",
    summary: "The master list of a mail, notes, music or files app. One highlight glides between rows, rows rise in sequence on first show, unread dots pop, and stars spring in.",
    code: `let rows = vec![
    ListRow::new("Brand guidelines.pdf").subtitle("PDF · 6.2 MB").icon("file-text").meta("Yesterday").starred(true),
    ListRow::new("Q4 planning.key").subtitle("Keynote").avatar("PN", 2).unread(true),
];
let out = ui.list_view("files", &rows, &mut selected); // selected: Option<usize>
if let Some(i) = out.activated { open(i) }            // double click or Enter`,
    options: [
      ["ListRow::new(title) .subtitle .preview .meta .icon .avatar(initials, tone) .unread .starred .badge(text, tone)", "Every field but the title is optional."],
      ["ListOut { changed, activated, starred, built }", "activated fires on double click or Enter; starred when a row's star is clicked; built is how many rows were built this frame."],
      ["ListRow::key(id)", "A stable id: rows inserted above the viewport don't move what you're reading."],
      ["Inside ui.scroll(..)", "Only the rows in view are built (Ui::virtual_rows), so 100k rows scroll at 60 fps."],
    ],
    keys: [["↑ ↓", "Move the selection"], ["Page Up / Page Down", "Move a page"], ["Home / End", "First / last row"], ["Type", "Jump to the next title that starts with it"], ["Enter", "Open the selected row"]],
  },
  {
    slug: "virtual-list", name: "Virtualized rows", group: "Data", demo: "virtual-list",
    summary: "100,000 rows of different heights, of which only the twenty or so in view are built. Heights are measured as rows appear, the scrollbar is right from the start, and loading history above keeps your place.",
    code: `ui.scroll("log").show(|ui| {
    let key = |i: usize| lines[i].id;
    ui.virtual_rows("lines", lines.len()).keys(&key).estimate(28.0).show(|ui, i| {
        log_line(ui, &lines[i]);
    });
});`,
    options: [
      [".estimate(px) .gap(px)", "Height of a row before it's measured (unseen rows follow the measured average), and space between rows."],
      [".keys(&|i| id)", "Stable row ids: measured heights and the scroll position follow rows across inserts, removals and sorts."],
      [".pin(Some(i)) .reveal(Some(i))", "Always build a row (the focused one), and scroll a row into view (keyboard navigation)."],
      ["VirtualOut { first, end, built, total_height }", "What was built this frame. list_view and table use this underneath."],
    ],
    keys: [],
  },
  {
    slug: "code-editor", name: "Code editor", group: "Developer tools", demo: "code-editor",
    summary: "A syntax-colored editor with a line-number gutter. The current-line band and the caret glide between lines, matching brackets light up, Enter keeps the indent, Tab indents and ⌘/ toggles comments. Undo, multi-line paste and IME work.",
    code: `let r = ui.code_editor("main", &mut source).lang("rust").rows(14).show();
if r.changed { recompile(&source) }`,
    options: [
      [".lang(\"rust\" | \"wgsl\" | \"toml\" | \"json\" | \"sh\" | \"js\" | \"py\")", "Syntax colors come from the theme's tokens, so they follow every look and accent."],
      [".rows(n)", "Visible lines; longer code scrolls, following the caret."],
    ],
    keys: [["Tab / ⇧Tab", "Indent / outdent (Esc, then Tab leaves)"], ["⌘ /", "Toggle line comments"], ["Home", "Indent, then column 0"], ["⌘ C / ⌘ X", "Copy or cut the line when nothing is selected"], ["⌘ Z / ⇧⌘ Z", "Undo / redo"]],
  },
  {
    slug: "file-browser", name: "File browser and path bar", group: "Data", demo: "file-browser",
    summary: "A folder listing with a breadcrumb path bar. Folders stay first, headers sort with an arrow that turns, the selection glides, rows rise in when the folder changes, and typing jumps to a name.",
    code: `let out = ui.file_browser("files", &path, &entries, &mut selected);
if let Some(i) = out.opened { if entries[i].folder { path.push(entries[i].name.clone()) } }
if let Some(k) = out.navigate { path.truncate(k + 1) }
if out.up { path.pop(); }
// Or just the breadcrumb:
if let Some(k) = ui.path_bar("crumbs", &["lumen", "crates", "lumen-core"]) { go_to(k) }`,
    options: [
      ["FileEntry::folder(name, modified, key) / ::file(name, bytes, modified, key)", "Icons follow the kind and extension unless you set .icon."],
      ["FileOut { changed, opened, navigate, up }", "Indices refer to your entries, whatever the sort."],
    ],
    keys: [["↑ ↓", "Move the selection"], ["Enter", "Open"], ["⌫ / ⌘↑", "Parent folder"], ["Letters", "Jump to a matching name"]],
  },
  {
    slug: "emoji-picker", name: "Emoji picker", group: "Inputs", demo: "emoji-picker",
    summary: "Search by name, switch categories under a sliding pill, and pick from a grid whose cells pop on hover. Recents come first, and a preview names the emoji under the pointer. Color emoji come from the system font.",
    code: `if let Some(e) = ui.emoji_picker("reactions") {
    message.push_str(e);
}`,
    options: [["→ Option<&'static str>", "The emoji picked this frame; recents are remembered per picker."]],
    keys: [["↓ from search", "Into the grid"], ["Arrows", "Move in the grid"], ["Enter", "Pick"], ["← → on categories", "Switch category"]],
  },
  {
    slug: "wheel-picker", name: "Wheel and time picker", group: "Selection", demo: "wheel-picker",
    summary: "A drum of values: drag it 1:1, throw it and it keeps its momentum before springing onto an item, or scroll a notch at a time. Items curve away and fade from the center band. time_wheel pairs two for hours and minutes.",
    code: `ui.wheel_picker("size", &mut size, &["S", "M", "L", "XL"]);
ui.time_wheel("alarm", &mut time, 5); // Time { hour, minute }, 5-minute steps`,
    options: [["time_wheel(key, &mut Time, minute_step)", "24-hour; returns true when the time changed."]],
    keys: [["↑ ↓", "Previous / next"], ["PageUp / PageDown", "Five at a time"], ["Home / End", "First / last"]],
  },
  {
    slug: "signature-pad", name: "Signature pad", group: "Inputs", demo: "signature-pad",
    summary: "Sign with a mouse, pen or finger. Strokes are smoothed, thin out as the pen speeds up, and land in accent ink that dries to the foreground. Undo and Clear fade strokes away, and a check pops in once it's signed. Points are stored in pad units, so a signature survives a resize.",
    code: `let mut sig = Signature::default();
ui.signature_pad("Signature", &mut sig).show();
let can_submit = !sig.is_empty();`,
    options: [["height(px)", "Height of the drawing area."], ["hint(text)", "Shown under the line while empty."], ["Signature { strokes }", "Vec of strokes of SigPoint { x, y, w } in 0..1 pad units; undo(), clear(), is_empty()."]],
    keys: [["Backspace", "Undo the last stroke"], ["Cmd/Ctrl + Backspace", "Clear"], ["Cmd/Ctrl + Z", "Undo"]],
  },
  {
    slug: "avatar-upload", name: "Avatar upload", group: "Inputs", demo: "avatar-upload",
    summary: "A round photo slot with its actions beside it. Empty, a dashed ring and camera turn toward the accent on hover; uploading, the photo shrinks inside a progress ring; done, a check pops in; failed, the ring turns red and Retry appears. Your app owns the file dialog and the upload: the widget reports choose, retry and remove.",
    code: `let out = ui.avatar_upload("photo", &state).initials("AL").title("Profile photo").hint("PNG or JPG").show();
if out.choose { open_file_dialog(); }
if out.remove { state = UploadState::Empty; }`,
    options: [["UploadState", "Empty, Uploading(0..1), Done or Failed(reason)."], ["AvatarOut", "choose, remove (also cancels an upload) and retry."], ["initials / tone / size", "What the photo shows and how big it is."]],
    keys: [["Enter / Space on the photo", "Choose, change or retry"]],
  },
  {
    slug: "time-dial", name: "Time dial", group: "Dates", demo: "time-dial",
    summary: "A clock face: drag or tap the hand to an hour, and the face turns into minutes. The hand springs the short way round, numerals crossfade between hours and minutes, and the chosen number sits inside the accent knob. Hour and minute tiles switch back and forth, with AM and PM beside them.",
    code: `ui.time_dial("alarm", &mut time).step(5).show(); // Time { hour, minute }`,
    options: [["step(minutes)", "Minutes snap to this step (1 to 30, default 5)."], ["diameter(px)", "Size of the face; defaults to seven control heights."], ["→ Response", "changed is true when the time changed."]],
    keys: [["↑ → / ↓ ←", "Next / previous hour or minute"], ["PageUp / PageDown", "Five at a time"], ["Enter / Space on the dial", "Switch between hours and minutes"]],
  },
  {
    slug: "aspect-ratio", name: "Aspect-ratio picker", group: "Selection", demo: "aspect-ratio",
    summary: "A crop preview over a sample photo and a row of ratio chips, each drawn in its own shape. The crop frame springs to the new shape, rule-of-thirds guides flash in while it moves, the selection pill glides between chips, and the orientation toggle turns every shape on its side.",
    code: `let ratios = [("Free", 0.0), ("1:1", 1.0), ("4:5", 0.8), ("16:9", 16.0 / 9.0)];
ui.aspect_ratio_picker("crop", &mut selected, &mut portrait, &ratios);`,
    options: [["ratios: &[(&str, f32)]", "Label and width / height; 0.0 is free."], ["portrait: &mut bool", "Turns fixed ratios on their side; the toggle is off for Free and 1:1."]],
    keys: [["← →", "Previous / next ratio"], ["Enter / Space", "Choose"]],
  },
  {
    slug: "ruler-slider", name: "Ruler slider", group: "Selection", demo: "ruler-slider",
    summary: "A tick ruler that slides under a fixed needle, the way camera and scale apps pick a value. Drag it 1:1 or throw it and it springs onto a tick; ticks near the needle grow, the edges fade, the needle stretches while held, and the readout rolls the way the value moved.",
    code: `ui.ruler_slider("Weight", &mut kg, 30.0..=200.0).step(0.1).major(10).unit("kg").show();`,
    options: [["step(v)", "Value between ticks."], ["major(n)", "A labelled long tick every n steps."], ["unit(text) / decimals(n)", "Readout formatting."]],
    keys: [["← → / ↓ ↑", "One tick"], ["Shift + arrows, PageUp / PageDown", "One major tick"], ["Home / End", "Ends of the range"]],
  },
  {
    slug: "calendar-heatmap", name: "Calendar heatmap", group: "Charts", demo: "calendar-heatmap",
    summary: "Month calendars whose days are tinted by a value on an accent ramp. Days fill in on first show, the hovered day lifts, the selection ring glides, and months wrap to fit.",
    code: `let changed = ui.calendar_heatmap("deploys", Date::new(2026, 10, 1), 2, &per_day, "deploys", &mut day);`,
    options: [["values: &[(Date, f32)]", "Days not listed count as 0; five levels from the largest value."], ["unit", "Used in the readout, such as \"Nov 14 · 12 deploys\"."]],
    keys: [["← →", "Previous / next day"], ["↑ ↓", "Week"], ["Home / End", "Month start / end"], ["PageUp / PageDown", "Month"]],
  },
  {
    slug: "tour", name: "Guided tour", group: "Overlays", demo: "tour",
    summary: "A modal walkthrough: the window dims, a rounded spotlight cuts out one widget at a time and glides to the next, and a card explains it with step dots, Back, Next and Skip.",
    code: `let steps = [
    TourStep::new(search.id, "Find anything", "Search projects, people and settings."),
    TourStep::new(new_btn.id, "Start a project", "Pick a template or import a repository."),
];
if ui.button("Take the tour").clicked() { step = Some(0) }
ui.tour("onboarding", &steps, &mut step); // step: Option<usize>`,
    options: [["TourStep::new(anchor_id, title, body)", "Anchors are any widget's Response::id."]],
    keys: [["→ / Enter", "Next"], ["←", "Back"], ["Esc", "End the tour"]],
  },
  {
    slug: "notification-list", name: "Notification inbox", group: "Feedback", demo: "notification-list",
    summary: "Filterable All / Unread notification feed with unread accent dots, Mark all read button, avatars and per-item archive.",
    code: `ui.notification_list("inbox", &mut s.notifications);`,
    options: [
      ["NotificationItem::new(id, author, title, body, time, unread)", "Clicking a row toggles read state; × archives it."],
    ],
    keys: [["Tab + Space", "Filter chips, Mark all read and row toggle"]],
  },
  {
    slug: "spotlight-card", name: "Spotlight card", group: "Motion", demo: "spotlight-card",
    summary: "Signature Ethereal glass card whose border and surface bloom with a soft OKLab radial accent spotlight tracking the pointer.",
    code: `ui.spotlight_card("gpu")
    .icon("zap")
    .title("Single-pass SDF")
    .show(|ui| {
        ui.label("Rounded rects, borders and glyphs batch into one draw call.").secondary();
    });`,
    options: [
      [".icon(name) .title(text) .show(|ui| ..)", "Suppresses radial pointer tracking automatically when reduced motion is enabled."],
    ],
    keys: [],
  },
  {
    slug: "editable-text", name: "Editable text", group: "Inputs", demo: "editable-text",
    summary: "Click-to-edit inline typography or heading that morphs seamlessly into an inline text editor on click or Enter, with Escape to revert.",
    code: `ui.editable_text("title", &mut s.heading).heading();
ui.editable_text("summary", &mut s.summary)
    .role(TextRole::Body)
    .placeholder("Add a description…");`,
    options: [
      [".heading() .role(TextRole)", "Typography role used in both rest and active inline-editing states."],
      [".placeholder(text) .disabled(bool)", "Fallback text shown when empty, and read-only lock."],
    ],
    keys: [["Enter / Space", "Begin editing or commit changes"], ["Escape", "Revert to the value before editing began"]],
  },
  {
    slug: "money-field", name: "Money field", group: "Inputs", demo: "money-field",
    summary: "Formatted currency input with live thousands separators, tabular digits, step buttons, and an optional cycleable currency pill (USD, EUR, GBP, JPY).",
    code: `ui.money_field(&mut s.budget)
    .label("Monthly cluster budget")
    .switchable_currency(&mut s.currency)
    .step(100.0)
    .fill_width();`,
    options: [
      [".currency(Currency) .switchable_currency(&mut Currency)", "Fixed or interactive USD / EUR / GBP / JPY symbol & decimal precision."],
      [".range(min, max) .step(f64) .label(text) .helper(text)", "Bounds, increment step and field captions."],
    ],
    keys: [["↑ / ↓", "Step amount up or down"], ["Enter", "Re-apply thousands grouping"]],
  },
  {
    slug: "masked-input", name: "Masked input", group: "Inputs", demo: "masked-input",
    summary: "Pattern-guided input (# = digit, A = letter, * = alphanumeric) that auto-inserts separators and keeps remaining template slots visible.",
    code: `ui.masked_input(&mut s.phone, "+1 (###) ###-####")
    .label("On-call pager")
    .icon("phone")
    .fill_width();
ui.masked_input(&mut s.key, "AAAA-####-AAAA").icon("key-round");`,
    options: [
      ["# / A / * / literals", "# matches ASCII digits, A uppercases letters, * matches alphanumerics; literals auto-insert."],
      [".label(text) .helper(text) .icon(name) .fill_width()", "Standard field chrome and leading icon."],
    ],
    keys: [["Backspace", "Steps cleanly across literal separators"]],
  },
  {
    slug: "mention-input", name: "Mention input", group: "Inputs", demo: "mention-input",
    summary: "Inline @mention composer: typing @ opens a floating teammate picker filtered live by handle or name, with recipient pills below.",
    code: `let team = [
    MentionCandidate::new("lena", "Lena Fischer", "Graphics"),
    MentionCandidate::new("jonas", "Jonas Berg", "Runtime"),
];
ui.mention_input(&mut s.note, &team).label("Review comment");`,
    options: [
      ["MentionCandidate::new(handle, name, role)", "Candidate shown with avatar, @handle and role inside the anchored popover."],
      [".label(text) .placeholder(text)", "Field label and placeholder hint."],
    ],
    keys: [["@", "Open candidate popover"], ["↑ / ↓ + Enter", "Navigate and insert @handle"], ["Escape", "Dismiss popover"]],
  },
  {
    slug: "step-slider", name: "Step slider", group: "Selection", demo: "step-slider",
    summary: "Discrete snapped slider over named tick stops, with spring thumb motion, press squash, cutout tick dots, and an active value badge.",
    code: `const TIERS: [&str; 5] = ["512 MB", "2 GB", "8 GB", "32 GB", "128 GB"];
ui.step_slider("Worker memory tier", &mut s.tier_idx, &TIERS);`,
    options: [
      ["ui.step_slider(label, &mut index, stops)", "Snaps pointer drags and clicks to discrete named stops."],
      [".disabled(bool) .changed()", "Disable interaction or inspect whether the stop index changed."],
    ],
    keys: [["← / →", "Previous / next stop"], ["Home / End", "Jump to first or last stop"]],
  },
  {
    slug: "transfer-list", name: "Transfer list", group: "Selection", demo: "transfer-list",
    summary: "Dual-pane shuttle for moving permissions, columns or items between Available and Assigned sets, with search filter and batch controls.",
    code: `ui.transfer_list("rbac", &mut s.available, &mut s.assigned)
    .titles("Available scopes", "Assigned to role")
    .height(160.0);`,
    options: [
      [".titles(left, right) .height(px)", "Custom pane headers with live selected/total badges and scroll viewport height."],
    ],
    keys: [["Tab + Space", "Toggle item checkboxes or trigger > / >> / < / << shuttle buttons"]],
  },
  {
    slug: "reorderable-list", name: "Reorderable list", group: "Data", demo: "reorderable-list",
    summary: "Sortable priority list with drag-grip pointer reordering, keyboard ArrowUp/ArrowDown swaps, and step buttons.",
    code: `ui.reorderable_list("stages", &mut s.stages);`,
    options: [
      ["ReorderItem::new(id, title).subtitle(..).badge(..)", "Row model with stable key, secondary subtitle and right-aligned pill."],
    ],
    keys: [["↑ / ↓", "Swap focused item up or down"], ["Pointer drag", "Drag row across neighbor threshold to swap"]],
  },
  {
    slug: "editor-tabs", name: "Editor tabs", group: "Navigation", demo: "editor-tabs",
    summary: "Closable IDE and browser tab strip with file icons, unsaved dot indicator (•), hover close button (×), + new tab action, and a spring-gliding active card.",
    code: `let out = ui.editor_tabs("files", &s.tabs, &mut s.active);
if let Some(i) = out.closed { s.tabs.remove(i); }
if out.add_clicked { /* open new tab */ }`,
    options: [
      ["EditorTab::new(id, label).icon(name).dirty(bool)", "Shows a warning dot when dirty that reveals × on hover."],
      ["EditorTabsOut { changed, closed, add_clicked }", "Reports selection changes, closed tab index and + clicks."],
    ],
    keys: [["← / →", "Cycle active tab when focused"]],
  },
  {
    slug: "table-of-contents", name: "Table of contents", group: "Navigation", demo: "table-of-contents",
    summary: "Vertical “On this page” anchor rail with nested section indentation and a spring-sliding vertical accent bar.",
    code: `let items = [
    TocItem::new("Architecture", 0),
    TocItem::new("Unified WGSL shader", 1),
    TocItem::new("Spring physics", 0),
];
ui.table_of_contents("toc", &items, &mut s.section);`,
    options: [
      ["TocItem::new(label, depth)", "Depth indents sub-headings while keeping one continuous left rail."],
    ],
    keys: [["↑ / ↓", "Move active section up or down"]],
  },
  {
    slug: "nav-bar", name: "Navigation bar", group: "Navigation", demo: "nav-bar",
    summary: "Top horizontal application navigation bar with a spring-gliding active pill, Lucide icons, and notification count badges.",
    code: `let items = [
    NavBarItem::new("Overview").icon("layout-dashboard"),
    NavBarItem::new("Deployments").icon("rocket").badge("3"),
];
ui.nav_bar("main_nav", &items, &mut s.section);`,
    options: [
      ["NavBarItem::new(label).icon(name).badge(text)", "Horizontal nav destination with optional leading icon and count pill."],
    ],
    keys: [["← / →", "Switch active destination"]],
  },
  {
    slug: "carousel", name: "Carousel", group: "Layout and surfaces", demo: "carousel",
    summary: "Spring-sliding horizontal feature and media deck with previous/next controls, slide counter, and stretching pill dot indicators.",
    code: `ui.carousel("deck", &mut s.slide, &slides).height(182.0);`,
    options: [
      ["CarouselSlide::new(tag, title, body, icon, tone)", "Slide card with ambient OKLab corner glow, tag pill and body copy."],
      [".height(px)", "Viewport height in logical pixels."],
    ],
    keys: [["← / →", "Slide to previous or next card"]],
  },
  {
    slug: "callout", name: "Callout", group: "Feedback", demo: "callout",
    summary: "Documentation and inline note/tip/warning/danger callout box with a 3.5 px tone accent bar, icon, heading, and rich body slot.",
    code: `ui.callout(Tone::Info, "Zero idle frame wakeups").show(|ui| {
    ui.label("FrameOutput::repaint_after returns None once springs settle.").secondary();
});`,
    options: [
      ["ui.callout(Tone, title).icon(name).show(|ui| ..)", "Supports Neutral, Info/Accent, Success, Warning and Danger tones."],
    ],
    keys: [],
  },
  {
    slug: "undo-toast", name: "Undo snackbar", group: "Feedback", demo: "undo-toast",
    summary: "Elevated action snackbar for reversible destructive actions, pairing a remaining-window progress strip with an Undo button and dismiss ×.",
    code: `let out = ui.undo_bar("arch", "Archived runner “eu-central-04”", 0.7).show();
if out.undo_clicked { restore_runner(); }`,
    options: [
      [".action_label(text) .show() -> UndoBarOut", "Returns undo_clicked and dismissed flags."],
    ],
    keys: [["Tab + Enter", "Trigger Undo or dismiss"]],
  },
  {
    slug: "image-frame", name: "Image frame", group: "Layout and surfaces", demo: "image-frame",
    summary: "Aspect-framed media surface with procedural high-DPI artwork, loading shimmer state, graceful broken-image fallback, caption bar, and click-to-zoom.",
    code: `ui.image_frame("shot", "Ethereal Aurora Pass")
    .caption("Click frame to zoom · 2× DPI")
    .status(ImageStatus::Loaded)
    .height(190.0);`,
    options: [
      [".status(ImageStatus::Loaded | Loading | Fallback)", "Draws artwork, loading pulse or broken-image empty state."],
      [".caption(text) .height(px)", "Floating glass caption bar and click-to-zoom spring."],
    ],
    keys: [["Enter / Space", "Toggle zoom when loaded"]],
  },
  {
    slug: "sankey-chart", name: "Sankey chart", group: "Charts", demo: "sankey-chart",
    summary: "Multi-stage flow diagram rendered with smooth cubic-Bezier gradient ribbons, automatic column layout, and conversion hover tooltips.",
    code: `let nodes = ["Ingress", "Edge Cache", "Origin API", "2xx Cached", "2xx Dynamic"];
let links = [SankeyLink::new(0, 1, 84.0), SankeyLink::new(1, 3, 78.0)];
ui.sankey_chart("flow", &nodes, &links, 210.0);`,
    options: [
      ["SankeyLink::new(from, to, value)", "Directed flow volume between node indices; stage columns resolve automatically."],
    ],
    keys: [],
  },
  {
    slug: "contribution-graph", name: "Contribution graph", group: "Charts", demo: "contribution-graph",
    summary: "GitHub-style 7-weekday × N-week calendar heatmap with month headers, perceptual OKLab color ramp, legend, and per-day hover tooltips.",
    code: `ui.contribution_graph("commits", &weeks, &["Jun", "Jul", "Aug", "Sep", "Oct", "Nov"]);`,
    options: [
      ["weeks: &[[u8; 7]]", "Each column holds 7 daily counts (Sun..Sat) scaled across a 5-stop OKLab ramp."],
    ],
    keys: [],
  },
  {
    slug: "histogram", name: "Histogram", group: "Charts", demo: "histogram",
    summary: "Distribution bucket histogram with spring-growing bars, vertical dashed percentile markers (p50, p95, p99), and sample share tooltips.",
    code: `let markers = [
    HistogramMarker::new(0.32, "p50 · 0.17ms", Tone::Success),
    HistogramMarker::new(0.88, "p99 · 0.39ms", Tone::Danger),
];
ui.histogram("lat", &buckets, &counts, &markers, 196.0);`,
    options: [
      ["HistogramMarker::new(pos_0_to_1, label, Tone)", "Draws a dashed reference line and pill callout flag."],
    ],
    keys: [],
  },
  {
    slug: "kpi-strip", name: "KPI strip", group: "Charts", demo: "kpi-strip",
    summary: "Executive multi-metric comparison strip inside one divided glass surface, pairing tabular readouts with delta badges and synchronized sparklines.",
    code: `let kpis = [
    KpiMetric::new("Throughput", "142.8k/s", 14.2, vec![94.0, 112.0, 142.8]),
    KpiMetric::new("Median frame", "0.18 ms", 8.4, vec![0.24, 0.20, 0.18]),
];
ui.kpi_strip("kpis", &kpis);`,
    options: [
      ["KpiMetric::new(label, value, delta_pct, trend)", "Colors trend sparkline and delta badge automatically from delta sign."],
    ],
    keys: [],
  },
  {
    slug: "day-schedule", name: "Week schedule", group: "Planning", demo: "day-schedule",
    summary: "Multi-day hourly time-block schedule calendar with tinted event cards, room tags, and a live current-time red needle.",
    code: `let events = [
    ScheduleEvent::new(0, 9.5, 2.0, "WGSL shader audit", Tone::Accent).room("Lab A"),
];
ui.day_schedule("cal", &["Mon 16", "Tue 17", "Wed 18"], 9, 17, Some((2, 13.3)), &events, 250.0);`,
    options: [
      ["ScheduleEvent::new(day, start_hr, dur_hr, title, Tone).room(..)", "Fractional hours place blocks accurately against the hour grid."],
    ],
    keys: [],
  },
  {
    slug: "node-editor", name: "Node editor", group: "Planning", demo: "node-editor",
    summary: "Interactive shader and workflow node graph with dot-grid canvas, pointer-draggable node cards, typed sockets, and cubic-Bezier wires.",
    code: `let wires = [GraphWire::new(0, 0, 1, 0), GraphWire::new(1, 0, 2, 0)];
ui.node_editor("shader_graph", &mut s.nodes, &wires, 224.0);`,
    options: [
      ["GraphNode::new(title, pos, inputs, outputs, Tone)", "Drag any node card to reposition it live while wires stay attached."],
      ["GraphWire::new(from_node, from_port, to_node, to_port)", "Anti-aliased cubic-Bezier connection."],
    ],
    keys: [["Pointer drag", "Move node cards within the canvas"]],
  },
  {
    slug: "commit-graph", name: "Commit graph", group: "Developer tools", demo: "commit-graph",
    summary: "Git branch and merge rail log with multi-lane Bezier curves, monospace short SHAs, HEAD/tag pills, and keyboard row selection.",
    code: `let commits = [
    CommitRow::new("f3d0731", "widgets: add pro inputs", "AL", "4m ago", 0, vec![0, 1])
        .badge("HEAD → ethereal"),
];
ui.commit_graph("log", &commits, &mut s.selected);`,
    options: [
      ["CommitRow::new(sha, msg, author, time, lane, parents).badge(..)", "Connects each commit dot smoothly to its parent lane indices."],
    ],
    keys: [["↑ / ↓", "Move selected commit row"]],
  },
  {
    slug: "test-explorer", name: "Test explorer", group: "Developer tools", demo: "test-explorer",
    summary: "Interactive test runner card with pass/fail/skip badges, Failed-only filter chip, expandable assertion diff diagnostics, and Re-run trigger.",
    code: `if ui.test_explorer("suite", &s.tests) {
    rerun_suite(&mut s.tests);
}`,
    options: [
      ["TestCase::passed(..) / ::failed(.., msg) / ::skipped(..)", "Clicking a failed test toggles its inline assertion diagnostic box."],
    ],
    keys: [["Tab + Space", "Filter failed tests, expand diagnostics or click Re-run"]],
  },
  {
    slug: "data-grid", name: "Data grid", group: "Data", demo: "data-grid",
    summary: "Spreadsheet-style editable cell grid with row gutter, active cell coordinate bar (A1, B2), arrow-key cell navigation, and Enter inline editing.",
    code: `let cols = [GridColumn::new("Region"), GridColumn::new("RPS Cap").mono()];
ui.data_grid("capacity", &cols, &mut s.rows);`,
    options: [
      ["GridColumn::new(title).mono()", "Column header with optional tabular monospace formatting."],
    ],
    keys: [["↑ / ↓ / ← / →", "Move active cell selection"], ["Enter", "Edit active cell inline or commit"], ["Escape", "Cancel cell edit"]],
  },
  {
    slug: "filter-bar", name: "Filter bar", group: "Data", demo: "filter-bar",
    summary: "Structured facet filter strip (Field · operator · Value ×) with an anchored + Add filter preset menu and one-click Clear.",
    code: `ui.filter_bar("flt", &mut s.facets, &[("Status", "is", "Healthy")]);`,
    options: [
      ["FilterFacet::new(field, op, value)", "Removable structured filter token."],
    ],
    keys: [["Tab + Enter", "Open + Add filter menu or remove facets"]],
  },
  {
    slug: "ai-citations", name: "AI citations", group: "AI", demo: "ai-citations",
    summary: "Grounded AI synthesis card with interactive inline [1], [2] citation pills synchronized with source reference cards below.",
    code: `ui.cited_answer("ans", &segments, &sources);`,
    options: [
      ["CitationSource::new(index, title, domain, snippet)", "Hovering or clicking either the inline badge or source card highlights both."],
    ],
    keys: [["Tab + Space", "Pin or unpin a citation highlight"]],
  },
  {
    slug: "text-effects", name: "Text effects", group: "Motion", demo: "text-effects",
    summary: "Ethereal kinetic typography: per-word staggered spring-rise headlines blended across perceptual OKLab accent gradients, with Replay and reduced-motion support.",
    code: `ui.text_effects_card(
    "hero_fx",
    "Kinetic Typography",
    "Closed-form springs meet perceptual OKLab.",
    "Each word settles to exact zero idle frame wakeups.",
);`,
    options: [
      ["ui.text_effects_card(key, tag, headline, caption)", "Renders immediately without motion when reduced_motion is active."],
    ],
    keys: [],
  },
  {
    slug: "rotary-knob", name: "Rotary knob", group: "Selection", demo: "rotary-knob",
    summary: "Studio-grade parameter dial adjusted by vertical pointer drag or arrow keys, drawn with a 270° GPU arc band, radial ticks, press squash, and tabular readout.",
    code: `ui.rotary_knob("Cutoff", &mut s.cutoff, 80.0..=2400.0).unit(" Hz");`,
    options: [
      [".unit(text) .diameter(px)", "Tabular suffix and outer arc diameter."],
    ],
    keys: [["↑ / ↓ / ← / →", "Adjust value in 5% steps"], ["Home / End", "Jump to min or max"]],
  },
  {
    slug: "compare-slider", name: "Comparison slider", group: "Motion", demo: "compare-slider",
    summary: "Interactive before/after split comparison frame with a draggable vertical divider, circular handle pill, and keyboard scrubbing.",
    code: `ui.compare_split("cmp", &mut s.ratio, "Wireframe mesh", "SDF shaded").height(212.0);`,
    options: [
      ["ui.compare_split(key, &mut ratio, left_label, right_label)", "Clips left and right passes cleanly at physical pixel resolution."],
    ],
    keys: [["← / →", "Scrub split divider left or right"]],
  },
  {
    slug: "countdown", name: "Countdown timer", group: "Feedback", demo: "countdown",
    summary: "Circular arc-ring countdown and Pomodoro timer paired with segmented tabular MIN / SEC glass tiles, Start/Pause, and Reset.",
    code: `ui.countdown_card("bake", "Canary bake window", &mut s.secs, 900, &mut s.running);`,
    options: [
      ["ui.countdown_card(key, title, &mut secs, total_secs, &mut running)", "Requests zero repaints while paused."],
    ],
    keys: [["Tab + Space", "Start/Pause or Reset"]],
  },
  {
    slug: "checklist-card", name: "Checklist card", group: "Planning", demo: "checklist-card",
    summary: "Onboarding and release-readiness checklist card with an animated completion progress bar, live n / m badge, and spring-checked rows.",
    code: `ui.checklist_card("rc", "v0.5 Release readiness", &mut s.steps);`,
    options: [
      ["ChecklistStep::new(title, detail, tag, done)", "Clicking anywhere on a step row toggles completion and springs the progress bar."],
    ],
    keys: [["Tab + Space", "Toggle step checkbox"]],
  },
  {
    slug: "action-bar", name: "Floating action bar", group: "Actions", demo: "action-bar",
    summary: "Elevated multi-selection batch command bar pairing a selected count pill with quick actions, destructive action, and × clear.",
    code: `let out = ui.floating_action_bar(
    "batch",
    selected_count,
    &[("download", "Export", false), ("trash-2", "Terminate", true)],
);`,
    options: [
      ["actions: &[(&str, &str, bool)]", "Tuple of (icon, label, is_danger); returns ActionBarOut { clicked_action, cleared }."],
    ],
    keys: [["Tab + Enter", "Activate batch action or clear selection"]],
  },
  {
    slug: "speed-dial", name: "Speed dial", group: "Actions", demo: "speed-dial",
    summary: "Expandable floating action button whose + icon rotates 45° via spring into × while fanning out labeled quick-action pills.",
    code: `let items = [
    SpeedDialItem::new("rocket", "Promote canary").shortcut("⌘P"),
    SpeedDialItem::new("terminal", "Open shell").shortcut("⌘J"),
];
ui.speed_dial("fab", &mut s.open, &items);`,
    options: [
      ["SpeedDialItem::new(icon, label).shortcut(..)", "Returns Some(index) when an item is clicked and closes the dial."],
    ],
    keys: [["Escape", "Collapse open speed dial"]],
  },
  {
    slug: "numeric-pad", name: "Numeric keypad", group: "Inputs", demo: "numeric-pad",
    summary: "Tactile 3×4 PIN and numeric entry pad with press-squash keys, Clear, Backspace, and masked or plain tabular readout.",
    code: `if ui.numeric_pad("pin", &mut s.code, 6, true) {
    unlock_vault();
}`,
    options: [
      ["ui.numeric_pad(key, &mut value, max_len, masked) -> bool", "Returns true the moment max_len digits are entered."],
    ],
    keys: [["Tab + Space", "Press keypad digits, Clear or Backspace"]],
  },
  {
    slug: "dimension-input", name: "Dimension input", group: "Inputs", demo: "dimension-input",
    summary: "CSS and CAD dimension field combining a tabular numeric editor with an inline px | % | rem | auto unit switcher.",
    code: `ui.dimension_input("max_w", "Max container width", &mut s.width, &mut s.unit);`,
    options: [
      ["DimensionUnit::Px | Percent | Rem | Auto", "Selecting Auto switches the value slot cleanly to Automatic sizing."],
    ],
    keys: [["↑ / ↓", "Increment or decrement numeric dimension"]],
  },
  {
    slug: "stacked-meter", name: "Stacked meter", group: "Feedback", demo: "stacked-meter",
    summary: "Multi-category proportion bar (storage or language breakdown) with hairline gaps, rounded outer caps, and an itemized dot legend.",
    code: `let segs = [
    MeterSegment::new("Rust", 68.0, "68% · 965 KB"),
    MeterSegment::new("WGSL", 14.0, "14% · 198 KB"),
];
ui.stacked_meter("bundle", "Artifact composition", "1.42 MB total", &segs);`,
    options: [
      ["MeterSegment::new(label, value, unit_hint)", "Normalizes segment shares automatically and animates entrance via spring."],
    ],
    keys: [],
  },
  {
    slug: "announcement-bar", name: "Announcement bar", group: "Feedback", demo: "announcement-bar",
    summary: "Pill-framed release and status ribbon with leading accent tag, single-line message, trailing-arrow CTA button, and dismiss ×.",
    code: `let (cta, dismissed) = ui.announcement_bar(
    "v05",
    "v0.5",
    "150 GPU-rendered Rust components across four OS looks",
    "Explore",
);`,
    options: [
      ["ui.announcement_bar(key, tag, message, cta) -> (bool, bool)", "Returns (cta_clicked, dismissed)."],
    ],
    keys: [["Tab + Enter", "Trigger CTA or dismiss"]],
  },
  {
    slug: "quantity-picker", name: "Quantity picker", group: "Selection", demo: "quantity-picker",
    summary: "Replica and item quantity selector pairing − / + stepper buttons and a rolling odometer readout with quick-set multiplier chips.",
    code: `ui.quantity_picker("workers", "Dedicated GPU replicas", &mut s.replicas, 1, 32, &[1, 4, 8, 16]);`,
    options: [
      ["presets: &[i64]", "Renders quick-set chips below the rolling counter."],
    ],
    keys: [["Tab + Space", "Step down/up or pick a preset chip"]],
  },
  {
    slug: "swatch-picker", name: "Swatch picker", group: "Selection", demo: "swatch-picker",
    summary: "Palette swatch selector with a spring-popping selection ring, OKLab lightness-contrasted checkmark, and live hex readout.",
    code: `ui.swatch_picker("brand", "Brand accent token", &mut s.idx, &swatches);`,
    options: [
      ["swatches: &[(&str, Color)]", "Automatically picks black or white checkmarks from OKLab lightness L."],
    ],
    keys: [["← / →", "Cycle selected color swatch"]],
  },
  {
    slug: "date-strip", name: "Date strip", group: "Dates", demo: "date-strip",
    summary: "Horizontal 7-day calendar strip with per-day event count dots and a spring-gliding active day pill.",
    code: `ui.date_strip("week", &days, &mut s.selected_day);`,
    options: [
      ["DateStripDay::new(weekday, day_num, events)", "Draws up to three event indicator dots beneath each day number."],
    ],
    keys: [["← / →", "Move active day backward or forward"]],
  },
  {
    slug: "world-clocks", name: "World clocks", group: "Dates", demo: "world-clocks",
    summary: "Distributed team timezone strip displaying tabular local times, sun/moon day-night icons, city names, and UTC offset pills.",
    code: `let clocks = [
    WorldClock::new("San Francisco", "SFO", "08:30", "UTC-8", true),
    WorldClock::new("Tokyo", "HND", "01:30", "UTC+9", false),
];
ui.world_clocks("tz", &clocks);`,
    options: [
      ["WorldClock::new(city, code, time_str, offset, is_daytime)", "Divided horizontal glass strip."],
    ],
    keys: [],
  },
  {
    slug: "bullet-chart", name: "Bullet chart", group: "Charts", demo: "bullet-chart",
    summary: "Stephen Few actual-versus-target bullet graph with three qualitative background bands, spring measure bar, and perpendicular target marker.",
    code: `let metrics = [
    BulletMetric::new("Edge cache hit ratio", 94.0, 90.0, 100.0, "%"),
];
ui.bullet_chart("slo", &metrics);`,
    options: [
      ["BulletMetric::new(label, actual, target, max, unit)", "Colors the primary bar emerald automatically when actual meets or beats target."],
    ],
    keys: [],
  },
  {
    slug: "metric-table", name: "Metric watchlist", group: "Charts", demo: "metric-table",
    summary: "Compact service and market watchlist table combining labels, inline area sparklines, tabular readouts, and % delta badges.",
    code: `ui.metric_table("watch", &rows);`,
    options: [
      ["MetricRow::new(name, sub, spark, value, delta_pct)", "Synchronizes sparkline color and delta pill tone."],
    ],
    keys: [],
  },
  {
    slug: "ai-model-picker", name: "AI model picker", group: "AI", demo: "ai-model-picker",
    summary: "LLM selector cards displaying provider, context window, token pricing, capability badges, and 5-dot latency ratings.",
    code: `ui.ai_model_picker("models", &models, &mut s.selected);`,
    options: [
      ["AiModelSpec::new(name, provider, ctx, price, speed_dots, tags)", "Spring-animated radio dot and accent border."],
    ],
    keys: [["↑ / ↓", "Switch selected model"]],
  },
  {
    slug: "context-meter", name: "Context meter", group: "AI", demo: "context-meter",
    summary: "AI context-window token budget inspector pairing a multi-slice GPU arc ring with itemized prompt, AST, and history token counts.",
    code: `let slices = [
    ContextSlice::new("System instructions", 8.4),
    ContextSlice::new("Workspace AST", 54.2),
];
ui.context_meter("ctx", &slices, 200.0);`,
    options: [
      ["ContextSlice::new(label, tokens_k)", "Draws proportional ring arcs against limit_k."],
    ],
    keys: [],
  },
  {
    slug: "api-inspector", name: "API inspector", group: "Developer tools", demo: "api-inspector",
    summary: "REST and RPC endpoint tester with a cycleable HTTP verb badge (GET/POST/PUT/DELETE), URL bar, Send button, status/latency header, and syntax JSON body.",
    code: `if ui.api_inspector("rpc", &mut s.method, &mut s.url, "200 OK", 14.8, resp_json) {
    dispatch_request();
}`,
    options: [
      ["HttpMethod::Get | Post | Put | Delete", "Click the method badge to cycle HTTP verbs."],
    ],
    keys: [["Tab + Enter", "Cycle verb, edit URL or click Send"]],
  },
  {
    slug: "permission-matrix", name: "Permission matrix", group: "Data", demo: "permission-matrix",
    summary: "Interactive RBAC role × capability checkbox matrix for editing access policies across roles.",
    code: `ui.permission_matrix("rbac", &["Admin", "Maintainer", "Dev"], &["deploys:promote", "secrets:rotate"], &mut s.grid);`,
    options: [
      ["granted: &mut [Vec<bool>]", "Capability-major boolean matrix; returns true on any toggle."],
    ],
    keys: [["Tab + Space", "Toggle role × capability cell"]],
  },
  {
    slug: "description-list", name: "Description list", group: "Data", demo: "description-list",
    summary: "Structured two-column metadata and resource specification card with monospace formatting and one-click clipboard copy icons.",
    code: `let pairs = [
    DescPair::new("Cluster ID", "clu_01J9LUMEN8842").mono().copyable(),
    DescPair::new("Primary region", "eu-central-1"),
];
ui.description_list("meta", "Cluster specification", &pairs);`,
    options: [
      ["DescPair::new(term, detail).mono().copyable()", "Writes directly to the system/browser clipboard when Copy is clicked."],
    ],
    keys: [],
  },
  {
    slug: "masonry-grid", name: "Masonry grid", group: "Layout and surfaces", demo: "masonry-grid",
    summary: "Multi-column staggered masonry layout that distributes variable-height feature and metric cards into balanced columns.",
    code: `ui.masonry_grid("bento", 3, &cards);`,
    options: [
      ["MasonryCard::new(tag, title, body, Tone).metric(..)", "Balances card heights automatically across 2–4 columns."],
    ],
    keys: [],
  },
  {
    slug: "collapsible-card", name: "Collapsible card", group: "Layout and surfaces", demo: "collapsible-card",
    summary: "Titled surface card whose header toggles disclosure with a spring-rotating polyline chevron and summary badge.",
    code: `ui.collapsible_card("tls", "Zero-trust mTLS", Some("Active"), &mut s.open, |ui| {
    ui.switch("Enforce strict SNI", &mut s.strict_sni);
});`,
    options: [
      ["ui.collapsible_card(key, title, badge, &mut open, |ui| ..)", "Rounds all four header corners when closed and top corners when open."],
    ],
    keys: [["Tab + Space", "Expand or collapse card"]],
  },
  {
    slug: "marquee", name: "Marquee ticker", group: "Motion", demo: "marquee",
    summary: "Seamless infinite horizontal pill ticker with soft edge fade masks, automatic pause-on-hover, and zero wakeups when paused or under reduced motion.",
    code: `ui.marquee_strip("caps", &[("layers", "Single-pass WGSL"), ("sparkles", "Closed-form springs")], s.playing);`,
    options: [
      ["playing: bool", "Pauses automatically on pointer hover or when reduced_motion is active."],
    ],
    keys: [],
  },
  {
    slug: "presence-pile", name: "Presence pile", group: "Feedback", demo: "presence-pile",
    summary: "Overlapping multiplayer collaborator avatar stack with live status cutout rings, +N overflow counter, hover lift, and viewport follow toggle.",
    code: `let peers = [
    PresencePeer::new("LF", "Lena Fischer", "Editing paint.rs · line 233", 0).editing(true),
    PresencePeer::new("JB", "Jonas Berg", "Inspecting flame graph", 1).editing(true),
];
ui.presence_pile("collab", &peers, 4, &mut s.following);`,
    options: [
      ["PresencePeer::new(initials, name, status, tone).editing(bool)", "Draws a cutout avatar ring and live editing dot."],
      ["max_visible: usize", "Collapses additional peers into a +N pill."],
    ],
    keys: [["Tab + Space", "Jump to collaborator or toggle Follow"]],
  },
  {
    slug: "comment-thread", name: "Comment thread", group: "Feedback", demo: "comment-thread",
    summary: "Threaded code and design review comment card with reply connector rails, bouncy heart reaction counters, and thread resolution toggle.",
    code: `ui.comment_thread("pr_148", "PR #148 · paint.rs", &mut s.comments, &mut s.resolved);`,
    options: [
      ["CommentEntry::new(author, initials, time, body).reply().with_likes(n, liked)", "Renders top-level comments and indented replies."],
      ["resolved: &mut bool", "Toggles Open / Resolved status badge and header button."],
    ],
    keys: [["Tab + Space", "Like comment or resolve thread"]],
  },
  {
    slug: "reaction-bar", name: "Reaction bar", group: "Actions", demo: "reaction-bar",
    summary: "Interactive strip of Lucide reaction pills with bouncy 0.95 press squash, accent active tint, live tabular counts, and a quick-add button.",
    code: `ui.reaction_bar("release_rx", &mut s.reactions);`,
    options: [
      ["ReactionChip::new(icon, label, count, active)", "Toggles active state and increments/decrements count on click."],
    ],
    keys: [["Tab + Space / Enter", "Toggle reaction pill"]],
  },
  {
    slug: "attachment-list", name: "Attachment list", group: "Inputs", demo: "attachment-list",
    summary: "File attachment cards with semantic extension badges, smooth spring upload progress bars or completion state, sizes, and remove buttons.",
    code: `ui.attachment_list("artifacts", &mut s.attachments);`,
    options: [
      ["AttachmentItem::uploaded(name, ext, size)", "Completed attachment row."],
      ["AttachmentItem::uploading(name, ext, size, progress)", "Displays an animated progress bar until complete."],
    ],
    keys: [["Tab + Enter", "Remove attachment"]],
  },
  {
    slug: "progress-steps", name: "Progress steps", group: "Navigation", demo: "progress-steps",
    summary: "Segmented linear onboarding and story walkthrough bars with clickable progress segments, step copy, and Previous / Continue navigation.",
    code: `ui.story_bars("onboard", &[("Connect repo", "..."), ("Deploy", "...")], &mut s.step);`,
    options: [
      ["steps: &[(&str, &str)]", "Step title and body description pairs."],
      ["current: &mut usize", "Active step index; segments animate with snappy springs."],
    ],
    keys: [["Tab + Enter", "Select segment or advance step"]],
  },
  {
    slug: "shortcut-bar", name: "Shortcut bar", group: "Navigation", demo: "shortcut-bar",
    summary: "Elevated command-hint footer bar pairing multi-key chords with clickable contextual actions.",
    code: `let hints = [ShortcutHint::new(&["⌘", "Enter"], "Deploy"), ShortcutHint::new(&["Esc"], "Clear")];
ui.shortcut_bar("footer", "3 runners selected", &hints);`,
    options: [
      ["ShortcutHint::new(keys, action)", "Renders keycaps beside the action label and returns Some(index) on click."],
    ],
    keys: [["Tab + Enter", "Trigger hinted command"]],
  },
  {
    slug: "view-switcher", name: "View switcher", group: "Selection", demo: "view-switcher",
    summary: "Workspace layout mode switcher (List, Board, Calendar, Graph) with Lucide icons and a single spring-gliding elevated indicator pill.",
    code: `let modes = [ViewMode::new("list", "List"), ViewMode::new("layout-grid", "Board")];
ui.view_switcher("views", &modes, &mut s.mode);`,
    options: [
      ["ViewMode::new(icon, label)", "Segment definition with icon and text."],
    ],
    keys: [["← / →", "Cycle active view mode"]],
  },
  {
    slug: "gradient-slider", name: "Gradient slider", group: "Selection", demo: "gradient-slider",
    summary: "Continuous multi-stop perceptual OKLab color and temperature track slider with an elevated spring thumb tinted to the sampled stop.",
    code: `ui.gradient_slider("temp", "Sampling temperature", &mut s.temp, 0.0..=1.5, &stops, "");`,
    options: [
      ["stops: &[Color]", "Interpolated across horizontal GPU segments in perceptual OKLab space."],
      ["unit: &str", "Appended to the tabular readout (e.g. \" K\")."],
    ],
    keys: [["← / →", "Step 5%"], ["Home / End", "Jump to min / max"]],
  },
  {
    slug: "schedule-picker", name: "Schedule picker", group: "Dates", demo: "schedule-picker",
    summary: "Recurring cron schedule builder with Hourly / Daily / Weekly cadence, M–S weekday pills, UTC window stepper, and live 5-field cron badge.",
    code: `ui.schedule_picker("backup_cron", &mut s.rule);`,
    options: [
      ["ScheduleRule { freq, weekdays, hour, minute }", "Exposes rule.cron_expr() for standard 5-field cron output."],
    ],
    keys: [["Tab + Space", "Toggle weekday pills or step run time"]],
  },
  {
    slug: "quarter-picker", name: "Quarter picker", group: "Dates", demo: "quarter-picker",
    summary: "Fiscal year stepper and Q1–Q4 quarter cards with month-range subtitles, spring selection borders, and arrow-key navigation.",
    code: `ui.quarter_picker("fy_picker", &mut s.year, &mut s.quarter);`,
    options: [
      ["year: &mut i32, quarter: &mut u8", "Clamps quarter to 1..=4 and returns true on change."],
    ],
    keys: [["← / →", "Previous / next quarter"]],
  },
  {
    slug: "win-loss-chart", name: "Win-loss chart", group: "Charts", demo: "win-loss-chart",
    summary: "Zero-baseline diverging horizontal bar chart plotting positive gains in green to the right and regressions in red to the left.",
    code: `let deltas = [DivergingBar::new("sdf_batch", 28.4, "%"), DivergingBar::new("cold_scan", -6.8, "%")];
ui.diverging_bars("bench", "Benchmark vs Baseline", &deltas, 32.0);`,
    options: [
      ["DivergingBar::new(label, delta, unit)", "Signed delta value relative to the center zero axis."],
    ],
    keys: [],
  },
  {
    slug: "waffle-chart", name: "Waffle chart", group: "Charts", demo: "waffle-chart",
    summary: "10×10 100-cell rounded-square proportion grid with spring entrance and right-hand percentage breakdown legend.",
    code: `let slices = [WaffleSlice::new("WebGPU", 58, Tone::Accent), WaffleSlice::new("Vulkan", 24, Tone::Success)];
ui.waffle_chart("fleet", "Active Backends", &slices);`,
    options: [
      ["WaffleSlice::new(label, cells, Tone)", "Each cell represents 1% of the 10×10 matrix."],
    ],
    keys: [],
  },
  {
    slug: "schema-viewer", name: "Schema viewer", group: "Developer tools", demo: "schema-viewer",
    summary: "Database table and struct schema inspector card with PK / FK / IDX key badges, monospace column names, nullability, and data types.",
    code: `let cols = [SchemaField::new("id", "uuid").pk(), SchemaField::new("org_id", "uuid").fk()];
ui.schema_card("deploys", "public.deployments", &cols);`,
    options: [
      ["SchemaField::new(name, ty).pk().fk().indexed().nullable()", "Builder methods for relational constraints."],
    ],
    keys: [],
  },
  {
    slug: "env-switcher", name: "Environment switcher", group: "Developer tools", demo: "env-switcher",
    summary: "Development / Staging / Production environment selector whose surface morphs into a guarded warning/danger banner with write-lock protection.",
    code: `ui.env_banner("env", &mut s.env_idx, "eu-central-1", &mut s.write_locked);`,
    options: [
      ["write_locked: &mut bool", "Automatically engages write lock when switching into Production."],
    ],
    keys: [["← / →", "Switch target environment"]],
  },
  {
    slug: "memory-profiler", name: "Memory profiler", group: "Developer tools", demo: "memory-profiler",
    summary: "GPU and allocator arena map showing unified slab blocks, high-water peak marker, and per-slab MiB utilization bars.",
    code: `let slabs = [ArenaSlab::new("Vertex VBO", 14.2, 32.0), ArenaSlab::new("Glyph Atlas", 8.6, 16.0)];
ui.memory_arena("arena", &slabs, 48.5);`,
    options: [
      ["ArenaSlab::new(name, used_mb, cap_mb)", "Individual slab allocation and capacity."],
      ["peak_mb: f32", "Draws a perpendicular high-water mark across the unified arena bar."],
    ],
    keys: [],
  },
  {
    slug: "ai-suggestion", name: "AI code suggestion", group: "AI", demo: "ai-suggestion",
    summary: "Inline AI patch review card with target file header, rationale, red/green monospace diff lines, and Accept / Dismiss workflow.",
    code: `ui.code_suggestion("patch", "src/shader.wgsl", "Use Gaussian erfc.", &removed, &added, &mut s.decision);`,
    options: [
      ["SuggestionDecision::{Pending, Accepted, Dismissed}", "Tracks review outcome with one-click reset."],
    ],
    keys: [["Tab + Enter", "Accept or dismiss proposed patch"]],
  },
  {
    slug: "prompt-templates", name: "Prompt templates", group: "AI", demo: "prompt-templates",
    summary: "Two-column starter prompt gallery with Lucide category icons, domain badges, {{variable}} placeholders, and tactile press feedback.",
    code: `if let Some(i) = ui.prompt_templates("starters", &templates) { /* insert templates[i] */ }`,
    options: [
      ["PromptTemplate::new(icon, title, category, prompt)", "Returns Some(index) when a template card is clicked."],
    ],
    keys: [["Tab + Enter", "Choose starter prompt"]],
  },
  {
    slug: "pricing-cards", name: "Pricing cards", group: "Layout and surfaces", demo: "pricing-cards",
    summary: "Multi-tier plan comparison cards with an Annual (-20%) billing switch, rolling odometer prices, Popular highlight, and feature checkmarks.",
    code: `ui.pricing_cards("plans", &plans, &mut s.annual, &mut s.selected_plan);`,
    options: [
      ["PlanSpec::new(name, monthly, annual, blurb, features).popular()", "Animates price digits via rolling_number when billing frequency toggles."],
    ],
    keys: [["Tab + Space", "Toggle annual billing or select plan"]],
  },
  {
    slug: "ab-comparison", name: "A/B comparison", group: "Data", demo: "ab-comparison",
    summary: "Statistical experiment card comparing Control vs Variant B conversion rates with relative lift badge, leader pill, and Bayesian confidence.",
    code: `ui.ab_comparison("exp", "WASM hydration", "Control", 61.4, "Variant B", 74.8, 99.2);`,
    options: [
      ["confidence_pct: f32", "Marks ≥95% confidence as statistically significant."],
    ],
    keys: [],
  },
  {
    slug: "equalizer", name: "Graphic equalizer", group: "Motion", demo: "equalizer",
    summary: "Interactive 8-band studio dB equalizer (-12..+12 dB) with Flat / Vocal / Warm / Air presets, smooth response curve, and vertical faders.",
    code: `ui.graphic_eq("master_eq", &mut s.bands_db, &mut s.preset);`,
    options: [
      ["bands_db: &mut [f32; 8]", "60 Hz to 16 kHz dB gains; springs smoothly between presets and settles to zero wakeups."],
    ],
    keys: [["↑ / ↓", "Adjust focused EQ band by 1.5 dB"]],
  },
];

const code = (s) => `<pre><code class="lang-rust">${esc(s)}</code></pre>`;
export function esc(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export const GUIDES = [
  {
    slug: "getting-started", title: "Getting started", lead: "Add lumen to a project and open a window in a few lines.",
    body: `
<h2>Install</h2>
<p>lumen is not on crates.io yet. Depend on the repository:</p>
${code(`[dependencies]
lumen = { git = "https://github.com/amiralibg/lumen" }`)}
<h2>Hello, lumen</h2>
<p>Your state is a plain struct. lumen calls your view every frame with <code>&amp;mut</code> access to it.</p>
${code(`#[derive(Default)]
struct State { notify: bool, volume: f32 }

fn main() {
    lumen::App::new("Settings").size(900, 600).run(State::default(), |ui, s| {
        ui.column().pad(24).gap(14).show(|ui| {
            ui.heading("General");
            ui.switch("Notifications", &mut s.notify);
            ui.slider("Volume", &mut s.volume, 0.0..=1.0);
            if ui.button("Save").primary().clicked() {
                println!("saved");
            }
        });
    });
}`)}
<p>Widgets are builders. Every option has a good default, and chained methods change it. The widget is added when the builder is dropped, or when you call <code>.clicked()</code>, <code>.changed()</code> or <code>.show()</code>.</p>
<h2>Look like the OS</h2>
${code(`lumen::App::new("Settings").native().run(state, view); // macOS, Windows or GNOME look + system accent`)}
<h2>Run the examples</h2>
${code(`cargo run -p lumen-gallery            # every widget, settings-app style
cargo run -p lumen-demos -- --list    # one demo per component
cargo run -p lumen-demos -- chart-line --dark --look mac`)}
<h2>Build for the web</h2>
${code(`rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130
./scripts/build-web.sh
python3 -m http.server -d web/dist 8080`)}
<p>Every demo on this site is that same Rust code compiled to WebAssembly and drawn with WebGPU or WebGL2.</p>`,
  },
  {
    slug: "tutorial", title: "Tutorial: a small app", lead: "Build a tiny expense tracker with a form, a chart and a toast.",
    body: `
<p>We'll build one screen: a form to add an expense, a bar chart by category, and a toast when something is added.</p>
<h2>1. State</h2>
${code(`#[derive(Clone, Copy, PartialEq)]
enum Category { Food, Travel, Tools }

struct Expense { what: String, amount: f32, cat: Category, date: lumen::Date }

struct App { what: String, amount: f32, cat: Category, date: lumen::Date, items: Vec<Expense> }`)}
<h2>2. The form</h2>
${code(`ui.card().title("Add expense").show(|ui| {
    ui.row().gap(12).fill_width().show(|ui| {
        ui.text_input(&mut s.what).label("What").fill_width();
        ui.select(&mut s.cat, &[(Category::Food, "Food"), (Category::Travel, "Travel"), (Category::Tools, "Tools")]).label("Category");
        ui.date_picker("Date", &mut s.date);
    });
    let amount = format!("\${:.0}", s.amount);
    ui.slider("Amount", &mut s.amount, 0.0..=500.0).step(5.0).readout(amount);
    if ui.button("Add").primary().icon("plus").clicked() && !s.what.is_empty() {
        s.items.push(Expense { what: std::mem::take(&mut s.what), amount: s.amount, cat: s.cat, date: s.date });
        ui.toast(lumen::Toast::new("Expense added").success());
    }
});`)}
<h2>3. The chart</h2>
${code(`let total = |c| s.items.iter().filter(|e| e.cat == c).map(|e| e.amount).sum::<f32>();
let data = vec![total(Category::Food), total(Category::Travel), total(Category::Tools)];
ui.card().title("By category").show(|ui| {
    ui.bar_chart("by-cat", &["Food", "Travel", "Tools"], vec![lumen::Series::new("Spent", data)]);
});`)}
<p>The bars spring to their new heights every time you add an expense. You never write animation code: widgets animate their own state.</p>`,
  },
  {
    slug: "layout", title: "Layout", lead: "Rows, columns and stacks with flex-style sizing.",
    body: `
<p>Containers hug their content by default. <code>.fill()</code> takes the remaining space; fixed sizes, min and max constrain it.</p>
${code(`ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
    ui.heading("Overview");
    ui.spacer();                       // pushes the rest to the right
    ui.button("Export").icon("download");
});

ui.column().width(320.0).pad(16).gap(8).bg(Token::Surface).radius(12.0).show(|ui| { /* … */ });

ui.scroll("list").show(|ui| { /* long content */ });`)}
<table><tr><th>Method</th><th>Effect</th></tr>
<tr><td><code>.pad(p) .gap(g)</code></td><td>Inner padding, space between children</td></tr>
<tr><td><code>.width(w) .height(h) .fill() .fill_width()</code></td><td>Sizing</td></tr>
<tr><td><code>.min_width .max_width .min_height</code></td><td>Constraints</td></tr>
<tr><td><code>.align(Align) .justify(Justify) .center()</code></td><td>Cross and main axis</td></tr>
<tr><td><code>.bg(Token) .border(Token) .radius(r) .clip()</code></td><td>Decoration</td></tr>
<tr><td><code>.id(key)</code></td><td>Query the rect next frame with <code>ui.rect_of(id)</code></td></tr></table>`,
  },
  {
    slug: "looks", title: "Looks and theming", lead: "Customize everything, animate every change, or match the OS.",
    body: `
<p>A <code>Look</code> holds the neutrals for both modes, control heights and radii, per-widget shapes, the type scale, preferred fonts and motion. Four ship with lumen:</p>
<table><tr><th></th><th>Lumen</th><th>macOS</th><th>Windows</th><th>GNOME</th></tr>
<tr><td>Control height</td><td>32</td><td>24</td><td>32</td><td>34</td></tr>
<tr><td>Radius</td><td>8</td><td>6</td><td>4</td><td>6</td></tr>
<tr><td>Buttons</td><td>raised + border</td><td>bevel + hairline shadow</td><td>bottom edge</td><td>flat filled</td></tr>
<tr><td>Switch</td><td>36×20</td><td>38×22</td><td>outlined off</td><td>46×26</td></tr>
<tr><td>Inputs</td><td>outline + halo</td><td>outline + halo</td><td>accent underline</td><td>filled</td></tr>
<tr><td>Font</td><td>Geist</td><td>SF Pro</td><td>Segoe UI Variable</td><td>Adwaita Sans</td></tr></table>
${code(`let look = Look::mac()
    .rounded(1.5)                     // 150% radii
    .density(Density::Compact)
    .motion(Motion::playful())        // or calm(), or .slower(2.0)
    .text_scale(1.1)
    .with_neutrals(Mode::Dark, |n| n.background = Color::hex(0x101418));
ui.set_look(look);                    // blends over ~0.45 s

ui.set_mode(Mode::Dark);
ui.set_accent(Accent::from_hue(160.0));   // any hue; shades derived for contrast
ui.set_accent(Accent::Custom(0xE5484D));
ui.set_accent(Accent::Native);            // follow the OS accent
ui.override_color(Mode::Light, Token::Background, Some(Color::hex(0xFFFDF8)));`)}
<h2>Following the OS</h2>
<p><code>App::native()</code> picks the look for the platform and uses the system accent. The app follows dark mode live (the change animates), re-reads the accent color and the reduce-motion setting when the window regains focus, and on the web reads <code>prefers-reduced-motion</code>.</p>
<h2>Tokens</h2>
<p>Widgets read colors through named tokens (<code>Background</code>, <code>Surface</code>, <code>Foreground</code>, <code>Accent</code>, <code>AccentStrong</code>, <code>Border</code>, <code>FocusRing</code>, <code>Control</code>, <code>Series0..5</code>, …). Every token blends in OKLab when the mode, accent or look changes.</p>`,
  },
  {
    slug: "motion", title: "Motion", lead: "Springs solved in closed form, so they're exact at any frame rate.",
    body: `
<p>Every moving part in lumen is a spring or a short fade. Springs keep their velocity when retargeted, so interrupting an animation never jerks. An idle UI requests no frames.</p>
${code(`// Inside your own widget:
let m = ui.motion();
let x = ui.spring(id, 0, if open { 1.0 } else { 0.0 }, m.smooth);
let a = ui.fade(id, 1, hovered, m.hover_in, m.hover_out, Ease::Linear);`)}
<table><tr><th>Spring</th><th>Use</th></tr>
<tr><td><code>snappy</code></td><td>Toggles, presses, focus rings</td></tr>
<tr><td><code>smooth</code></td><td>Layout, sliding indicators, chart values</td></tr>
<tr><td><code>bouncy</code></td><td>Release, hover grow, toasts</td></tr>
<tr><td><code>gentle</code></td><td>Programmatic scrolling</td></tr></table>
<p>Motion presets: <code>Motion::lumen()</code>, <code>mac()</code>, <code>windows()</code>, <code>gnome()</code>, <code>playful()</code>, <code>calm()</code>. <code>.slower(2.0)</code> scales everything and keeps each spring's damping ratio. <code>ui.set_reduced_motion(true)</code> replaces movement with short fades.</p>`,
  },
  {
    slug: "custom-widgets", title: "Custom widgets", lead: "Interaction, state, animation and painting for your own components.",
    body: `
${code(`fn rating(ui: &mut Ui, value: &mut u8) {
    let id = ui.make_id("rating");
    let ix = ui.interact(id, Sense::CLICK, false);
    let accent = ui.color(Token::Accent);
    let muted = ui.color(Token::Track);
    let shown = ui.spring(id, 0, *value as f32, ui.motion().bouncy);
    if ix.clicked {
        if let Some(p) = ix.pointer {
            *value = (((p.x - ix.rect.min.x) / 24.0).floor() as u8 + 1).min(5);
        }
    }
    ui.leaf(Style::default(), vec2(120.0, 24.0), Some(id), Sense::CLICK, CursorIcon::Pointer,
        Some(Box::new(move |cx, r| {
            for i in 0..5 {
                let c = vec2(r.min.x + 12.0 + i as f32 * 24.0, r.center().y);
                let on = (shown - i as f32).clamp(0.0, 1.0);
                cx.painter.circle(c, 8.0, muted.mix(accent, on), 0.0, Color::TRANSPARENT);
            }
        })));
}`)}
<ul>
<li><b>Interaction</b> comes from last frame's layout: <code>ui.interact(id, sense, disabled)</code> gives hover, press, click, focus and pointer.</li>
<li><b>State</b>: <code>ui.mem::&lt;T&gt;(id)</code> stores anything per widget, dropped when the widget disappears.</li>
<li><b>Painting</b>: rounded rects with borders, shadows, text, icons, anti-aliased polylines, fills and arcs.</li>
<li><b>Layers</b>: <code>ui.layer(..)</code> draws above everything and is hit-tested first; popovers and dialogs are built on it.</li>
</ul>`,
  },
  {
    slug: "gpui", title: "Using lumen in GPUI", lead: "Put lumen panels inside a GPUI (Zed) app, next to your own GPUI elements.",
    body: `
<p><code>lumen-gpui</code> wraps a lumen UI in an ordinary GPUI view. Drop it anywhere in your element tree, give it a size like any other child, and it draws lumen's widgets, looks and motion there. Mouse, scroll, keyboard, clipboard, cursors and the window's light or dark appearance are wired up for you.</p>
<figure style="margin:24px auto;max-width:460px"><img src="img/gpui-dashboard.png" alt="A GPUI window: a GPUI header and region sidebar on the left, and a lumen dashboard with stat cards, a traffic chart, a text input, a switch, a slider and a deploy button filling the rest." style="width:100%;height:auto;display:block" loading="lazy"><figcaption style="text-align:center;font-size:13px;opacity:.7;margin-top:8px">The header and sidebar are GPUI elements. Everything right of the sidebar is one <code>LumenView</code>.</figcaption></figure>
<p>It targets <code>gpui 0.2.2</code> from crates.io. GPUI builds on macOS (Xcode's Metal toolchain required), Linux and Windows.</p>
<h2>Add it</h2>
${code(`[dependencies]
gpui = "0.2.2"
lumen-gpui = { git = "https://github.com/amiralibg/lumen" }`, "toml")}
<h2>A lumen panel in a GPUI window</h2>
${code(`use gpui::{prelude::*, div, px, size, App, Application, Bounds, Context, Entity, Window, WindowBounds, WindowOptions};
use lumen_gpui::LumenView;

struct Settings { notify: bool, volume: f32 }

struct Shell { panel: Entity<LumenView<Settings>> }

impl Render for Shell {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col()
            .child(div().p_2().child("A GPUI header"))
            .child(div().flex_1().child(self.panel.clone())) // lumen fills this
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(720.), px(480.)), cx);
        let opts = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() };
        cx.open_window(opts, |window, cx| {
            let panel = LumenView::build(Settings { notify: true, volume: 0.6 }, |ui, s| {
                ui.column().pad(24).gap(14).show(|ui| {
                    ui.heading("General");
                    ui.switch("Notifications", &mut s.notify);
                    ui.slider("Volume", &mut s.volume, 0.0..=1.0);
                });
            }, window, cx);
            cx.new(|_| Shell { panel })
        }).unwrap();
    });
}`)}
<h2>Sharing state with GPUI</h2>
<p>The view owns your state struct, the same way <code>lumen::App</code> does. GPUI code reads it with <code>state()</code> and changes it with <code>update()</code>, which also schedules a redraw:</p>
${code(`// GPUI → lumen: a GPUI click changes what lumen shows.
panel.update(cx, |p, cx| p.update(cx, |s| s.region = 2));

// lumen → GPUI: re-render a GPUI view whenever lumen handles input.
let _watch = cx.observe(&panel, |_, _, cx| cx.notify());
let deploys = panel.read(cx).state().deploys;`)}
<h2>Looks and options</h2>
${code(`cx.new(|cx| LumenView::new(state, view, window, cx)
    .look(Look::mac())        // any look, or change it later with ui.set_look
    .accent(Accent::Violet)
    .mode(Mode::Dark)         // default: follow the GPUI window's appearance
    .transparent())           // let GPUI content show between widgets`)}
<h2>How it works, and what it costs</h2>
<p>GPUI has no hook for another renderer to draw into its frame, so lumen renders offscreen with wgpu (one device shared by every view), reads the pixels back in GPUI's BGRA layout and paints them as a GPUI image. A view at rest renders nothing. It only produces frames after input, a state change, a resize, a theme change, or while something animates.</p>
<p>The readback is pipelined: GPUI shows the newest finished frame while the GPU renders the next one, so the main thread never waits for the GPU. On a 1326×1352-pixel panel animating on Apple silicon, that's 1.7 ms of main-thread time per frame (2.5 ms at p95), against 3.1 to 3.6 ms when each frame waits for its readback, at the cost of one frame of latency. For a whole window of lumen, use <code>lumen::App</code>, which presents directly. Set <code>LUMEN_GPUI_TRACE=1</code> to print each frame's times, and <code>LUMEN_GPUI_SYNC=1</code> to compare with blocking readback.</p>
<h2>Menus and popovers</h2>
<p>Menus, select lists, popovers, context menus and tooltips may extend past the view's bounds, over neighbouring GPUI elements, like a native popup. While one sticks out, the frame is painted in a GPUI <code>deferred</code> layer on top of the window, unclipped by scrolling parents, and only the menu itself takes the mouse. A click anywhere else closes it and still reaches the GPUI element under the pointer. Dialogs, sheets and toasts stay inside the view. Call <code>.contain_overlays()</code> to keep everything inside instead.</p>
<h2>Text input</h2>
<p>A focused lumen text field registers a GPUI input handler, so input methods work: CJK composition shows underlined in the field and commits on confirm, macOS press-and-hold accents and dead keys work, and the candidate window sits under the caret. Scripts the bundled Geist lacks load the system fonts as fallbacks the first time they appear.</p>
<h2>gpui-ce</h2>
<p>The community fork <code>gpui-ce</code> 0.3 is supported with <code>default-features = false, features = ["ce"]</code>. Start the app with <code>lumen_gpui::application()</code>, which works with either GPUI. The <code>ce</code> build compiles and passes its tests, but isn't verified in a window yet: on macOS 27, gpui-ce 0.3.0 deadlocks in its own <code>open_window</code>.</p>
<h2>Limits today</h2>
<ul>
<li>Accessibility trees aren't bridged to GPUI.</li>
</ul>
<p>Run the example: <code>cd crates/lumen-gpui && cargo run --example dashboard</code>, or <code>cargo run --example selftest</code> for a scripted check in a real window. It shows a GPUI sidebar driving a lumen dashboard, and a lumen Deploy button updating the GPUI header.</p>`,
  },
  {
    slug: "architecture", title: "Architecture", lead: "Five small crates. The core never touches a window or a GPU.",
    body: `
<pre><code>lumen-text   cosmic-text shaping, bundled fonts, glyph atlas
   ↓
lumen-core   ids, input, layout, looks, springs, widgets → DrawData
   ↓
lumen-wgpu   one pipeline, one WGSL shader (SDF rects, shadows, glyphs, meshes)
   ↓
lumen-app    winit event loop (native + wasm), clipboard, OS preferences, screenshots
   ↓
lumen        facade

lumen-gpui   optional: hosts lumen views inside a GPUI app (separate workspace)</code></pre>
<p>Each frame your closure records a layout tree. Widgets resolve interaction against the previous frame's hit rects. After the closure returns, the tree is laid out (hug, fixed or fill sizing, padding, gap, align, justify), then painted into one vertex buffer. Each 64-byte vertex carries its own shape data, so a backend is one shader.</p>`,
  },
  {
    slug: "faq", title: "FAQ", lead: "Native UI, GPUI, platforms and what's next.",
    body: `
<h2>Is lumen "native UI components for Rust"?</h2>
<p>It's native code, not a web view: your app compiles to a single binary that draws with the GPU through wgpu (Metal, Vulkan, DirectX 12, OpenGL) and to WebAssembly for the browser. lumen <em>draws its own components</em>, the way egui, GPUI, iced, Slint and Flutter do. It doesn't wrap AppKit, Win32 or GTK widgets. The platform looks reproduce each OS's control conventions (sizes, shapes, focus rings, fonts, accent color, motion) closely, but they're lumen widgets. If you need the OS's actual widgets, use bindings to those toolkits.</p>
<h2>Does lumen work with GPUI?</h2>
<p>Yes, through the <code>lumen-gpui</code> crate. A <code>LumenView</code> is a regular GPUI view, so you can build your app in GPUI and use lumen for panels, charts, forms and dialogs inside it. See <a href="#/guide/gpui">Using lumen in GPUI</a> for setup, state sharing and the cost of the offscreen bridge. For a window that's entirely lumen, <code>lumen::App</code> is still the faster path, and it also runs in the browser.</p>
<h2>Platforms?</h2>
<p>macOS, Windows, Linux (X11 and Wayland through winit), and browsers with WebGPU or WebGL2.</p>
<h2>Accessibility?</h2>
<p>Everything is keyboard reachable with visible focus. Screen reader support through AccessKit is on the roadmap.</p>
<h2>What's next?</h2>
<p>Combobox, command palette, number and tag inputs, a data table and virtualized grid, tree, code editor, and docking panels. And a password manager built with lumen.</p>`,
  },
];

export const CHANGELOG = [
  { v: "Unreleased", date: "2026-11-24", items: [
    "Every app fits a phone: under 1000 px the sidebar collapses to icons, and under 640 px each app shows one pane at a time with a back button (a tap opens a note, message or issue; music gets a mini player).",
    "Horizontal scrolling: Ui::scroll_row keeps its children at natural width and scrolls sideways with a trackpad or Shift + wheel, while a plain wheel still scrolls the page. Editor tabs, pricing cards and the phone board use it.",
    "Narrow cards: one-line labels keep their width instead of drawing over each other, icons and keycaps never shrink, chart labels thin out by measured width, and world clocks, quarter picker, countdown, shortcut bar, settings rows and model picker reflow. --check also reports text spilling out of squeezed widgets.",
    "Site: showcase scenes fill the column, the docs sidebar marks the current page with a sliding glass pill, pages keep their margins and tables stack on phones, and a demo that loses its GPU context falls back to its poster and reloads.",
    "Six new components: code editor, file browser with path bar, emoji picker, wheel and time picker, calendar heatmap and guided tour (177 in all), plus Preferences, a settings app.",
    "Color emoji: the glyph atlas is RGBA, so emoji keep their colors; scripts the bundled fonts lack load system fonts on first use.",
    "Input methods: CJK composition, dead keys and press-and-hold accents show underlined in text fields, natively and inside GPUI.",
    "GPUI: menus and popovers can overflow a lumen view, readback is pipelined (about half the main-thread time), and gpui-ce is supported behind a feature.",
    "Layout: narrow rows no longer squeeze children below their descendants' fixed widths; a --check mode reports clipped and overlapping text in any demo.",
    "GPUI support: the new lumen-gpui crate hosts lumen views inside a GPUI (Zed) app, with mouse, keyboard, clipboard, cursors and dark mode forwarded. See the GPUI guide.",
    "Seven complete apps, live on the Apps page: Postbox (mail), Cadence (music), Relay (issue tracker), Margin (notes), Lumi (AI assistant), Tally (finance) and Today (to-do).",
    "New list view: a selectable master list with a gliding highlight, staggered row entrances, unread dots that pop, springy stars and arrow-key navigation.",
    "Painter layers: push_layer(opacity, offset) fades and moves a whole group at once, so entrances move rows and everything in them together.",
    "Fixes: waveform playback now runs on real elapsed time instead of assuming 60 fps, and its bars fit narrow cards; gantt and trace labels end in an ellipsis instead of cutting glyphs; the equalizer header stacks in narrow cards.",
    "Eight showcase scenes: AI coding workspace, observability console, shader studio, release control plane, analytics terminal, RFC studio, cloud governance and an on-call launch room.",
  ] },
  { v: "Earlier", date: "2026-11-18", items: [
    "170 components across 14 groups: full CUI-parity suite adding presence pile, threaded comments, reaction bar, attachment list, story progress steps, shortcut bar, view switcher, OKLab gradient slider, cron schedule & fiscal quarter pickers, win-loss & 10×10 waffle charts, schema viewer, guarded environment switcher, GPU memory arena, AI patch suggestion, prompt templates, pricing cards, A/B comparison & 8-band graphic equalizer.",
    "Enterprise & tactile controls: floating action bar, speed-dial FAB, numeric keypad, CSS/CAD dimension input, stacked meter, announcement bar, quantity & swatch pickers, week date strip, world clocks, bullet & watchlist charts, AI model picker, context-window meter, REST API inspector, RBAC permission matrix, description list, masonry grid, collapsible card & marquee ticker.",
    "Visualizations, graphs & tactile controls: Sankey flow ribbons, GitHub contribution calendar, percentile histogram, KPI strip, week schedule, draggable node editor, Git commit graph, test explorer, editable data grid, facet filter bar, AI citations, kinetic text effects, rotary knob, before/after split slider, countdown timer & checklist card.",
    "AI, data & workspace: AI reasoning, tool calls, prompt composer & chat; data table, tree, accordion, timeline, markdown, syntax code block, JSON viewer, key-value editor, log console, split pane, menu bar, terminal, property inspector, flame graph, notifications & spotlight cards.",
    "Charts & planning: progress and concentric activity rings, bar list, OKLab heatmap, treemap, radar, scatter, conversion funnel, OHLC candlestick, Gantt roadmap & Kanban board.",
    "DevTools, motion & chrome: unified diff viewer, trace waterfall, uptime strip, rolling odometer numbers, neighbor-magnifying dock, scrubbable audio waveform & look-aware window chrome.",
    "Inputs, pickers & overlays: combobox, multi-select, time & HSV/hex color pickers, sheet/drawer, ⌘K command palette, hover card, context menu, confirm popover, text area, scrubbable number field, tag input, OTP code, password strength, search field, alert/banner, skeleton, empty state, breadcrumb, pagination, wizard steps, radio cards, rating, dual-thumb range slider, shortcut recorder, split/hold/copy buttons, dropzone, toolbar, status bar & segmented meter.",
    "Ethereal: the Lumen look takes the site's night-sky palette, 36 px controls, 16 px glass cards with a light top edge and a pointer-following accent glow.",
    "New: filter chips, stepper, toggle group, avatar and avatar group.",
    "Motion: lines and sparklines draw themselves in and their end dot pops; select rows arrive in sequence, the check springs in, and a new value rolls into the trigger.",
    "Slider can fill its container (.width(f32::INFINITY)).",
    "Layers: interactive overlays with hit testing, outside-click and Escape dismissal.",
    "Popover, menu, select, dialog, toast, tabs, radio, progress, spinner, badge and status.",
    "Calendar, range calendar, date picker and date range picker; lumen::Date.",
    "Charts: line, area, grouped and stacked bars, donut, gauge, sparkline, stat tiles.",
    "Anti-aliased mesh painting: polylines, gradient fills, arc bands.",
    "Looks: Lumen, macOS, Windows 11 and GNOME presets that blend when switched; custom and native accents; token overrides; segmented control.",
    "OS integration: live dark mode, system accent color and reduce-motion.",
  ]},
  { v: "Phase 1", date: "2026-11-17", items: [
    "Core: OKLab color, theme tokens, closed-form springs, flex layout, painter, one-shader wgpu renderer.",
    "Widgets: button, switch, checkbox, slider, text input, card, settings rows, sidebar, tooltip, accent picker.",
    "winit app for native and web, clipboard, headless screenshots, gallery.",
  ]},
];
