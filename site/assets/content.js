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
    ],
    keys: [["↑ ↓", "Move row focus"], ["Space / Enter", "Toggle focused header sort or row selection"]],
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
    summary: "Native-feel window header bar that morphs between macOS traffic lights, Windows 11 caption buttons, GNOME Adwaita and Lumen.",
    code: `ui.window_chrome("main_win", "lumen — workspace")
    .subtitle("ethereal")
    .show(|ui| {
        ui.label("Window body");
    });`,
    options: [
      [".subtitle(text) .show(|ui| ..)", "Follows ui.look() automatically and morphs when switching looks."],
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
<tr><td>Font</td><td>Inter / Geist</td><td>SF Pro</td><td>Segoe UI Variable</td><td>Adwaita Sans</td></tr></table>
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
lumen        facade</code></pre>
<p>Each frame your closure records a layout tree. Widgets resolve interaction against the previous frame's hit rects. After the closure returns, the tree is laid out (hug, fixed or fill sizing, padding, gap, align, justify), then painted into one vertex buffer. Each 64-byte vertex carries its own shape data, so a backend is one shader.</p>`,
  },
  {
    slug: "faq", title: "FAQ", lead: "Native UI, GPUI, platforms and what's next.",
    body: `
<h2>Is lumen "native UI components for Rust"?</h2>
<p>It's native code, not a web view: your app compiles to a single binary that draws with the GPU through wgpu (Metal, Vulkan, DirectX 12, OpenGL) and to WebAssembly for the browser. lumen <em>draws its own components</em>, the way egui, GPUI, iced, Slint and Flutter do. It doesn't wrap AppKit, Win32 or GTK widgets. The platform looks reproduce each OS's control conventions (sizes, shapes, focus rings, fonts, accent color, motion) closely, but they're lumen widgets. If you need the OS's actual widgets, use bindings to those toolkits.</p>
<h2>Does lumen work with GPUI?</h2>
<p>Not directly. GPUI (Zed's framework) and lumen are separate UI frameworks with their own event loops, layout and renderers, so you pick one per window. <code>lumen-core</code> only produces triangles, so you could draw a lumen UI inside a GPUI app if you wire an adapter for input and GPU textures yourself, but nothing in lumen does that today. lumen runs on winit + wgpu and also runs in the browser.</p>
<h2>Platforms?</h2>
<p>macOS, Windows, Linux (X11 and Wayland through winit), and browsers with WebGPU or WebGL2.</p>
<h2>Accessibility?</h2>
<p>Everything is keyboard reachable with visible focus. Screen reader support through AccessKit is on the roadmap.</p>
<h2>What's next?</h2>
<p>Combobox, command palette, number and tag inputs, a data table and virtualized grid, tree, code editor, and docking panels. And a password manager built with lumen.</p>`,
  },
];

export const CHANGELOG = [
  { v: "Unreleased", date: "2026-11-18", items: [
    "114 components across 14 groups: editable text, money & masked fields, @mention autocomplete, discrete step slider, transfer & reorderable lists, closable editor tabs, table of contents, nav bar, carousel, callout, undo snackbar & zoomable image frame.",
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
