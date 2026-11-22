//! [`Checkbox`]: a design-aware checkbox. The design system (Cyberpunk, Material, Dodai or
//! Neo-brutalism) is read from the theme's [`Look`](dodai_ui_core::Look) flags and the colours from
//! its palette tokens; the glyphs come from `tui-checkbox`.

use crate::TuiTheme;
use dodai_ui_core::{ControlState, Look};
use ratatui::{buffer::Buffer, layout::Rect, style::Color, text::Line, widgets::Widget};
use tui_checkbox::cyber::{Cyber, CyberPalette, CyberSize, Design};

/// The checkbox design of a theme, from its [`Look`] flags (never from its id or family).
///
/// Tonal themes are Material, themes with sinking hard-shadow controls Neo-brutalism, cut-corner
/// or luminous themes Cyberpunk, and everything else (soft elevation, no spec) Dodai.
pub fn checkbox_design(look: &Look) -> Design {
    if look.tonal {
        Design::Material
    } else if look.sink {
        Design::Neobrutalism
    } else if look.cut_corners || look.luminous {
        Design::Cyberpunk
    } else {
        Design::Dodai
    }
}

/// Checkbox drawn in the design system of the theme.
///
/// ```
/// use dodai_ui_ratatui::{Checkbox, TuiTheme};
/// let tui = TuiTheme::new(dodai_ui_core::Theme::default());
/// let cb = Checkbox::new(&tui, "Neural link", true).block(true);
/// assert_eq!(cb.height(), 3);
/// ```
///
/// With 256-colour terminals ([`ColorDepth::Ansi256`](crate::ColorDepth)) the gradient and the
/// tinted label bars snap to the nearest end colour, as colours cannot be mixed.
pub struct Checkbox<'a> {
    theme: &'a TuiTheme,
    label: String,
    checked: bool,
    mixed: bool,
    state: ControlState,
    block: bool,
    shadow: bool,
    readout: bool,
    design: Option<Design>,
}

impl<'a> Checkbox<'a> {
    /// A checkbox; the label honours `typography.uppercase_labels`.
    pub fn new(theme: &'a TuiTheme, label: &str, checked: bool) -> Self {
        Self {
            theme,
            label: theme.label(label),
            checked,
            mixed: false,
            state: ControlState::Idle,
            block: false,
            shadow: false,
            readout: false,
            design: None,
        }
    }

    /// Partly checked (wins over `checked` when drawing).
    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
        self
    }

    /// Interaction state: hovered, pressed and focused all light the checkbox.
    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }

    /// Three rows by five cells instead of one by three.
    pub fn block(mut self, block: bool) -> Self {
        self.block = block;
        self
    }

    /// Hard offset shadow, off by default. Only the Cyberpunk design has one; elsewhere a no-op.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    /// Status readout after the label (`//ONLINE`, `On`, ...).
    pub fn readout(mut self, readout: bool) -> Self {
        self.readout = readout;
        self
    }

    /// Forces a design instead of the theme's own.
    pub fn design(mut self, design: Design) -> Self {
        self.design = Some(design);
        self
    }

    /// The design this checkbox is drawn in.
    pub fn resolved_design(&self) -> Design {
        self.design
            .unwrap_or_else(|| checkbox_design(&self.theme.theme().look()))
    }

    /// Colours from the theme tokens: `muted` is the outline, `ink` the mark on a filled box.
    fn palette(&self) -> CyberPalette {
        let th = self.theme.theme();
        let p = &th.palette;
        let c = |x| self.theme.color(x);
        // Only the Cyberpunk design draws a shadow: the background warmed towards the primary.
        let shadow: Color = c(p.background.mix(p.primary, 0.3));
        CyberPalette {
            primary: c(p.primary),
            secondary: c(p.secondary),
            // The unchecked fill: raised surface lifted towards the outline so it reads on the page.
            surface: c(p.surface_raised.mix(p.border_strong, 0.45)),
            text: c(p.text),
            muted: c(p.border_strong),
            ink: c(p.on_primary),
            shadow,
        }
    }

    /// The `tui-checkbox` widget this draws, for layout or further styling.
    pub fn widget(&self) -> tui_checkbox::Checkbox<'static> {
        let design = self.resolved_design();
        let cyber = Cyber::for_design(design)
            .size(if self.block {
                CyberSize::Block
            } else {
                CyberSize::Inline
            })
            .shadow(self.shadow)
            .readout(self.readout)
            // The label is already cased by `typography.uppercase_labels`.
            .uppercase(false)
            .glow(design == Design::Cyberpunk)
            .palette(self.palette());
        tui_checkbox::Checkbox::new(self.label.clone(), self.checked)
            .indeterminate(self.mixed)
            .focused(matches!(
                self.state,
                ControlState::Focused | ControlState::Hovered | ControlState::Pressed
            ))
            .disabled(self.state == ControlState::Disabled)
            .cyber(cyber)
    }

    /// Width in cells (see [`tui_checkbox::Checkbox::width`]).
    pub fn width(&self) -> u16 {
        self.widget().width()
    }

    /// Height in rows.
    pub fn height(&self) -> u16 {
        self.widget().height()
    }

    /// The first line only, for inline use in a list row.
    pub fn line(&self) -> Line<'static> {
        let area = Rect::new(0, 0, self.width(), 1);
        let mut buf = Buffer::empty(area);
        self.widget().render(area, &mut buf);
        let spans = (0..area.width)
            .map(|x| {
                let cell = &buf[(x, 0)];
                ratatui::text::Span::styled(cell.symbol().to_string(), cell.style())
            })
            .collect::<Vec<_>>();
        Line::from(spans)
    }
}

impl Widget for Checkbox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.widget().render(area, buf);
    }
}
