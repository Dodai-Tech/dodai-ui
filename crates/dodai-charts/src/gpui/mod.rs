//! # dodai-charts
//!
//! Interactive, animated charts for [gpui](https://gpui.rs), drawn in the active Dodai design
//! (Cyberpunk, Material, Neo-Brutalism or Dodai) through `dodai-gpui`'s theme. Every chart
//! eases to new data, lifts the hovered item, keeps a click selection and shows a readout
//! line; the geometry comes from `dodai-charts`.
//!
//! Install the theme first (`dodai_gpui::install`), then place charts like any element:
//!
//! ```no_run
//! use dodai_charts::gpui::{LineChart, Series};
//! let chart = LineChart::new([Series::new("2025", [3.0, 5.0, 4.0, 8.0])]);
//! ```
#![warn(missing_docs)]

pub mod annotate;
pub mod bar;
pub mod bubble_timeline;
pub mod cascade;
pub mod common;
pub mod compare;
pub mod distribution;
pub mod extra;
pub mod fan;
pub mod flow;
pub(crate) mod focus;
pub mod grid;
pub mod hierarchy;
pub mod kpi;
pub mod layered;
pub mod line;
pub mod market;
pub mod milestones;
pub mod network;
pub mod packed;
pub mod part_whole;
pub mod peaks;
pub mod pie;
pub mod process;
pub mod pulse;
pub mod radar;
pub mod shards;
pub mod spark;
pub mod spread;
pub mod status;
pub mod survey;
pub mod trend;
pub mod voronoi;

pub use crate::fan::FanOrder;
pub use crate::gantt::Task as GanttTask;
pub use crate::shards::Outline;
pub use crate::Gradient;
pub use annotate::Annotation;
pub use bubble_timeline::{Bubble, BubbleTimeline};
pub use cascade::{FlowStep, SplitFlow};
pub use common::{
    bar_max, chart_glow, fmt_num, heat_color, heat_color_with, marker_points, nice_ticks, norm,
    pie_fractions, pie_hit, polar, set_chart_glow, smooth_points, spark_columns, BarChart,
    ChartGlow, Heatmap, Legend, LineChart, Marker, PieChart, PieStyle, RadarChart, ScatterPlot,
    Series, SparkGuide, SparkStyle, Sparkline,
};
pub use compare::{Bullet, Lollipop, PairedBars, SlopeChart};
pub use distribution::{histogram_bins, quartiles, BoxPlot, Histogram};
pub use dodai_gpui::Live;
pub use extra::{CalendarHeatmap, RoseChart, RoseScale, Sankey, ViolinChart};
pub use fan::{FanSpan, SpanFan};
pub use flow::{waterfall_steps, Funnel, Waterfall};
pub use hierarchy::{ChartNode, Icicle, Marimekko, Sunburst, Waffle};
pub use kpi::KpiTile;
pub use layered::{BumpChart, HorizonChart, RidgelineChart};
pub use market::{ChartStack, DepthChart, Gantt, RenkoChart};
pub use milestones::{Milestone, TimeScatter};
pub use network::{ArcDiagram, ChordDiagram, Hexbin, NetworkGraph, ParallelCoordinates};
pub use packed::{CirclePack, ProportionalSquares};
pub use part_whole::{treemap_layout, RadialBar, Treemap};
pub use peaks::PeakRange;
pub use process::{ControlChart, EventTimeline, StateSpan};
pub use pulse::{Pulse, PulseGrid, PulseKind, PulseMode, PulseRhythm};
pub use shards::{ShardGroup, ShardMap};
pub use spread::{Beeswarm, Ecdf, ErrorBars};
pub use status::{BucketBar, GradientScale, Slot, SlotGrid, SlotState, StageBars, StatusStrip};
pub use survey::{DivergingBars, ParetoChart};
pub use trend::{
    overlay_lines, Candle, CandleOverlay, CandlePane, CandleStyle, Candlestick, StackOffset,
    StackedAreaChart, TradeMarker,
};
pub use voronoi::{VoronoiChart, VoronoiMetric, VoronoiPoint};
