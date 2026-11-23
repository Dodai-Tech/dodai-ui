//! Every Dodai UI component in Slint, with a theme switcher (button per
//! theme + `t`/`T`) and glow +/- controls.
use dodai_slint::slint::{self, ComponentHandle, ModelRc, SharedString, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::slint! {
    import { Theme, LButton, LPanel, LTag, LTabs, LToggle, LCheckbox, LRadio, LProgress, LSpinner,
             LSlider, LInput, LListItem, LDivider, LTooltip, LText, LScanlines, LBackdrop } from "ui/dodai.slint";
    export { Theme }

    export component Showcase inherits Window {
        title: "Dodai UI · Slint";
        // The Dodai UI logo (assets/icons/dodai.svg); paths are from the crate's manifest.
        icon: @image-url("../../assets/icons/dodai-128.png");
        preferred-width: 980px;
        preferred-height: 900px;
        background: Theme.background;
        in property <[string]> theme-names;
        in property <string> current;
        in property <string> glow-label;
        in-out property <int> tab;
        in-out property <int> radio: 0;
        in-out property <int> list-sel: 0;
        in-out property <float> slider: 0.4;
        callback pick-theme(int);
        callback cycle(bool);
        callback glow(float);

        LBackdrop {}

        FocusScope {
            key-pressed(e) => {
                if (e.text == "t") { root.cycle(true); return accept; }
                if (e.text == "T") { root.cycle(false); return accept; }
                reject
            }
            VerticalLayout {
                padding: Theme.sp-lg;
                spacing: Theme.sp-md;
                HorizontalLayout {
                    spacing: Theme.sp-sm;
                    alignment: start;
                    Image {
                        source: @image-url("../../assets/icons/dodai-128.png");
                        width: Theme.size-xl * 1.6;
                        height: Theme.size-xl * 1.6;
                    }
                    LText { kind: 0; content: "Dodai UI · " + root.current; font-size: Theme.size-xl; vertical-alignment: center; }
                }
                HorizontalLayout {
                    spacing: Theme.sp-sm;
                    for n[i] in root.theme-names: LButton { text: n; variant: n == root.current ? 0 : 2; clicked => { root.pick-theme(i); } }
                    LText { content: "glow " + root.glow-label; kind: 2; vertical-alignment: center; }
                    LButton { text: "−"; variant: 3; clicked => { root.glow(-0.25); } }
                    LButton { text: "+"; variant: 3; clicked => { root.glow(0.25); } }
                }
                HorizontalLayout {
                    spacing: Theme.sp-md;
                    LPanel {
                        title: "Buttons";
                        HorizontalLayout { spacing: Theme.sp-sm;
                            LButton { text: "Primary"; variant: 0; }
                            LButton { text: "Secondary"; variant: 1; }
                            LButton { text: "Outline"; variant: 2; }
                        }
                        HorizontalLayout { spacing: Theme.sp-sm;
                            LButton { text: "Outline 2"; variant: 3; }
                            LButton { text: "Danger"; variant: 4; }
                            LButton { text: "Ghost"; variant: 5; }
                            LButton { text: "Disabled"; enabled: false; }
                        }
                    }
                    LPanel {
                        title: "Tags";
                        focused: true;
                        HorizontalLayout { spacing: Theme.sp-sm;
                            LTag { text: "info"; tone: 0; }
                            LTag { text: "ok"; tone: 1; solid: true; }
                            LTag { text: "warn"; tone: 2; }
                            LTag { text: "fail"; tone: 3; solid: true; }
                        }
                        LTabs { tabs: ["One", "Two", "Three"]; current <=> root.tab; }
                    }
                }
                HorizontalLayout {
                    spacing: Theme.sp-md;
                    LPanel {
                        title: "Controls";
                        HorizontalLayout { spacing: Theme.sp-md;
                            LToggle { checked: true; }
                            LToggle { }
                            LToggle { enabled: false; }
                        }
                        LCheckbox { text: "Checked"; checked: true; }
                        LCheckbox { text: "Unchecked"; }
                        LRadio { text: "Radio A"; selected: root.radio == 0; clicked => { root.radio = 0; } }
                        LRadio { text: "Radio B"; selected: root.radio == 1; clicked => { root.radio = 1; } }
                    }
                    LPanel {
                        title: "Progress · slider · input";
                        LProgress { value: root.slider; }
                        LProgress { value: 0.7; tone: 1; }
                        LProgress { value: 0.5; tone: 2; }
                        LProgress { value: 0.3; tone: 3; }
                        HorizontalLayout { spacing: 14px; LSpinner { } LSpinner { size: 36px; } }
                        LSlider { value <=> root.slider; }
                        LInput { placeholder: "Type here…"; }
                    }
                    LPanel {
                        title: "List";
                        for n[i] in ["Alpha", "Beta", "Gamma"]: LListItem { text: n; selected: root.list-sel == i; clicked => { root.list-sel = i; } }
                        LDivider { }
                        LText { kind: 3; content: "mono text"; }
                        LText { kind: 2; content: "muted text"; }
                        LText { kind: 1; content: "label text"; }
                    }
                }
                tip-area := Rectangle {
                    height: 40px;
                    LText { content: "Hover for tooltip"; kind: 2; }
                    ta := TouchArea { }
                    LTooltip { x: 160px; y: 0px; text: "Tooltip"; shown: ta.has-hover; }
                }
            }
            LScanlines { }
        }
    }
}

fn main() {
    let ui = Showcase::new().unwrap();
    let sw = Rc::new(RefCell::new(dodai_core::switcher()));
    let glow = Rc::new(RefCell::new(1.0f32));
    let names: Vec<SharedString> = sw
        .borrow()
        .themes()
        .iter()
        .map(|t| t.name().into())
        .collect();
    ui.set_theme_names(ModelRc::new(VecModel::from(names)));

    let apply = {
        let (sw, glow) = (sw.clone(), glow.clone());
        let weak = ui.as_weak();
        Rc::new(move || {
            let ui = weak.unwrap();
            let t = sw
                .borrow()
                .current()
                .clone()
                .with_glow_intensity(*glow.borrow());
            dodai_slint::install_theme!(Theme, &ui, &t);
            ui.set_current(t.name().into());
            ui.set_glow_label(format!("{:.2}", *glow.borrow()).into());
        })
    };
    apply();
    {
        let (sw, a) = (sw.clone(), apply.clone());
        ui.on_pick_theme(move |i| {
            sw.borrow_mut().select_index(i as usize);
            a();
        });
    }
    {
        let (sw, a) = (sw.clone(), apply.clone());
        ui.on_cycle(move |fwd| {
            if fwd {
                sw.borrow_mut().cycle_next();
            } else {
                sw.borrow_mut().cycle_prev();
            }
            a();
        });
    }
    {
        let a = apply.clone();
        ui.on_glow(move |d| {
            let mut g = glow.borrow_mut();
            *g = (*g + d).clamp(0.0, 3.0);
            drop(g);
            a();
        });
    }
    if std::env::var_os("SLINT_SHOWCASE_SMOKE").is_some() {
        ui.show().unwrap();
        slint::Timer::single_shot(std::time::Duration::from_millis(800), || {
            let _ = slint::quit_event_loop();
        });
    }
    ui.run().unwrap();
}
