use crate::block::{Block, BlockId, BlockState};
use wezterm_term::{CommandBlockEvent, StableRowIndex};

#[derive(Debug, Default)]
pub struct BlockManager {
    blocks: Vec<Block>,
    next_id: u64,
}

impl BlockManager {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn all(&self) -> &[Block] {
        &self.blocks
    }

    pub fn latest(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn on_event(&mut self, event: CommandBlockEvent, row: StableRowIndex) {
        match event {
            CommandBlockEvent::PromptStart => {
                if let Some(last) = self.blocks.last_mut() {
                    if last.state == BlockState::Running {
                        last.state = BlockState::Completed;
                    }
                }
                let id = BlockId(self.next_id);
                self.next_id += 1;
                self.blocks.push(Block::new(id, row));
            }
            CommandBlockEvent::InputStart { column } => {
                if let Some(last) = self.blocks.last_mut() {
                    if last.input_column.is_none() {
                        last.input_column = Some(column);
                    }
                }
            }
            CommandBlockEvent::OutputStart => {
                if let Some(last) = self.blocks.last_mut() {
                    last.output_start = Some(row);
                    last.state = BlockState::Running;
                }
            }
            CommandBlockEvent::CommandEnd { status } => {
                if let Some(last) = self.blocks.last_mut() {
                    last.exit_status = Some(status);
                    let end = if row > last.prompt_row {
                        row - 1
                    } else {
                        last.prompt_row
                    };
                    last.output_end = Some(end);
                    last.state = BlockState::Completed;
                }
            }
        }
    }

    /// Binary-searches for the block whose row span covers `row`.
    pub fn block_at(&self, row: StableRowIndex) -> Option<&Block> {
        let idx = self.blocks.partition_point(|b| b.prompt_row <= row);
        if idx == 0 {
            return None;
        }
        let candidate = &self.blocks[idx - 1];
        if candidate.contains_row(row) {
            Some(candidate)
        } else {
            None
        }
    }

    /// Yields every block whose span overlaps `[start, end)`.
    pub fn iter_range(
        &self,
        start: StableRowIndex,
        end: StableRowIndex,
    ) -> impl Iterator<Item = &Block> {
        self.blocks.iter().filter(move |b| {
            let (s, e) = b.row_span();
            s < end && e > start
        })
    }
}
