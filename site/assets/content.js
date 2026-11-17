// Site content: components and guides. Rendered by app.js.

export const REPO = "https://github.com/amiralibg/lumen";

export const GROUPS = [
  "Actions",
  "Inputs",
  "Selection",
  "Navigation",
  "Overlays",
  "Feedback",
  "Dates",
  "Charts",
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
      [".disabled(bool) .full_width()", "Disabled buttons ignore the pointer and keyboard."],
      [".tooltip(text) .tooltip_shortcut(text, keys)", "Shown after a delay; instant while moving between tooltips."],
      [".clicked() .show()", "Add the button and read its Response."],
    ],
    keys: [["Space / Enter", "Activate"], ["Tab", "Move focus; the ring shows only for keyboard focus"]],
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
  { v: "Unreleased", date: "2026-11-17", items: [
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
