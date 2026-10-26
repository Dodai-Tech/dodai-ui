//! Entry animation scope of a section, and replaying one section.

use super::*;
use dodai_gpui::EntrySet;

use catalog::caps;

impl Showcase {
    /// Entry set applied to a section's components: the app's own motion, or none for a section
    /// whose components have no entry animation.
    pub(crate) fn scope_of(&self, title: &'static str, cx: &gpui::App) -> EntrySet {
        let cp = caps(title);
        if !(cp.entry || cp.fill || cp.stagger) {
            return EntrySet::NONE;
        }
        dodai_gpui::motion::current(cx).entry
    }

    /// Replay one section only (re-keys just its subtree).
    pub(crate) fn bump(&mut self, title: &'static str) {
        *self.sec_replay.entry(title).or_insert(0) += 1;
    }
}
