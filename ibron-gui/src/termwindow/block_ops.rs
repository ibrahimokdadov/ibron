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
        let text = extract_command_text(pane, &block);
        if text.is_empty() {
            log::warn!("BlockCopyCommand: empty command text");
            return;
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, text);
    }

    pub(crate) fn block_rerun(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockRerun: no block to rerun");
            return;
        };
        let text = extract_command_text(pane, &block);
        if text.is_empty() {
            log::warn!("BlockRerun: empty command text");
            return;
        }
        if let Err(err) = pane.send_paste(&text) {
            log::error!("BlockRerun: send_paste failed: {err:#}");
            return;
        }
        if let Err(err) = pane.send_paste("\r") {
            log::error!("BlockRerun: send_paste(CR) failed: {err:#}");
        }
    }

    pub(crate) fn block_copy_output(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockCopyOutput: no block to copy");
            return;
        };
        let text = extract_output_text(pane, &block);
        if text.is_empty() {
            log::warn!("BlockCopyOutput: no output to copy");
            return;
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, text);
    }

    pub(crate) fn block_toggle_fold(&mut self, pane: &Arc<dyn Pane>) {
        let pane_id = pane.pane_id();
        let Some(block) = self.focused_block_snapshot(pane_id) else {
            log::warn!("BlockFold: no block in focus");
            return;
        };
        let set = self.folded_blocks.entry(pane_id).or_default();
        if !set.insert(block.id) {
            set.remove(&block.id);
        }
        if let Some(w) = self.window.as_ref() {
            w.invalidate();
        }
    }

    pub(crate) fn block_toggle_bookmark(&mut self, pane: &Arc<dyn Pane>) {
        let pane_id = pane.pane_id();
        let Some(block) = self.focused_block_snapshot(pane_id) else {
            log::warn!("BlockBookmark: no block in focus");
            return;
        };
        let set = self.bookmarked_blocks.entry(pane_id).or_default();
        if !set.insert(block.id) {
            set.remove(&block.id);
        }
        if let Some(w) = self.window.as_ref() {
            w.invalidate();
        }
    }

    pub(crate) fn block_share(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockShare: no block in focus");
            return;
        };
        let md = format_block_as_markdown(pane, &block, None);
        if md.is_empty() {
            log::warn!("BlockShare: empty block");
            return;
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, md);
    }

    pub(crate) fn block_ask_ai(&self, pane: &Arc<dyn Pane>) {
        let Some(block) = self.focused_block_snapshot(pane.pane_id()) else {
            log::warn!("BlockAskAI: no block in focus");
            return;
        };
        let prefix = "Please explain or debug the following terminal command and its output:\n\n";
        let md = format_block_as_markdown(pane, &block, Some(prefix));
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, md);
    }

    pub(crate) fn block_search_open(&self, pane: &Arc<dyn Pane>) {
        let pane_id = pane.pane_id();
        let Some(manager) = self.blocks.get(&pane_id) else {
            log::warn!("BlockSearchOpen: no blocks in this pane");
            return;
        };
        let mut out = String::from("# Command blocks\n\n");
        for block in manager.all() {
            let cmd = extract_command_text(pane, block);
            let status = match block.exit_status {
                Some(0) => "ok".to_string(),
                Some(s) => format!("exit {s}"),
                None => "running".to_string(),
            };
            out.push_str(&format!(
                "- [{}] `{}`\n",
                status,
                cmd.lines().next().unwrap_or("")
            ));
        }
        self.copy_to_clipboard(ClipboardCopyDestination::Clipboard, out);
    }
}

fn extract_output_text(pane: &Arc<dyn Pane>, block: &Block) -> String {
    let Some(start) = block.output_start else {
        return String::new();
    };
    let end_inclusive = block.output_end.unwrap_or(start);
    extract_text(pane, start..end_inclusive + 1)
}

fn format_block_as_markdown(pane: &Arc<dyn Pane>, block: &Block, prefix: Option<&str>) -> String {
    let mut out = String::new();
    if let Some(p) = prefix {
        out.push_str(p);
    }
    let cmd = extract_command_text(pane, block);
    if !cmd.is_empty() {
        out.push_str("**Command:**\n\n```\n");
        out.push_str(&cmd);
        if !cmd.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n\n");
    }
    let output = extract_output_text(pane, block);
    if !output.is_empty() {
        let status = match block.exit_status {
            Some(0) => "Output (exit 0):".to_string(),
            Some(s) => format!("Output (exit {s}):"),
            None => "Output (still running):".to_string(),
        };
        out.push_str(&format!("**{status}**\n\n```\n"));
        out.push_str(&output);
        if !output.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
    }
    out
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

/// Pulls just the command the user typed from `block`, trimming the shell
/// prompt when OSC 133;B gave us an input column. Falls back to the full
/// prompt row(s) if no InputStart was seen.
fn extract_command_text(pane: &Arc<dyn Pane>, block: &Block) -> String {
    let end = block.output_start.unwrap_or(block.prompt_row + 1);
    let text = extract_text(pane, block.prompt_row..end);
    match block.input_column {
        Some(col) if col > 0 => {
            let mut lines = text.splitn(2, '\n');
            let first = lines.next().unwrap_or("");
            let rest = lines.next();
            let trimmed_first = trim_first_columns(first, col);
            match rest {
                Some(r) => {
                    let mut out = String::with_capacity(trimmed_first.len() + 1 + r.len());
                    out.push_str(&trimmed_first);
                    out.push('\n');
                    out.push_str(r);
                    out
                }
                None => trimmed_first.into_owned(),
            }
        }
        _ => text,
    }
}

fn trim_first_columns(s: &str, cols: usize) -> std::borrow::Cow<'_, str> {
    match s.char_indices().nth(cols) {
        Some((byte_offset, _)) => std::borrow::Cow::Borrowed(&s[byte_offset..]),
        None => std::borrow::Cow::Borrowed(""),
    }
}
