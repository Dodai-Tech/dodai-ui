//! Text input, pick list and menu styles.

use iced::{
    overlay::menu,
    widget::{pick_list, text_input},
    Background, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme};

impl IcedTheme {
    /// Text input style (active / hovered / focused / disabled).
    pub fn text_input(
        &self,
    ) -> impl Fn(&IcedBaseTheme, text_input::Status) -> text_input::Style + 'static {
        let idle = self.theme.input(false);
        let focus = self.theme.input(true);
        let p = self.theme.palette.clone();
        let this = self.clone();
        // The glass the surface gives input fields (see-through; blur: [`IcedTheme::field`]).
        let glass = self.field_glass();
        move |_t, status| {
            let (t, border, value, bg) = match status {
                text_input::Status::Focused { .. } => {
                    (focus, focus.border, focus.text, focus.background)
                }
                text_input::Status::Hovered => (idle, p.text_muted, idle.text, idle.background),
                text_input::Status::Disabled => (idle, p.border, p.text_disabled, p.surface_raised),
                text_input::Status::Active => (idle, idle.border, idle.text, idle.background),
            };
            text_input::Style {
                background: Background::Color(color(glass.color(bg))),
                border: this.border(border),
                icon: color(t.placeholder),
                placeholder: color(t.placeholder),
                value: color(value),
                selection: color(t.selection),
            }
        }
    }

    /// Pick-list (closed field) style.
    pub fn pick_list(
        &self,
    ) -> impl Fn(&IcedBaseTheme, pick_list::Status) -> pick_list::Style + 'static {
        let idle = self.theme.input(false);
        let focus = self.theme.input(true);
        let muted = self.theme.palette.text_muted;
        let this = self.clone();
        let glass = self.field_glass();
        move |_t, status| {
            let border = match status {
                pick_list::Status::Active => idle.border,
                pick_list::Status::Hovered => muted,
                pick_list::Status::Opened { .. } => focus.border,
            };
            pick_list::Style {
                text_color: color(idle.text),
                placeholder_color: color(idle.placeholder),
                handle_color: color(muted),
                background: Background::Color(color(glass.color(idle.background))),
                border: this.border(border),
            }
        }
    }

    /// Drop-down menu (pick list / combo box overlay) style.
    pub fn menu(&self) -> impl Fn(&IcedBaseTheme) -> menu::Style + 'static {
        let p = self.theme.palette.clone();
        let sel = self
            .theme
            .list_item(dodai_ui_core::ItemState::SELECTED_HOVERED);
        let border = self.border(p.border_strong);
        let shadow = self.shadow(false, true);
        move |_| menu::Style {
            background: Background::Color(color(p.surface)),
            border,
            text_color: color(p.text_muted),
            selected_text_color: color(sel.foreground),
            selected_background: Background::Color(color(sel.background.unwrap_or(p.selection))),
            shadow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_ui_core::Theme;

    fn ti(t: Theme, s: text_input::Status) -> text_input::Style {
        let lt = IcedTheme::new(t);
        lt.text_input()(&lt.iced_theme(), s)
    }

    #[test]
    fn input_states() {
        let t = cyberpunk();
        let (idle_t, focus_t) = (t.input(false), t.input(true));
        let a = ti(cyberpunk(), text_input::Status::Active);
        assert_eq!(a.border.color, color(idle_t.border));
        let f = ti(
            cyberpunk(),
            text_input::Status::Focused { is_hovered: false },
        );
        assert_eq!(f.border.color, color(focus_t.border));
        assert_ne!(a.border.color, f.border.color);
        let h = ti(cyberpunk(), text_input::Status::Hovered);
        assert_ne!(h.border.color, a.border.color);
        let d = ti(cyberpunk(), text_input::Status::Disabled);
        assert_eq!(d.value, color(cyberpunk().palette.text_disabled));
    }

    #[test]
    fn input_radius_adapts() {
        assert_eq!(
            ti(rounded_light(), text_input::Status::Active)
                .border
                .radius,
            8.0.into()
        );
        assert_eq!(
            ti(hard_square(), text_input::Status::Active).border.radius,
            0.0.into()
        );
    }

    #[test]
    fn pick_list_and_menu() {
        let lt = IcedTheme::new(rounded_light());
        let p = lt.pick_list()(
            &lt.iced_theme(),
            pick_list::Status::Opened { is_hovered: false },
        );
        assert_eq!(p.border.color, color(lt.theme().input(true).border));
        assert_eq!(p.border.radius, 8.0.into());
        let m = lt.menu()(&lt.iced_theme());
        assert_eq!(
            m.background,
            Background::Color(color(lt.theme().palette.surface))
        );
        assert_eq!(m.border.radius, 8.0.into());
        let sel = lt
            .theme()
            .list_item(dodai_ui_core::ItemState::SELECTED_HOVERED);
        assert_eq!(m.selected_text_color, color(sel.foreground));
    }
}
