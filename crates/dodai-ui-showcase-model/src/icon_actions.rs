//! The icon detail's actions: copy the Rust usage or the SVG, save the SVG to `~/Downloads`.
//!
//! Saving happens here (it is plain file I/O); copying is an [`Effect`] the toolkit performs
//! with its own clipboard.

use dodai_ui_core::Theme;

use crate::icons::IconRef;
use crate::state::Values;

/// Key prefix of the three buttons.
pub const PREFIX: &str = "icon:";
/// Value key of the line under the buttons: `<icon name>|<message>`.
pub const MSG: &str = "icon-msg";

/// Something only the toolkit can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Put this text on the clipboard.
    Copy(String),
}

/// The button keys.
pub fn copy_svg() -> String {
    format!("{PREFIX}copy-svg")
}
/// See [`copy_svg`].
pub fn copy_rust() -> String {
    format!("{PREFIX}copy-rust")
}
/// See [`copy_svg`].
pub fn save_svg() -> String {
    format!("{PREFIX}save-svg")
}

/// The message to show under the buttons for `icon` (nothing once another icon is picked).
pub fn message(values: &Values, icon: IconRef) -> Option<&str> {
    let Some(crate::state::Value::Text(line)) = values.0.get(MSG) else {
        return None;
    };
    let (name, text) = line.split_once('|')?;
    (name == icon.name()).then_some(text)
}

/// Lay the one-line SVG out for reading: attributes and path data on their own lines, wrapped at
/// spaces (still valid SVG, and the same drawing).
pub fn pretty_svg(svg: &str) -> String {
    let flat = svg
        .replace(" xmlns=", "\n  xmlns=")
        .replace(" width=", "\n  width=")
        .replace(" height=", "\n  height=")
        .replace(" viewBox=", "\n  viewBox=")
        .replace("><path", ">\n  <path")
        .replace(" d=\"", "\n    d=\"")
        .replace("/></svg>", "/>\n</svg>");
    let mut out = String::new();
    for line in flat.lines() {
        let mut width = 0;
        for (i, word) in line.split(' ').enumerate() {
            if i > 0 {
                if width + word.len() > 96 {
                    out.push_str("\n      ");
                    width = 6;
                } else {
                    out.push(' ');
                    width += 1;
                }
            }
            out.push_str(word);
            width += word.len();
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// A file name for `name`: letters, digits, `-` and `_` only.
fn file_name(name: &str) -> String {
    name.replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "_",
    )
}

/// Where downloads go.
fn downloads() -> std::path::PathBuf {
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join("Downloads")
}

/// Run the action `what` (`copy-svg`, `copy-rust`, `save-svg`) on the picked icon; the message
/// goes to `values`. `dir` overrides the download folder (tests).
pub fn run(
    values: &mut crate::state::Values,
    theme: &Theme,
    what: &str,
    dir: Option<&std::path::Path>,
) -> Option<Effect> {
    let icon = IconRef::from_index(values.index_of("icon-sel", 0).wrapping_sub(1))?;
    let filled = values.flag(&crate::Bind::new("icon-filled", false));
    let svg = pretty_svg(&icon.svg(theme, filled).replace(">\n<", "><"));
    let (effect, text) = match what {
        "copy-svg" => (Some(Effect::Copy(svg)), "SVG copied".to_string()),
        "copy-rust" => {
            let usage = icon.usage();
            (
                Some(Effect::Copy(usage.clone())),
                format!("Copied `{usage}`"),
            )
        }
        "save-svg" => {
            let dir = dir
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(downloads);
            let path = dir.join(format!("{}.svg", file_name(icon.name())));
            let text =
                match std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, &svg)) {
                    Ok(()) => format!("Saved {}", path.display()),
                    Err(e) => format!("Could not save: {e}"),
                };
            (None, text)
        }
        _ => return None,
    };
    values.set(
        MSG.to_string(),
        crate::state::Value::Text(format!("{}|{text}", icon.name())),
    );
    effect
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Value, Values};

    fn picked() -> (Values, Theme) {
        let mut v = Values::default();
        v.set("icon-sel".to_string(), Value::Index(1));
        (v, Theme::default())
    }

    #[test]
    fn copy_svg_hands_the_svg_to_the_toolkit() {
        let (mut v, t) = picked();
        let Some(Effect::Copy(svg)) = run(&mut v, &t, "copy-svg", None) else {
            panic!("no copy effect");
        };
        assert!(svg.starts_with("<svg"), "{svg}");
        assert_eq!(
            message(&v, IconRef::from_index(0).unwrap()),
            Some("SVG copied")
        );
    }

    #[test]
    fn copy_rust_hands_over_the_usage() {
        let (mut v, t) = picked();
        let icon = IconRef::from_index(0).unwrap();
        assert_eq!(
            run(&mut v, &t, "copy-rust", None),
            Some(Effect::Copy(icon.usage()))
        );
    }

    #[test]
    fn save_writes_the_file_and_says_where() {
        let (mut v, t) = picked();
        let dir = std::env::temp_dir().join("dodai-icon-save-test");
        assert_eq!(run(&mut v, &t, "save-svg", Some(&dir)), None);
        let icon = IconRef::from_index(0).unwrap();
        let msg = message(&v, icon).unwrap();
        assert!(msg.starts_with("Saved "), "{msg}");
        let file = dir.join(format!("{}.svg", file_name(icon.name())));
        assert!(std::fs::read_to_string(file).unwrap().starts_with("<svg"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_message_belongs_to_its_icon() {
        let (mut v, t) = picked();
        run(&mut v, &t, "copy-svg", None);
        assert_eq!(message(&v, IconRef::from_index(1).unwrap()), None);
    }

    #[test]
    fn nothing_picked_does_nothing() {
        let mut v = Values::default();
        assert_eq!(run(&mut v, &Theme::default(), "copy-svg", None), None);
    }
}
