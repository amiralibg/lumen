# Component roadmap

lumen now ships **171 live components across 14 groups** (up from 28) and seven complete apps, completing every item in Batch 2, Batch 3, Batch 4, and Batch 5 plus full CUI-parity collaboration, studio, enterprise, AI, developer-tools, visualization, date, and motion components. Every widget follows the look system: geometry and motion from `ui.look()` / `ui.motion()`, colors strictly from `Token::*`, closed-form springs, reduced-motion fallbacks, and zero idle wakeups at rest.

## Design bar for every widget

- It has all of its states drawn: rest, hover, pressed, focus-visible, disabled, loading, empty and error where they apply.
- Motion is a spring or a named ease, never linear. Press feedback is a squash of about 0.96, and every motion has a reduced-motion fallback.
- It looks right in all four looks and in both themes. The screenshot review in `scripts/screenshots.sh` covers 4 looks × 2 themes.
- Keyboard first: roving focus in composites, Escape closes overlays, and focus returns to the trigger.
- The demo page shows a real use, not a lone control. "Add a runner", not just "Button".

## Batch 2: the everyday gaps (do first)

| Group | Missing |
|---|---|
| Inputs | text area, number field (scrub to change), tag input, one-time code, password + strength, search field, prefix/suffix, money field |
| Selection | radio cards, chips/filter chips, rating, toggle group, step slider, range slider (two thumbs) |
| Pickers | combobox (type to filter), multi-select, date range picker, time picker, color picker |
| Overlays | sheet/drawer, command palette, hover card, context menu, confirm popover |
| Feedback | alert/banner, skeleton, empty state, inline spinner button, undo toast |
| Navigation | breadcrumb, pagination, stepper, toolbar, menu bar, status bar |
| Basics | avatar + avatar group, keycap, divider, kbd hint, image with fallback |

## Batch 3: data

table (sort, select, sticky header), data grid (virtualized, editable cells), list + tree, accordion, timeline, reorderable list, code block (syntax colored), JSON viewer, markdown view, key-value editor, property inspector, log console.

## Batch 4: charts and planning

heatmap, calendar heatmap, treemap, radar, funnel, scatter, stacked bar, bar list, progress ring and activity rings, candlestick, sankey; kanban board, week calendar, month calendar, gantt, node editor.

## Batch 5: what will set lumen apart

- **AI interfaces**: message list with streaming text, reasoning/tool-call disclosure, citations, composer with attachments and model picker, voice waveform.
- **Developer tools**: diff viewer, terminal, network waterfall, trace/flame graph, commit graph, test explorer.
- **Motion set**: rolling numbers, text effects, dock, now-playing with waveform, spotlight cards.
- **Native chrome**: window title bar per look, traffic lights or caption buttons, native-feel menus. CUI does not have this, and it fits our looks story.

## Website parity and beyond (from the CUI teardown)

CUI's site has Overview, Guide, Components (17 groups, each page with a live example, options and keyboard table), Showcase (12 live compositions), Apps (11 full apps running live) and Changelog. Its weak spots, which we should beat:

- Its static pages are plain name-and-blurb lists with no previews. Ours should prerender every page with screenshots so crawlers and link previews get real content.
- Apps and Showcase entries have no source link, size or platform tags. Ours get "Open live", "View source", binary size and platform chips.
- Its component count does not match its index. Ours is generated from `content.js`, so it is always accurate.
- Settings panel parity: accent, theme, renderer, motion speed, pixel ratio, reduced motion and side by side. We add look, density, roundness and a custom accent picker.

## Next round (from the November 2026 CUI comparison)

CUI lists 174 component entries (several bundle more than one component) and 11 apps. Gaps worth closing, roughly in order:

- **Apps:** a settings app and a small "caffeine curve" style toy app; lumen now has mail, music, tracker, notes, assistant, finance and to-do.
- **Inputs and pickers:** signature pad, avatar upload, wheel picker and time dial, emoji picker, aspect-ratio picker, ruler slider.
- **Data and devtools:** code editor, hex viewer, tree table with path bar, file browser, problems panel, query builder, regex tester, stack trace, blame and coverage views, feature flags, dependency graph.
- **Overlays and feedback:** guided tour, veil, onboarding checklist, swipe actions, docking panels.
- **Charts:** calendar heatmap, bubble chart, chart annotations, dot-matrix display.
- **Voice AI:** recorder, voice note and live session.
- **Platform:** virtualized lists, accessibility for `lumen-gpui` (IME, overflowing menus and pipelined readback landed in November 2026).
