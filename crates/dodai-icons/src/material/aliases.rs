//! Familiar (Feather/Lucide-style) icon names mapped to Material Symbols
//! names, so code written against another icon set keeps working.

/// `(alias, material name)`, sorted by alias.
pub(crate) static ALIASES: &[(&str, &str)] = &[
    ("archive", "archive"),
    ("area-chart", "area_chart"),
    ("arrow-down", "arrow_downward"),
    ("arrow-left", "arrow_back"),
    ("arrow-right", "arrow_forward"),
    ("arrow-up", "arrow_upward"),
    ("bar-chart", "bar_chart"),
    ("battery", "battery_full"),
    ("bell", "notifications"),
    ("bell-off", "notifications_off"),
    ("bluetooth", "bluetooth"),
    ("bolt", "bolt"),
    ("bookmark", "bookmark"),
    ("bug", "bug_report"),
    ("calendar", "calendar_month"),
    ("camera", "photo_camera"),
    ("cart", "shopping_cart"),
    ("chat", "chat"),
    ("check", "check"),
    ("check-circle", "check_circle"),
    ("chevron-down", "keyboard_arrow_down"),
    ("chevron-left", "chevron_left"),
    ("chevron-right", "chevron_right"),
    ("chevron-up", "keyboard_arrow_up"),
    ("chevrons-left", "keyboard_double_arrow_left"),
    ("chevrons-right", "keyboard_double_arrow_right"),
    ("chip", "developer_board"),
    ("clock", "schedule"),
    ("close", "close"),
    ("cloud", "cloud"),
    ("code", "code"),
    ("copy", "content_copy"),
    ("cpu", "memory"),
    ("crosshair", "my_location"),
    ("cube", "deployed_code"),
    ("database", "database"),
    ("dot", "circle"),
    ("dots", "more_horiz"),
    ("dots-vertical", "more_vert"),
    ("download", "download"),
    ("drone", "flight"),
    ("edit", "edit"),
    ("error", "error"),
    ("external-link", "open_in_new"),
    ("eye", "visibility"),
    ("eye-off", "visibility_off"),
    ("file", "draft"),
    ("file-text", "description"),
    ("filter", "filter_alt"),
    ("flag", "flag"),
    ("flame", "local_fire_department"),
    ("folder", "folder"),
    ("gamepad", "sports_esports"),
    ("gift", "redeem"),
    ("git-branch", "call_split"),
    ("globe", "language"),
    ("grid", "grid_view"),
    ("hash", "tag"),
    ("heart", "favorite"),
    ("help", "help"),
    ("home", "home"),
    ("hourglass", "hourglass_empty"),
    ("image", "image"),
    ("info", "info"),
    ("key", "key"),
    ("layers", "layers"),
    ("layers-plus", "add_box"),
    ("line-chart", "show_chart"),
    ("link", "link"),
    ("lock", "lock"),
    ("mail", "mail"),
    ("map-pin", "location_on"),
    ("maximize", "fullscreen"),
    ("menu", "menu"),
    ("mic", "mic"),
    ("minimize", "fullscreen_exit"),
    ("minus", "remove"),
    ("moon", "dark_mode"),
    ("music", "music_note"),
    ("mute", "volume_off"),
    ("network", "lan"),
    ("next", "skip_next"),
    ("paste", "content_paste"),
    ("pause", "pause"),
    ("phone", "call"),
    ("pie-chart", "pie_chart"),
    ("pin", "keep"),
    ("pixels", "apps"),
    ("play", "play_arrow"),
    ("plus", "add"),
    ("power", "power_settings_new"),
    ("prev", "skip_previous"),
    ("print", "print"),
    ("radar", "radar"),
    ("record", "fiber_manual_record"),
    ("redo", "redo"),
    ("refresh", "refresh"),
    ("repeat", "repeat"),
    ("robot", "smart_toy"),
    ("save", "save"),
    ("search", "search"),
    ("send", "send"),
    ("server", "dns"),
    ("settings", "settings"),
    ("share", "share"),
    ("shield", "shield"),
    ("shield-check", "verified_user"),
    ("sidebar", "left_panel_open"),
    ("signal", "signal_cellular_alt"),
    ("skull", "skull"),
    ("sliders", "tune"),
    ("sort", "sort"),
    ("sparkle", "stars"),
    ("star", "star"),
    ("stop", "stop"),
    ("sun", "light_mode"),
    ("table", "table_chart"),
    ("tag", "sell"),
    ("target", "target"),
    ("terminal", "terminal"),
    ("timer", "timer"),
    ("trash", "delete"),
    ("undo", "undo"),
    ("unlink", "link_off"),
    ("unlock", "lock_open"),
    ("upload", "upload"),
    ("user", "person"),
    ("users", "group"),
    ("visor", "vrpano"),
    ("volume", "volume_up"),
    ("wallet", "account_balance_wallet"),
    ("warning", "warning"),
    ("wave", "waves"),
    ("wifi", "wifi"),
    ("zoom-in", "zoom_in"),
    ("zoom-out", "zoom_out"),
];

/// The Material name for `name`: the mapped name when `name` is a known alias
/// (`"trash"` → `"delete"`), otherwise `name` unchanged.
pub(crate) fn resolve(name: &str) -> &str {
    match ALIASES.binary_search_by(|(a, _)| (*a).cmp(name)) {
        Ok(i) => ALIASES[i].1,
        Err(_) => name,
    }
}

/// The familiar (Feather-style) alias of a Material name, when there is one:
/// `"arrow_back"` -> `"arrow-left"`.
#[doc(hidden)]
pub fn alias_of(material: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|(_, m)| *m == material)
        .map(|(a, _)| *a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_and_resolves() {
        assert!(ALIASES.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(resolve("trash"), "delete");
        assert_eq!(resolve("home"), "home");
        assert_eq!(resolve("not-an-alias"), "not-an-alias");
    }

    #[cfg(feature = "material")]
    #[test]
    fn every_alias_target_exists_in_the_catalogue() {
        for (a, m) in ALIASES {
            assert!(
                crate::material::find(crate::material::outlined::ALL, m).is_some(),
                "{a} -> {m}"
            );
            assert!(
                crate::material::find(crate::material::outlined::ALL, &format!("{m}-fill"))
                    .is_some(),
                "{a} -> {m}-fill"
            );
        }
    }
}
