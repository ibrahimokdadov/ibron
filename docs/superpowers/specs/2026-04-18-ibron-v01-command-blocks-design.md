# ibron v0.1 — Command Blocks

**Status:** Design approved 2026-04-18
**Author:** ibrahimokdadov
**Project:** ibron (soft-fork of [wezterm/wezterm](https://github.com/wezterm/wezterm), MIT)
**Repo:** `github.com/ibrahimokdadov/ibron`

## Context

ibron is a full fork of wezterm aimed at replacing Warp as the user's daily terminal. Warp's problems for this user: flaky PowerShell support on Windows, subscription cost creeping up. Wezterm is the right base — GPU-accelerated, cross-platform, rock-solid PowerShell support, Lua config, MIT-licensed.

v0.1 ships Warp's single most recognizable feature: **command blocks**. Every command you run and its output become a first-class, navigable UI object.

This spec covers v0.1 only. Downstream features (AI error help, autocomplete, session restore, sidebar) get their own specs.

## Goals (v0.1)

- Every command executed in a shell with ibron's integration script becomes a visually distinct "block" on screen.
- Users can operate on blocks with keyboard and mouse: copy command, copy output, re-run, share, trigger-AI (hook only — AI ships in v0.2), fold, bookmark, search.
- Zero regression vs upstream wezterm when shell integration is not installed — terminal behaves exactly like wezterm.
- Rebrand complete: `ibron` / `ibron-gui` binaries, `~/.ibron.lua` config, "ibron" window class, upstream attribution preserved.

## Non-goals (v0.1)

- AI features (v0.2).
- Ghost-text autocomplete (v0.3).
- Session/tab restore across restarts (v0.4).
- Sidebar UI, CPU/mem indicators, folder tree (v0.5).
- Breaking existing wezterm Lua config — all existing `.wezterm.lua` configs must continue to work when placed at `~/.ibron.lua`.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  User's shell (PowerShell / bash / zsh)         │
│  ibron-shell-integration.{ps1,sh,zsh} sourced   │
│  → emits OSC 133 A/B/C/D markers                │
└────────────────┬────────────────────────────────┘
                 │ stdout + escape sequences
                 ▼
┌─────────────────────────────────────────────────┐
│  wezterm terminal parser  (UNCHANGED)           │
│  crates/termwiz, crates/wezterm-term            │
└────────────────┬────────────────────────────────┘
                 │ parsed cells + OSC escape events
                 ▼
┌─────────────────────────────────────────────────┐
│  NEW: Block Manager                             │
│   crates/ibron-blocks/                          │
│   • listens for OSC 133 events from term parser │
│   • builds Block{cmd, output_range, exit, time} │
│   • owns per-pane block history (VecDeque)      │
│   • exposes block lookup by screen coords       │
└────────────────┬────────────────────────────────┘
                 │ block boundary metadata
                 ▼
┌─────────────────────────────────────────────────┐
│  wezterm renderer (MODIFIED)                    │
│   crates/wezterm-gui/src/termwindow/            │
│   • consults BlockManager during draw           │
│   • draws block chrome (borders, hover, status) │
│   • routes mouse/key events to block ops        │
└─────────────────────────────────────────────────┘
                 ▲
                 │ user input
┌────────────────┴────────────────────────────────┐
│  Block Operations (independent modules)         │
│   crates/ibron-blocks/src/ops/                  │
│   copy_cmd, copy_output, rerun, share,          │
│   ask_ai (stub in v0.1), fold, bookmark, search │
└─────────────────────────────────────────────────┘
```

### Why a new crate, not inline changes

`crates/ibron-blocks` lives alongside upstream crates but never modifies them except at well-defined integration points (one event-listener registration, one renderer call, one key-handler call). This keeps the upstream merge surface small — when `git merge upstream/main` runs, conflicts concentrate in ~5 known files, not scattered across the codebase.

## Components

### 1. Shell integration scripts

Files: `assets/shell-integration/ibron.ps1`, `ibron.sh`, `ibron.zsh`, `ibron.fish`.

Each script hooks into the shell's prompt machinery to emit OSC 133 escape sequences:

| Marker | When | Meaning |
|--------|------|---------|
| `OSC 133 ; A ST` | Just before prompt | "Prompt starts here" |
| `OSC 133 ; B ST` | Right after prompt, before user types | "Command input starts here" |
| `OSC 133 ; C ST` | After user hits enter | "Command output starts here" |
| `OSC 133 ; D ; <exit> ST` | After command finishes | "Command ended with exit code" |

Standard: same sequences used by iTerm2, Warp, Ghostty, WezTerm's own experimental support. No ibron-specific extensions in v0.1.

PowerShell implementation sketch:

```powershell
$global:__ibron_LastExitCode = 0
function prompt {
    $exit = $LASTEXITCODE
    if ($null -ne $exit) { $global:__ibron_LastExitCode = $exit }
    "$([char]27)]133;D;$($global:__ibron_LastExitCode)$([char]27)\" +
    "$([char]27)]133;A$([char]27)\" +
    "PS $($executionContext.SessionState.Path.CurrentLocation)> " +
    "$([char]27)]133;B$([char]27)\"
}
# C marker emitted via PSReadLine hook on AcceptLine
```

Installation: `ibron install-shell-integration` subcommand writes the appropriate script to the user's shell config (or prints instructions if auto-detection fails).

### 2. Block data model (`crates/ibron-blocks/src/block.rs`)

```rust
pub struct Block {
    pub id: BlockId,              // u64, monotonic per pane
    pub command: String,           // captured between B and C markers
    pub output_start: StableRowIndex,
    pub output_end: Option<StableRowIndex>,  // None while running
    pub exit_code: Option<i32>,
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
    pub cwd: Option<PathBuf>,      // from OSC 7 if available
    pub folded: bool,
    pub bookmarked: bool,
}

pub enum BlockState { Running, Success, Failure }
```

`StableRowIndex` is wezterm's existing scrollback-stable row reference — survives scrollback trimming.

### 3. OSC 133 parser (`crates/ibron-blocks/src/parser.rs`)

Hooks into wezterm's existing OSC dispatch (`crates/wezterm-term/src/terminalstate/mod.rs`) via a new callback trait `BlockEventSink`. Emits typed events:

```rust
pub enum BlockEvent {
    PromptStart,
    CommandStart,
    CommandEnd { exit: i32 },
    OutputStart,
}
```

### 4. Block Manager (`crates/ibron-blocks/src/manager.rs`)

Owns a `VecDeque<Block>` per pane, capped at `max_blocks_per_pane` (default 1000, Lua-configurable). State machine:

```
Idle --[PromptStart]--> AwaitingCommand
AwaitingCommand --[CommandStart]--> CapturingCommand
CapturingCommand --[OutputStart]--> Running
Running --[CommandEnd]--> Idle (emits finished Block)
```

Handles malformed marker sequences by resetting to `Idle` — worst case, one block is lost, terminal stays usable.

### 5. Renderer integration (`crates/wezterm-gui/src/termwindow/render/paint.rs`)

Before painting each line, query `BlockManager.block_at(row)`. If the row is a block boundary, draw:

- **Left gutter**: 2px colored bar — green for success, red for failure, yellow for running.
- **Top edge**: 1px horizontal line separating from previous block.
- **Hover**: when mouse is over block, fade a subtle background tint.
- **Focused block**: slightly brighter gutter (keyboard nav).
- **Status glyph** in gutter near top: ✓ / ✗ / ⟳ (configurable in Lua).

All colors pulled from wezterm's color scheme system — no new theming primitives.

### 6. Input routing (`crates/wezterm-gui/src/termwindow/keyassignment.rs`)

Add new `KeyAssignment` variants:

```rust
BlockCopyCommand, BlockCopyOutput, BlockRerun, BlockShare,
BlockAskAI, BlockFold, BlockBookmark, BlockSearchOpen,
BlockFocusPrev, BlockFocusNext,
```

These become available in Lua config:

```lua
-- default bindings (user can override in ibron.lua)
{ key='c', mods='CTRL|SHIFT', action=wezterm.action.BlockCopyCommand },
{ key='y', mods='CTRL|SHIFT', action=wezterm.action.BlockCopyOutput },
{ key='r', mods='CTRL|SHIFT', action=wezterm.action.BlockRerun },
-- ...
```

Mouse: clicking the gutter of a block focuses it. Double-click copies command. Right-click opens block context menu (new — small modal, reuses wezterm's existing overlay system).

### 7. Operations (one module each under `crates/ibron-blocks/src/ops/`)

| Op | Module | Shipping behavior |
|----|--------|-------------------|
| A. Copy command | `copy_cmd.rs` | Writes `block.command` to system clipboard |
| B. Copy output | `copy_output.rs` | Writes block's captured output rows to clipboard |
| C. Re-run | `rerun.rs` | Sends `block.command + "\n"` to the current pane's pty |
| D. Share | `share.rs` | Formats block as markdown (```cmd\n...\n```\n\nOutput:\n...), copies to clipboard. No network upload in v0.1. |
| E. Ask AI | `ask_ai.rs` | v0.1: opens a new pane with the block serialized to stdin and runs `$IBRON_AI_CMD` (env var). Actual integration is v0.2. |
| F. Fold | `fold.rs` | Toggles `block.folded`; renderer skips output rows when folded and draws collapsed summary line instead |
| G. Bookmark | `bookmark.rs` | Toggles `block.bookmarked`; bookmark glyph shows in gutter; `BlockFocusNext` with bookmark-only modifier jumps between bookmarks |
| H. Search | `search.rs` | Opens a search overlay (new, reuses wezterm's existing search UI scaffolding). Queries across all blocks in this pane. Incremental, case-insensitive by default, regex toggle. |

Each op ships as its own PR with its own tests. Order matters: blocks must render before ops are testable, so PR order is: [render] → [copy_cmd + copy_output] → [rerun] → [fold] → [search] → [bookmark] → [share] → [ask_ai stub].

## Data flow (happy path)

1. User types `ls` in PowerShell. PSReadLine AcceptLine hook fires, emits `OSC 133 C ST` to stdout before handing off to the command.
2. ibron PTY reads the OSC; wezterm-term's parser dispatches it; `BlockEventSink::dispatch(OutputStart)` fires.
3. BlockManager transitions `CapturingCommand → Running`, stamps `output_start = current_row`, captures `command` string from the rows between B and C markers.
4. `ls` runs, output flows through pty. Renderer paints as normal. On each paint, renderer queries `block_at(row)` and draws the left gutter bar (yellow, because running).
5. `ls` finishes; PSReadLine's prompt function emits `OSC 133 D ; 0 ST`. BlockManager finalizes the block: `output_end = current_row - 1`, `exit_code = 0`, `state = Success`. Gutter flips to green.
6. User presses `Ctrl+Shift+Y`. Keybinding → `BlockCopyOutput` → op reads output rows → writes to clipboard.

## Error handling

| Situation | Behavior |
|-----------|----------|
| Shell integration not installed | No blocks rendered. Terminal behaves exactly like upstream wezterm. A one-time hint shown on first launch: "Run `ibron install-shell-integration` to enable command blocks." |
| Malformed OSC sequence | BlockManager resets state machine to `Idle`, logs at `debug` level, drops partial block. No user-visible error. |
| Shell sends `D` without preceding `C` | Treat as empty output block. |
| Command still running when pane closes | Block ends in `Running` state forever — rendered with a "(terminated)" label in gutter. |
| Scrollback eviction removes block rows | BlockManager evicts the block from history (keeps bookmarked blocks longer — configurable). |
| User has >1000 blocks | Oldest non-bookmarked blocks evicted first. |

## Testing

### Unit tests (fast, run in CI)
- OSC 133 parser: all marker combinations, malformed sequences, nested sequences.
- BlockManager state machine: every transition, every error path.
- Each op module: mock clipboard/pty, assert correct calls.

### Integration tests (slower, run in CI)
- Spawn a real `bash` subprocess with the integration script sourced, pipe commands through, assert BlockManager produced correct blocks.
- Same for `pwsh` on Windows CI runners.

### Visual checkpoints (manual, per PR)
Each PR that changes rendering includes a screenshot in its description. Checkpoint 1: "blocks visible with correct gutter colors." Checkpoint 2: "folded block shows collapsed summary." Checkpoint 3: "search overlay highlights matches." These are reviewed by the user personally.

### Regression guard
A "null shell integration" test: start ibron with no integration script, run 100 commands, verify zero blocks created and visual output matches a golden snapshot from upstream wezterm at the same commit.

## Rebrand scope (one-time, part of this milestone)

| Item | Change |
|------|--------|
| `Cargo.toml` workspace name | `wezterm` → `ibron` |
| Binary crate names | `wezterm`, `wezterm-gui`, `wezterm-mux-server` → `ibron`, `ibron-gui`, `ibron-mux-server` |
| Default config path | `~/.wezterm.lua` → `~/.ibron.lua` (with a fallback read of `~/.wezterm.lua` for one release; log a deprecation) |
| Window class | `org.wezfurlong.wezterm` → `com.ibrahimokdadov.ibron` |
| App display name | `WezTerm` → `ibron` |
| Icons / .ico / .icns | Placeholder for v0.1 (use a temporary mark; real icon design deferred) |
| README.md | Rewrite top-level README pointing at ibron's goals; keep `UPSTREAM-README.md` with original |
| LICENSE / NOTICE | Preserve MIT; add NOTICE crediting wezterm/wezterm |

Binaries that ship to users for v0.1 are **Windows-only** (x86_64). macOS/Linux builds continue to work for developers but are not release artifacts until v0.2.

## Roadmap (non-normative — separate specs follow)

- **v0.2 — AI error help.** When a block has `exit_code != 0`, `BlockAskAI` sends command + output + exit code + cwd + recent history to Claude API; inline suggestion rendered underneath the block. Needs: API key management, streaming response rendering, cost guardrails.
- **v0.3 — Smart autocomplete.** Ghost-text suggestions from history + LLM. Shell-integration adds command-logging hook.
- **v0.4 — Restore tabs/sessions.** Serialize pane tree + cwd + scrollback tail + open blocks to `~/.config/ibron/sessions/<name>.toml`. Restore on launch flag.
- **v0.5 — Sidebar.** Vertical panel on the left with: tab list (rich), per-tab live CPU/mem (polls `sysinfo` crate at 1Hz), folder tree for current cwd (reuses `ignore` crate for gitignore).

## Open questions (to revisit at v0.2 spec time, not blocking v0.1)

- Which AI provider? Default to Anthropic (user is an Anthropic customer; API key via env var).
- Should blocks persist across pane close? Probably yes for bookmarked blocks — deferred.
- Cross-pane block search vs per-pane? v0.1 is per-pane; global search is a v0.5 question.

## Success criteria for v0.1 shipping

1. User can run `ibron` on Windows, see their PowerShell prompt, run `dir`, and see a visually distinct block with green gutter.
2. User can press `Ctrl+Shift+C` on a focused block and paste the command somewhere else.
3. User can press `Ctrl+Shift+F` (or similar), type "dir", and see all `dir` blocks highlighted across scrollback.
4. Upstream wezterm's existing test suite still passes.
5. A fresh `git merge upstream/main` produces conflicts only in the ~5 known integration files.
