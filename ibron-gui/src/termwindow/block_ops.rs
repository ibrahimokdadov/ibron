//! Phase 5 command-block operations.
//!
//! Operations act on a "focused" block within the active pane. When no
//! focus has been explicitly set, operations target the most recent block.
//! BlockFocusPrev / BlockFocusNext cycle through blocks in creation order.

use crate::TermWindow;
use config::keyassignment::ClipboardCopyDestination;
use ibron_blocks::Block;
use mux::pane::Pane;
use std::sync::Arc;
use wezterm_term::StableRowIndex;
use window::WindowOps;

impl TermWindow {
    fn focused_block_snapshot(&self, pane_id: mux::pane::PaneId) -> Option<Block> {
        let manager = self.blocks.get(&pane_id)?;
        let blocks = manager.all();
        if blocks.is_empty() {
            return None;
        }
        if let Some(id) = self.focused_block.get(&pane_id) {
            if let Some(b) = blocks.iter().find(|b| b.id == *id) {
                return Some(b.clone());
            }
        }
        blocks.last().cloned()
    }

    pub(crate) fn block_focus_step(&mut self, pane: &Arc<dyn Pane>, delta: isize) {
        let pane_id = pane.pane_id();
        let Some(manager) = self.blocks.get(&pane_id) else {
            return;
        };
        let blocks = manager.all();
        if blocks.is_empty() {
            return;
        }
        let current_idx = self
            .focused_block
            .get(&pane_id)
            .and_then(|id| blocks.iter().position(|b| b.id == *id))
            .unwrap_or(blocks.len() - 1);
        let len = blocks.len() as isize;
        let next = ((current_idx as isize + delta).rem_euclid(len)) as usize;
        let new_id = blocks[next].id;
        self.focused_block.insert(pane_id, new_id);
        if let Some(w) = self.window.as_ref() {
            w.invalidate();
        }
    }

    pub(crate) fn block_copy_command(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockCopyCommand: no block to copy");
            return;
        };
        let end = block.output_start.unwrap_or(block.prompt_row + 1);
        let text = extract_text(pane, block.prompt_row..end);
        if text.is_empty() {
            log::warn!("BlockCopyCommand: empty command text");
            return;
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, text);
    }

    pub(crate) fn block_copy_output(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockCopyOutput: no block to copy");
            return;
        };
        let Some(start) = block.output_start else {
            log::warn!("BlockCopyOutput: block has no output yet");
            return;
        };
        let end_inclusive = block.output_end.unwrap_or(start);
        let text = extract_text(pane, start..end_inclusive + 1);
        if text.is_empty() {
            log::warn!("BlockCopyOutput: empty output text");
            return;
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, text);
    }
}

fn extract_text(pane: &Arc<dyn Pane>, range: std::ops::Range<StableRowIndex>) -> String {
    if range.end <= range.start {
        return String::new();
    }
    let (_first, lines) = pane.get_lines(range);
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(line.as_str().trim_end());
    }
    out
}
