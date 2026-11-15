# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2026-11-15
### ✨ Features
- glass on every container and input field in every toolkit (gpui panes, popups, pickers, toasts; iced fields; bevy inputs; web fields/lists; yew, leptos, egui, freya)
- alerts and tooltips take the glass; floating surfaces keep a minimum opacity where content behind cannot be blurred
### 🐛 Bug Fixes
- opaque floating popups in dark Dodai, Select::popup_surface frost/opacity
- frosted glass on popover, command palette, autocomplete menu, date/time/color pickers and toasts
### 📚 Documentation
- don't intra-doc-link the feature-gated schema()
**Full Changelog**: https://github.com/sorinirimies/dodai-ui/compare/v0.1.1...v0.1.3
## [0.1.1] - 2026-11-15
### ✨ Features
- per-design spinner sets (Dodai quad/grid/orbit/courses, Cyberpunk glitch/scanner/data/bars, Neo-Brutalism shuffle/hop/pillars/steps, Material 3 ring/petals/bounce); gpui Spinner .style()/.size(); showcase shows them
- shared base set plus per-design custom ones; Material 3 arc and loading indicators under Spinners; cyberpunk reticle
- Material 2 classic arc, Dodai sweep ring
- per-design progress bars (hud, stripes, stones), M3 wavy indeterminate; smoother arc spinners
- iced draws Classic/Sweep as real arcs; web Courses stays in its box
- group shared vs per-design spinners and progress; squircle Sweep; circular progress only in Material
- skeleton placeholders live in the shared card
- shared/per-design Loading page in every toolkit (model, web, dioxus, iced, bevy); styled progress bars; squircle Sweep
- Material wavy/circular progress in every model toolkit; spinners for egui, slint, freya, yew, leptos
### 🐛 Bug Fixes
- real stroked arcs for Classic and Sweep, Courses stays in its box, Quad squares no longer overlap
- bevy styled bars update in place (no flicker), web shuffle offset, iced DODAI_SIZE
### 🔄 CI
- msrv job retries the flaky toolchain install
### 🔧 Chores
- drop unused workspace dependencies (cargo warnings on every run)
- drop duplicate top spinner demo
- bump freya and shader-kernels literal versions to 0.1.1
### 🧪 Testing
- arc dot count
**Full Changelog**: https://github.com/sorinirimies/dodai-ui/compare/v0.1.0...v0.1.1
## [0.1.0] - 2026-11-12
### ♻️  Refactor
- drop sample themes, keep Spec builder ready for new themes
- split charts_extra into family modules (trend, flow, distribution, part_whole)
- responsive structured layout (min_w_0, max-width centered body, narrow sidebar, header chips row)
- merge design system and theme into one panel
- single dodai-shaders crate for CPU and GPU (gpu feature + rust-gpu build of the same crate); drop dodai-shaders-gpu and -kernels
### ⚡ Performance
- optimise deps in dev profile (6->55 fps), alloc-free entry ids, keep looping demos off All page; DODAI_AUTOSCROLL fps probe
- cached bind groups, lock-free pipeline cache with background prewarm; gpui opens the GPU off the UI thread (CPU until ready)
- cull off-screen sections and icon groups (lazy), pace shader renders, compile GPU pipelines on demand, redraw shaders at their fps
- pies with different looks no longer share state (tween never settled, redrew every frame); next_frame() tally for finding what keeps redrawing
### ✨ Features
- legendary themes design system with Cyberpunk theme for ratatui, iced, gpui and topcoat
- chamfered canvas button with hover/press/glow
- 3-row chamfered slab buttons
- configurable glow intensity; iced button glow no longer clipped
- shape kinds/corners, component tokens, Spec builder, 4 new themes
- complete themed component library with shaped widgets
- complete themed component library
- theme-driven component parity
- full component CSS, per-theme data-theme blocks
- themed components over topcoat CSS, SSR showcase
- themed components over topcoat CSS, SSR showcase
- themed components over topcoat CSS, SSR showcase
- themed component library and showcase
- generated .slint theme library and showcase
- themed components and showcase
- themed UI plugin, shader chamfers, showcase
- table, feedback, menu, nav, dialog, select, theme selector, text input
- Knob, vertical Slider direction; docs: component coverage
- per-component scanline texture toggle (core with_scanlines, gpui set_scanlines)
- Card, IconButton, Fab, Navigation (bar/rail/drawer), TopAppBar, SearchBar
- design-system (family) selector; showcase uses dropdowns only
- sheets, full-screen dialog, snackbar, range slider, carousel
- DatePicker, TimePicker
- toolbar, empty state, rating, stepper, timeline, color picker, drop zone, tag field, sortable table
- TreeView, SplitPane
- CommandPalette, Combobox
- SearchView, RichTooltip, IconBadge
- charts (line, bar, pie/donut, sparkline, gauge, radar, scatter, heatmap), more showcase, docs
- merge new components and charts into main showcase
- slider ticks/readout/format, vertical range slider; showcase sidebar with categories + filter
- slider value bubble above/below thumb
- chart config (format, y-range, title, stepped, value labels), aligned line x-axis, sidebar up/down nav
- 9 new chart types (stacked area, histogram, waterfall, funnel, candlestick, box plot, radial bar, bubble, treemap) + showcase
- pie/donut slice gaps and hover pop-out; showcase DODAI_CAT/DODAI_ONLY dev env
- motion primitives (Easing, Motion, composable Curve), sparkline tip markers
- export Marker, pie_hit
- PieChart selection (click toggle, controlled .selected/.on_select) with outline
- entry fade for all charts, Tween, animated pie pop, configurable selected_color (default accent)
- entry fade on 44 non-chart components via shared motion::entry
- EntryKind (fade/rise/drop/slide/none), grow() value-in for progress, slider, range, knob, gauge, bar, line, pie, radial, histogram
- Animation showcase category (settings, easing curves, loading, docs), global + per-section entry dropdowns, clamp overshoot easing
- EntryScope — per-section entry style overrides every component inside
- composable EntrySet; per-section effect chips (add/remove to compose)
- per-section Animations chips in a right-hand column
- dodai-icons crate (framework-agnostic icon geometry) + gpui Icon, sidebar icons
- per-effect duration via hover popover on animation pills; section content no longer shrinks
- Panel::subheader; animation pills below each section title with per-effect duration hover popover
- glow slider + scanlines toggle in Design system panel
- per-section replay (no global reset) + play/loop/stop player; Repeat/Stop icons
- Panel::scope applies entry effects to inner components only; drop Global chip
- configurable fill/value-in duration (EntrySet::with_grow_ms) + Fill pill popover per section
- entry animation on Snackbar, Toast, DialogCard, Legend, Sparkline
- entry animation on text components; showcase Loading page (spinner, indeterminate, progress, skeleton)
- Spring easing, stagger delay (per-section Stagger pill), Spring panel on Animation page
- dense canvas Heatmap with hover readout; showcase heatmap 24x64; Loading panel has no animation pills
- Playback (Loop/Once/Stopped) for Spinner/IndeterminateBar/Skeleton; Loading player with duration
- value-in animation for funnel, waterfall, candlestick, box plot, bubble, treemap, stacked area, radar, scatter, dense heatmap; CountUp; gauge count-up label
- eased transitions for tabs, toggle, checkbox, radio, accordion, slider thumb
- button hover/press blend + press nudge; skeleton shimmer sweep
- Play button deselects when the run ends
- shape-adaptive shimmer via ShapeStyle::with_shimmer; Button::shimmer; skeleton uses it
- per-section animation controls only where they apply
- real single/multi-line text inputs, drop legacy dodai_input demo
- physics easings (bounce, elastic, back, anticipate, spring presets), spring toggle thumb; sidebar subsections (Animation: Entry/Physics)
- subsection headings in page body; category/subsection click scrolls to top
- ResponsiveShell (collapsing sidebar drawer) + DesignSystemTree (tree/dropdown); showcase uses both, glow+scanlines in one row
- DesignSystemTree::properties; glow/scanlines shown under Cyberpunk
- DesignSystemTree children collapsible by clicking the active system
- tree expansion is global; picking a section re-opens design systems
- per-button cut corners (Corners presets + set/get), showcase picker
- Panel/Card .corners(), theme-wide set_corners(), showcase controls
- .corners() on all shaped components (Tag, Select, Toast, Tabs, Menu, ...)
- ShapeSystem — corner primitives, scale, roles, presets, lerp, outline
- composable ShapeSystem (per-role kind, with_role/scale/style, Px size), Theme::with_shape_system/shape_for
- dodai-charts-core (pure chart logic) + gpui components/charts features; CI, publish order, dioxus+bevy jobs
- RenkoChart, DepthChart, Gantt, ChartStack, LineChart secondary y axis
- candlestick extras (Heikin-Ashi, OHLC, volume, overlays, RSI/MACD panes, markers) + heatmap gradients
- dodai-shaders crate (56 CPU shaders: cyberpunk, Material 3, Matrix, retro, pixel art, nature), gpui ShaderView + Shaders showcase category, standalone gallery
- 34 new shaders (90 total), Player (fps cap, adaptive resolution), faster scheduling/bloom/fireworks/pico16, half-block export, bench
- Ghostty family (17 Rust ports), 19 more retro terminal/monitor shaders, presets, gallery + showcase integration (126 shaders)
- dropdown selection, source/effect tags, chain chips in playground; gallery legend
- sharper default (pixel 2, up to 960x540, 10ms budget)
- GPU route - same Rust shaders compiled to WGSL via rust-gpu, wgpu host crate, ShaderView GPU path with CPU fallback
- Blade Runner-inspired scenes and effects, rebuilt rain city (137 shaders, GPU kernels regenerated)
- Ghibli family: 6 painted 2D scenes, 2 cel-shaded 3D scenes, 2 post effects, presets (147 shaders, GPU kernels regenerated)
- 8 more pixel-art scenes and 3 pixel post effects (158 shaders, GPU kernels regenerated)
- own showcase category; redo meadow/clouds/forest; 6 new scenes, 5 new effects, presets (169 shaders)
- DMT Hyperspace section (sacred geometry, visionary-style figure/eye, 7 effects, 8 presets) + Mandelbrot zoom in Cosmic; 187 shaders
- glow_waves generator, a palette variation of the Shadertoy wave shader
- top bar fills the width; Shaders sub-groups by kind (Playground, Sources, Effects, Presets)
- every shader shows on every design; Shaders sections named for what they look like
- shaders grouped by look (8 source groups, 6 effect groups) with the family as a chip; groups table in dodai-shaders
- featured shaders first in each group, the rest by name
- NavigationItem::expanded (tree branch chevron); showcase sidebar folds categories and shader groups
- playground above All with no count; shader entries count shaders and presets
- ShaderView::transparent (light on nothing, for overlays); Waves & flow cards use it; glow_waves loses its checkerboard; card pictures no longer collapse
- add the dodai-ui app icon (glass layers on a gradient foundation) with generated platform assets and a macOS Dock icon
- new dodai-ui logo - a UI (header, sidebar, cards) built on its dodai, the timber sill frame of a Japanese house on its foundation stone
- Dodai UI logo as window icon and beside the title
### 🐛 Bug Fixes
- match cyberpunk.net button shape (bottom-right chamfer only), rename examples
- solid slab buttons, headless HTML dump for screenshots
- outline button uses block-glyph frame with cut corner
- top-align panel children
- tab indicator follows cut corner
- pie slices tessellated as convex wedges, no gaps
- pie wedge deref compile fix
- donut drawn as filled annular sectors, no stroke artefacts
- pie/donut slice gaps are constant-width parallel straight
- pie wedges split only past half a turn, fewer AA seams
- key body by category so entry animations replay on every section click
- compact per-section animation chips
- entry wrapper is a flex row so children keep content width
- sections full width, animation chips overlay top-right
- section wrapper is flex column so sections fill width
- entry wrapper flex column keeps children stretching; ListItem icon; showcase category icons; full-width body
- entry wrapper flex column so panels stretch horizontally
- replay re-keys only inner components (Panel::scope_key) so the panel no longer flickers
- fill duration enables grow without entry effect; replay on fill/stagger change; Play shows selected
- loop/play length from entry+fill+stagger, no hardcoded gap
- pin libc 0.2.189 (0.2.190 removed ENOATTR; xattr 0.2.3 via gpui breaks on Linux)
- Fill/Stagger chips toggle on click
- update tests/examples for colour theme names
- no animation controls on shader sections
- rain/snow/matrix_depth moved up; blade rain head on leading end; flow regression test + example; kernels regenerated
- energy_rise bands are level and soft; README: no run-shaders recipe
- the section jump rows go under the chosen sub-group, not after the last one
- use PI in energy_rise (clippy approx_constant)
- searchlights never drew (beam got the horizon as its y); example listing shaders that do not move
### 📚 Documentation
- add changelog
- update component coverage
- readme motion section + push-all
- describe Cyberpunk as a cyberpunk-inspired theme
- fix unresolved GaugeStyle intra-doc link (rustdoc -D warnings)
- gpui features, motion; ci comment
- gpui gap analysis across the four designs
- gap status
- add the Audit and Fuzz badges
- this repo's agent rules as a project skill in .agents/skills
- the agent skill for this repo lives in the global skills repo again
- showcases in the other toolkits
- 0.1.0 changelog covers spinners, icon picker, menus, All sidebar
### 🔄 CI
- per-crate jobs (base core/icons first, dependents need base); fix rustdoc link errors
- drop needless drop(queue) (clippy drop_non_drop on Linux), run macOS jobs only on GitHub
- kernels entries sync job + manual/weekly full wgsl rebuild check
- harden cargo network retries on the flaky self-hosted runner
- retry the rust toolchain install once (static.rust-lang.org downloads flake on the runner)
- bump rustls-webpki, ignore slint-pinned quick-xml advisories
- merge pull requests when CI is green, leave Dependabot majors for review
- merge Dependabot major bumps too when CI is green
- nightly update also takes major bumps (merged only when green)
- move macOS jobs to their own job (matrix not allowed in job-level if)
- add Forgejo Actions workflows for Codeberg runners
- Forgejo release script, optional sudo on Codeberg runners
- rustfmt and clippy fixes for stable 1.100; msrv checks topcoat lib only
### 🔧 Chores
- rename project to legendary-ui
- scaffold egui/slint/dioxus/leptos/yew/bevy adapter crates
- move dioxus/leptos examples into their crates
- unused var
- multi-remote push/pull/release recipes (from tokenburn)
- force/per-remote variants, fast-fail ssh, gh check (from pi-tokenburn)
- sync Cargo.lock (dodai-charts-core dependency)
### 🧪 Testing
- fix per-effect duration expectation
- headless render smoke test (22 components x every built-in theme)
- publish-order test checks ordering, not a stale crate count
- shared tests check contracts, not design names; per-design assertions move into each design's own tests
- showcase category count allows Surface effects
