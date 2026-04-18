//! ibron-blocks: command-block layer for the ibron terminal.
//!
//! Consumes `wezterm_term::CommandBlockEvent` (derived from OSC 133 A/C/D)
//! and maintains a per-pane `BlockManager` — a list of `Block`s keyed by
//! stable row index. The renderer (Phase 4) and operations (Phase 5)
//! query this model.

pub mod block;
pub mod manager;

pub use block::{Block, BlockId, BlockState};
pub use manager::BlockManager;

#[cfg(test)]
mod tests {
    use super::*;
    use wezterm_term::CommandBlockEvent as E;

    #[test]
    fn block_new_is_awaiting_command() {
        let b = Block::new(BlockId(1), 3);
        assert_eq!(b.id, BlockId(1));
        assert_eq!(b.prompt_row, 3);
        assert!(b.output_start.is_none());
        assert!(b.exit_status.is_none());
        assert_eq!(b.state, BlockState::AwaitingCommand);
    }

    #[test]
    fn manager_tracks_one_command_end_to_end() {
        let mut m = BlockManager::new();
        m.on_event(E::PromptStart, 10);
        m.on_event(E::OutputStart, 11);
        m.on_event(E::CommandEnd { status: 0 }, 14);

        assert_eq!(m.len(), 1);
        let b = m.latest().unwrap();
        assert_eq!(b.prompt_row, 10);
        assert_eq!(b.output_start, Some(11));
        assert_eq!(b.output_end, Some(13));
        assert_eq!(b.exit_status, Some(0));
        assert_eq!(b.state, BlockState::Completed);
    }

    #[test]
    fn manager_block_at_returns_covering_block() {
        let mut m = BlockManager::new();
        m.on_event(E::PromptStart, 10);
        m.on_event(E::OutputStart, 11);
        m.on_event(E::CommandEnd { status: 1 }, 14);
        m.on_event(E::PromptStart, 15);
        m.on_event(E::OutputStart, 16);
        m.on_event(E::CommandEnd { status: 0 }, 18);

        assert_eq!(m.block_at(10).map(|b| b.id), Some(BlockId(1)));
        assert_eq!(m.block_at(13).map(|b| b.id), Some(BlockId(1)));
        assert_eq!(m.block_at(14).map(|b| b.id), None);
        assert_eq!(m.block_at(15).map(|b| b.id), Some(BlockId(2)));
        assert_eq!(m.block_at(17).map(|b| b.id), Some(BlockId(2)));
        assert_eq!(m.block_at(99).map(|b| b.id), None);
    }

    #[test]
    fn manager_tolerates_prompt_while_running() {
        let mut m = BlockManager::new();
        m.on_event(E::PromptStart, 1);
        m.on_event(E::OutputStart, 2);
        m.on_event(E::PromptStart, 5);

        assert_eq!(m.len(), 2);
        let first = &m.all()[0];
        assert_eq!(first.state, BlockState::Completed);
        assert!(first.exit_status.is_none());
    }

    #[test]
    fn manager_records_input_column_once_per_block() {
        let mut m = BlockManager::new();
        m.on_event(E::PromptStart, 0);
        m.on_event(E::InputStart { column: 2 }, 0);
        // A second InputStart (e.g. continuation marker) should not
        // clobber the first, which is what we want for command extraction.
        m.on_event(E::InputStart { column: 99 }, 0);
        assert_eq!(m.latest().unwrap().input_column, Some(2));
    }

    #[test]
    fn manager_iter_range_filters_visible_window() {
        let mut m = BlockManager::new();
        m.on_event(E::PromptStart, 10);
        m.on_event(E::OutputStart, 11);
        m.on_event(E::CommandEnd { status: 0 }, 13);
        m.on_event(E::PromptStart, 20);
        m.on_event(E::OutputStart, 21);
        m.on_event(E::CommandEnd { status: 0 }, 22);

        let ids: Vec<_> = m.iter_range(12, 21).map(|b| b.id).collect();
        assert_eq!(ids, vec![BlockId(1), BlockId(2)]);
        let ids: Vec<_> = m.iter_range(15, 19).map(|b| b.id).collect();
        assert!(ids.is_empty());
    }
}
