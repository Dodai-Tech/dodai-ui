//! Lists every theme the enabled design features provide and the primary button each resolves
//! to, then the same theme through the toolkit features that are on.
//!
//! ```sh
//! cargo run -p dodai-ui --example themes --features all-designs
//! cargo run -p dodai-ui --example themes --features "cyberpunk,ratatui"
//! ```

use dodai_ui::core::{switcher, ButtonVariant, ControlState};

fn main() {
    let mut sw = switcher();
    println!("{} themes", sw.themes().len());
    for i in 0..sw.themes().len() {
        sw.select_index(i);
        let t = sw.current();
        let b = t.button(ButtonVariant::Primary, ControlState::Idle);
        println!(
            "{:<34} {:<14} primary: bg {:?} fg {:?}",
            t.meta.name, t.meta.family, b.background, b.foreground
        );
    }

    // Toolkit modules exist only when their feature is on.
    #[cfg(feature = "ratatui")]
    {
        let _ = dodai_ui::ratatui::RatatuiTheme::new(sw.current().clone());
        println!("ratatui adapter ready");
    }
    #[cfg(feature = "iced")]
    {
        let _ = dodai_ui::iced::IcedTheme::new(sw.current().clone());
        println!("iced adapter ready");
    }
}
