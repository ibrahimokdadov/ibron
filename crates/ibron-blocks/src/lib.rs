//! ibron-blocks: command-block layer for the ibron terminal.
//!
//! Empty in phase 1 — exists to reserve the workspace slot and verify the
//! build graph. Real code arrives in phase 3 on top of wezterm's existing
//! semantic zone infrastructure (see docs/superpowers/plans/notes/
//! phase1-scaffold-survey.md for context).

#[cfg(test)]
mod tests {
    #[test]
    fn crate_exists() {
        assert_eq!(2 + 2, 4);
    }
}
