//! Pure selection / paging math shared by components (no gpui context needed).

pub use dodai_core::paging::{page_window, step_index, PageItem};

pub use dodai_core::StepState as StepStatus;

/// Status of step `i` when `current` is the active step.
pub fn step_status(i: usize, current: usize) -> StepStatus {
    StepStatus::of(i, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        assert_eq!(step_status(0, 1), StepStatus::Done);
        assert_eq!(step_status(1, 1), StepStatus::Current);
        assert_eq!(step_status(2, 1), StepStatus::Todo);
    }
}
