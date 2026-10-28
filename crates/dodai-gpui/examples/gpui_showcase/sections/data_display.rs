//! The "Data display" category sections.

use super::super::*;

impl Showcase {
    /// The builders of this category, one per group of sections (built lazily).
    pub(crate) fn groups_data_display() -> Vec<Group> {
        vec![
            Self::tags_lists_section,
            Self::misc_section,
            Self::list_divider_section,
            Self::timeline_section,
            Self::tooltip_section,
            Self::table_section,
            Self::code_section,
            Self::image_section,
            Self::progress_section,
            Self::clock_section,
            Self::scroll_section,
        ]
    }

    fn tags_lists_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let tags = row()
            .child(Tag::new("online", Tone::Success).solid(true))
            .child(Tag::new("warn", Tone::Warning).solid(true))
            .child(Tag::new("error", Tone::Danger).solid(true))
            .child(Tag::new("info", Tone::Info).solid(true))
            .child(Tag::new("online", Tone::Success))
            .child(Tag::new("warn", Tone::Warning))
            .child(Tag::new("error", Tone::Danger))
            .child(Tag::new("info", Tone::Info));
        let lists = div()
            .flex()
            .flex_col()
            .child(Muted::new("One line"))
            .child(ListItem::new("l1", "Inbox").icon(IconName::Home))
            .child(Muted::new("Two lines · trailing text"))
            .child(
                ListItem::new("l2", "Weekly digest")
                    .supporting("Everything that changed this week")
                    .trailing("10:24")
                    .leading(dodai_gpui::Avatar::new("WD").size(40.0)),
            )
            .child(Muted::new("Three lines"))
            .child(
                ListItem::new("l3", "Design review")
                    .overline("Tomorrow")
                    .supporting("Bring the colour schemes and the new app bars")
                    .icon(IconName::Heart),
            );
        let chips = row()
            .child(dodai_gpui::Chip::new("c-assist", "Assist").leading(IconName::Edit))
            .child(
                dodai_gpui::Chip::new("c-assist-e", "Elevated")
                    .leading(IconName::Edit)
                    .elevated(true),
            )
            .child(dodai_gpui::Chip::new("c-filter-off", "Filter").filter(true))
            .child(
                dodai_gpui::Chip::new("c-filter-on", "Filter")
                    .filter(true)
                    .selected(true),
            )
            .child(
                dodai_gpui::Chip::new("c-input", "Input")
                    .leading(IconName::Heart)
                    .on_remove(|_, _, _| {}),
            )
            .child(dodai_gpui::Chip::new("c-sugg", "Suggestion"));
        out.push((
            "Data display",
            "Tags · Chips · Badges",
            self.section("Tags", cx).child(tags).into_any_element(),
            "Tags",
        ));
        out.push((
            "Data display",
            "List items · Chips",
            self.section("List items · Chips", cx)
                .child(lists)
                .child(chips)
                .into_any_element(),
            "List items · Chips",
        ));
        out
    }

    fn misc_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let (m1, m2) = (me.clone(), me.clone());
        let w_misc = row()
            .child(
                Rating::new("rating", self.rating)
                    .half(true)
                    .on_change(move |v, _, cx| {
                        m1.update(cx, |s, cx| {
                            s.rating = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                NumberStepper::new("step", self.stepper)
                    .range(0, 20)
                    .on_change(move |v, _, cx| {
                        m2.update(cx, |s, cx| {
                            s.stepper = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                div()
                    .id("rich")
                    .text_color(hsla(t.palette.link))
                    .tooltip(RichTooltip::build(
                        "Rich tooltip",
                        "Title, body and an action.",
                        Some("Learn more"),
                    ))
                    .child("Hover (rich)"),
            )
            .child(
                IconBadge::new(IconButton::new("bell", "🔔").icon(IconName::Bell))
                    .count(12, Tone::Danger),
            )
            .child(
                IconBadge::new(IconButton::new("mail", "✉").icon(IconName::Mail))
                    .dot(Tone::Success),
            );
        out.push((
            "Data display",
            "Badges · Rating · Tooltips",
            self.section("Rating · Stepper · Badges · Tooltips", cx)
                .child(w_misc)
                .into_any_element(),
            "Rating · Stepper · Badges · Tooltips",
        ));
        out
    }

    fn list_divider_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
        let mut list = div().flex().flex_col().gap(px(t.spacing.xs));
        for (i, name) in ["Inbox", "Drafts", "Sent", "Archive"]
            .into_iter()
            .enumerate()
        {
            let m = me.clone();
            list = list.child(
                ListItem::new(("li", i), name)
                    .selected(self.list == i)
                    .on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.list = i;
                            cx.notify();
                        })
                    }),
            );
        }
        out.push((
            "Data display",
            "List · Divider",
            self.section("List · Divider", cx)
                .child(list)
                .child(Divider::new())
                .into_any_element(),
            "List · Divider",
        ));
        out
    }

    fn timeline_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let timeline = Timeline::new([
            TimelineEvent::new("Charts added")
                .time("today")
                .tone(Tone::Success),
            TimelineEvent::new("Search view")
                .time("today")
                .detail("fuzzy + highlight"),
            TimelineEvent::new("Build failed")
                .time("earlier")
                .tone(Tone::Danger),
        ]);
        out.push((
            "Data display",
            "Timeline · Empty state",
            self.section("Timeline · empty state", cx)
                .child(
                    row()
                        .child(div().flex_1().min_w(px(220.0)).child(timeline))
                        .child(
                            EmptyState::new("∅", "Nothing here")
                                .message("Add something to get started."),
                        ),
                )
                .into_any_element(),
            "Timeline · empty state",
        ));
        out
    }

    fn tooltip_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        vec![(
            "Data display",
            "Tooltip · Rich tooltip",
            self.section("Tooltip", cx)
                .child(
                    div()
                        .id("tip")
                        .w(px(220.0))
                        .text_color(hsla(t.palette.link))
                        .tooltip(DodaiTooltip::text("Themed tooltip"))
                        .child("Hover me"),
                )
                .child(div().flex().child(DodaiTooltip::view("Plain tooltip", cx)))
                .child(div().flex().child(RichTooltip::view(
                    "Rich tooltip",
                    "Supporting text, 12 px corners, elevation 2.",
                    Some("Learn more"),
                    cx,
                )))
                .into_any_element(),
            "Tooltip",
        )]
    }

    fn table_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let dt_rows: std::rc::Rc<Vec<Vec<SharedString>>> = std::rc::Rc::new(
            (0..300)
                .map(|i| {
                    let roles = ["Engineer", "Designer", "Analyst", "Manager"];
                    let regions = ["EMEA", "APAC", "AMER"];
                    vec![
                        SharedString::from(format!("User {:03}", (i * 37) % 300)),
                        SharedString::from(roles[i % 4]),
                        SharedString::from(regions[(i / 3) % 3]),
                        SharedString::from(format!("{}", (i * 7919) % 1000)),
                    ]
                })
                .collect(),
        );
        let dt_filter_text = self.dt_filter.read(cx).text().to_string();
        let data_table = col()
            .child(
                row()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(200.0))
                            .child(self.dt_filter.clone()),
                    )
                    .child(Muted::new(
                        "click a header to sort · drag a column edge · click rows to select",
                    )),
            )
            .child(
                dodai_gpui::DataTable::new("dt")
                    .columns([
                        dodai_gpui::Column::new("Name").width(160.0),
                        dodai_gpui::Column::new("Role").width(140.0),
                        dodai_gpui::Column::new("Region").width(110.0),
                        dodai_gpui::Column::new("Score").width(100.0).numeric(true),
                    ])
                    .rows_shared(dt_rows)
                    .filter(dt_filter_text)
                    .height(260.0),
            );
        out.push((
            "Data display",
            "Data table · sort filter select resize virtual scroll",
            self.section("Data table", cx)
                .child(data_table)
                .into_any_element(),
            "Data table",
        ));
        out
    }

    fn code_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let mut out: Vec<Section> = Vec::new();
        let md_src = "# Markdown\n\nSupports **bold**, *italic*, `inline code` and [links](https://example.com).\n\n- headings and paragraphs\n- nested lists\n  - indent two spaces\n    1. numbered inside\n    2. restarts per level\n- fenced code and tables\n\n| Token | Value | Note |\n|:--|--:|:-:|\n| radius | 12 | rounded |\n| chamfer | 0 | none |\n\n> quotes read muted\n\n```rust\nfn main() {\n    let n: u32 = 42; // answer\n    println!(\"{n}\");\n}\n```";
        out.push((
                "Data display",
                "Code block · Markdown · rich text",
                self.section("Code block · Markdown", cx)
                    .child(
                        dodai_gpui::CodeBlock::new(
                            "code",
                            "use std::collections::HashMap;\n\n/// Count words.\nfn count(text: &str) -> HashMap<String, usize> {\n    let mut m = HashMap::new();\n    for w in text.split_whitespace() {\n        *m.entry(w.to_lowercase()).or_insert(0) += 1;\n    }\n    m // 42 buckets max\n}",
                        )
                        .language("rust"),
                    )
                    .child(dodai_gpui::Markdown::new("md", md_src))
                    .into_any_element(),
                "Code block · Markdown",
            ));
        out
    }

    fn image_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let sample = format!("{}/examples/assets/sample.png", env!("CARGO_MANIFEST_DIR"));
        out.push((
            "Data display",
            "Image · media · Avatar group",
            self.section("Image · Avatar group", cx)
                .child(
                    row()
                        .items_start()
                        .child(
                            dodai_gpui::Image::path(sample.clone())
                                .size(260.0, 160.0)
                                .caption("Image (cover)"),
                        )
                        .child(
                            dodai_gpui::Image::path(sample.clone())
                                .size(260.0, 160.0)
                                .video(true)
                                .caption("Video poster"),
                        )
                        .child(
                            dodai_gpui::Image::path(sample)
                                .size(160.0, 160.0)
                                .fit(dodai_gpui::ImageFit::Contain)
                                .caption("Contain"),
                        ),
                )
                .child(
                    row()
                        .child(dodai_gpui::AvatarGroup::new(["AL", "BK", "CM", "DN"]))
                        .child(
                            dodai_gpui::AvatarGroup::new([
                                "AL", "BK", "CM", "DN", "EO", "FP", "GQ",
                            ])
                            .max(4),
                        )
                        .child(
                            dodai_gpui::AvatarGroup::new(["AL", "BK", "CM", "DN", "EO"])
                                .max(3)
                                .size(28.0),
                        ),
                )
                .into_any_element(),
            "Image · Avatar group",
        ));
        out
    }

    fn progress_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        out.push((
            "Data display",
            "Circular progress · Banner",
            self.section("Circular progress · Banner", cx)
                .child(
                    row()
                        .child(dodai_gpui::CircularProgress::new(0.25))
                        .child(
                            dodai_gpui::CircularProgress::new(0.72)
                                .tone(Tone::Success)
                                .size(72.0)
                                .thickness(7.0),
                        )
                        .child(
                            dodai_gpui::CircularProgress::new(1.0)
                                .tone(Tone::Info)
                                .size(40.0)
                                .thickness(4.0),
                        )
                        .child(dodai_gpui::CircularProgress::indeterminate())
                        .child(dodai_gpui::CircularProgress::new(0.6).wavy(true).size(64.0))
                        .child(
                            dodai_gpui::CircularProgress::indeterminate()
                                .wavy(true)
                                .size(64.0),
                        ),
                )
                .child(
                    dodai_gpui::Banner::new("bn1", Tone::Info, "A new version is available.")
                        .action("Update", |_, _| {}),
                )
                .child(dodai_gpui::Banner::new(
                    "bn2",
                    Tone::Warning,
                    "Your trial ends in 3 days.",
                ))
                .child(
                    dodai_gpui::Banner::new("bn3", Tone::Danger, "Connection lost. Retrying…")
                        .dismissible(false),
                )
                .into_any_element(),
            "Circular progress · Banner",
        ));
        out
    }

    fn clock_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        vec![
            (
                "Data display",
                "Clock · Timer",
                self.section("Clock · Timer", cx)
                    .child(
                        row()
                            .child(dodai_gpui::Clock::new().local())
                            .child(dodai_gpui::Timer::new("tmr").minutes(1)),
                    )
                    .into_any_element(),
                "Clock · Timer",
            ),
            (
                "Data display",
                "Stopwatch · Tap tempo",
                self.section("Stopwatch · Tap tempo", cx)
                    .child(
                        row()
                            .child(dodai_gpui::Stopwatch::new("sw"))
                            .child(dodai_gpui::TapTempo::new("tap")),
                    )
                    .into_any_element(),
                "Stopwatch · Tap tempo",
            ),
        ]
    }

    fn scroll_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let mut out: Vec<Section> = Vec::new();
        out.push((
            "Data display",
            "Scroll area · styled scrollbar",
            self.section("Scroll area", cx)
                .child(dodai_gpui::ScrollArea::new("sa").height(200.0).child(
                    div().flex().flex_col().gap(px(8.0)).children(
                        (1..=40).map(|i| {
                            Label::new(format!("Row {i} · drag the thumb or use the wheel"))
                        }),
                    ),
                ))
                .into_any_element(),
            "Scroll area",
        ));
        out
    }
}
