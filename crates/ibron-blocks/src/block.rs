use serde::{Deserialize, Serialize};
use wezterm_term::StableRowIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    AwaitingCommand,
    Running,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub prompt_row: StableRowIndex,
    /// Column on `prompt_row` where user input starts (right after the
    /// shell prompt). Populated by OSC 133;B.
    pub input_column: Option<usize>,
    pub output_start: Option<StableRowIndex>,
    pub output_end: Option<StableRowIndex>,
    pub exit_status: Option<i32>,
    pub state: BlockState,
}

impl Block {
    pub fn new(id: BlockId, prompt_row: StableRowIndex) -> Self {
        Self {
            id,
            prompt_row,
            input_column: None,
            output_start: None,
            output_end: None,
            exit_status: None,
            state: BlockState::AwaitingCommand,
        }
    }

    /// (inclusive start, exclusive end) stable-row span covered by this block.
    /// If output hasn't started yet, the span is just the prompt row.
    pub fn row_span(&self) -> (StableRowIndex, StableRowIndex) {
        let end = self
            .output_end
            .map(|r| r + 1)
            .or_else(|| self.output_start.map(|r| r + 1))
            .unwrap_or(self.prompt_row + 1);
        (self.prompt_row, end)
    }

    pub fn contains_row(&self, row: StableRowIndex) -> bool {
        let (s, e) = self.row_span();
        s <= row && row < e
    }
}
