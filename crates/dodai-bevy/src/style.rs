//! Pure token → style mapping (no ECS), shared by the systems and the tests.

use dodai_core::{Color as Rgba, Theme};

use crate::widgets::{TextRole, ToggleKind};

/// Resolved text style. `color == None` means "inherit the owner's foreground".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// Font size px.
    pub size: f32,
    /// Fixed colour, or inherited.
    pub color: Option<Rgba>,
    /// Display UPPERCASE.
    pub upper: bool,
}

/// Map a role to a style from the theme's typography tokens.
pub fn text_style(theme: &Theme, role: TextRole) -> TextStyle {
    let ty = &theme.typography;
    let p = &theme.palette;
    let up = ty.uppercase_labels;
    let (size, color, upper) = match role {
        TextRole::Heading => (ty.size_xl, Some(p.text), up),
        TextRole::Subheading => (ty.size_lg, Some(p.text), up),
        TextRole::Body => (ty.size_md, Some(p.text), false),
        TextRole::Label => (ty.size_sm, Some(p.text), up),
        TextRole::Muted => (ty.size_sm, Some(p.text_muted), false),
        TextRole::Mono => (ty.size_sm, Some(p.text_muted), false),
        TextRole::Caption => (ty.size_xs, Some(p.text_muted), false),
        TextRole::Control => (ty.size_sm, None, up),
        TextRole::ControlSmall => (ty.size_xs, None, up),
        TextRole::Content => (ty.size_md, None, false),
    };
    TextStyle { size, color, upper }
}

/// The string to display for `raw` under this style.
pub fn display_text(raw: &str, style: &TextStyle) -> String {
    if style.upper {
        raw.to_uppercase()
    } else {
        raw.to_owned()
    }
}

/// Outer size `(w, h)` of a toggle control, from shape tokens.
pub fn toggle_size(theme: &Theme, kind: ToggleKind) -> (f32, f32) {
    let h = (theme.shape.control_height * 0.5).max(16.0);
    match kind {
        ToggleKind::Switch => (h * 2.0, h),
        _ => (h, h),
    }
}

/// Padding of a control: `(horizontal, vertical)`.
pub fn control_padding(theme: &Theme) -> (f32, f32) {
    (theme.spacing.lg, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{rounded_light, square_theme};
    use dodai_core::themes;

    #[test]
    fn uppercase_follows_token_not_id() {
        let mut t = rounded_light();
        let s = text_style(&t, TextRole::Control);
        assert!(!s.upper);
        assert_eq!(display_text("go", &s), "go");
        t.typography.uppercase_labels = true;
        let s = text_style(&t, TextRole::Control);
        assert_eq!(display_text("go", &s), "GO");
        // body / content stay raw
        assert!(!text_style(&t, TextRole::Body).upper);
        assert!(!text_style(&t, TextRole::Content).upper);
    }

    #[test]
    fn sizes_come_from_typography() {
        for t in themes::all()
            .into_iter()
            .chain([rounded_light(), square_theme()])
        {
            assert_eq!(text_style(&t, TextRole::Heading).size, t.typography.size_xl);
            assert_eq!(text_style(&t, TextRole::Caption).size, t.typography.size_xs);
            assert_eq!(
                text_style(&t, TextRole::Muted).color,
                Some(t.palette.text_muted)
            );
            assert_eq!(text_style(&t, TextRole::Control).color, None);
        }
    }

    #[test]
    fn toggle_size_scales_with_control_height() {
        let (w, h) = toggle_size(&rounded_light(), ToggleKind::Switch);
        assert_eq!((w, h), (40.0, 20.0));
        let (w, h) = toggle_size(&square_theme(), ToggleKind::Checkbox);
        assert_eq!((w, h), (16.0, 16.0));
    }
}
