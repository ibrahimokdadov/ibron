# Phase 1 Scaffold Survey

Captured 2026-04-18 from wezterm upstream at commit `577474d89` ("Prevent \"screen scraping\", disable DECRQCRA (#7701)").

Reference for later phases. Re-run relevant surveys before each phase in case upstream changed.

## Workspace layout

Wezterm uses **top-level crate directories** (not `crates/*`). Workspace members as declared in root `Cargo.toml`:

```
bidi, bidi/generate, strip-ansi-escapes, sync-color-schemes,
deps/cairo, wezterm, wezterm-blob-leases, wezterm-cell,
wezterm-escape-parser, wezterm-dynamic, wezterm-gui,
wezterm-mux-server, wezterm-open-url, wezterm-ssh,
wezterm-surface, wezterm-uds
```

Many other crates exist at top level (e.g. `config/`, `term/`, `mux/`, `termwiz/`, `lua-api-crates/*`) and are pulled in via `[workspace.dependencies]` with `path = "..."`. Cargo auto-includes them as workspace members transitively.

**ibron convention (intentional deviation):** Our own crates go under `crates/ibron-*/` to visually segregate ibron-owned code from upstream. Adds ~2 characters to paths; pays for itself the first time `git merge upstream/main` conflicts need disambiguation.

## Binary crates (produce executables)

Detected by presence of `src/main.rs` (no explicit `[[bin]]` sections in any Cargo.toml):

- `wezterm/src/main.rs` → produces `wezterm` binary (CLI + mux client)
- `wezterm-gui/src/main.rs` → produces `wezterm-gui` binary (the GUI)
- `wezterm-mux-server/src/main.rs` → produces `wezterm-mux-server` binary (daemon)

These are the three binaries the rebrand phase (1b) will rename to `ibron`, `ibron-gui`, `ibron-mux-server`.

## Config path references

File: `config/src/config.rs`

| Line | Context |
|------|---------|
| 1015 | `HOME_DIR.join(".wezterm.lua")` — primary home config |
| 1017 | `dir.join("wezterm.lua")` — per-dir config |
| 1031 | `exe_dir.join("wezterm.lua")` — next to binary |
| 1064 | Comment |

Phase 1b should add `.ibron.lua` / `ibron.lua` paths to each list, kept first, with `.wezterm.lua` / `wezterm.lua` retained as deprecated fallbacks.

## Window class / app ID references

String: `org.wezfurlong.wezterm`

| File | Line | Role |
|------|------|------|
| `assets/flatpak/org.wezfurlong.wezterm.appdata.template.xml` | 4, 28 | Flatpak metadata |
| `assets/wezterm.appdata.xml` | 4, 28 | AppStream metadata |
| `assets/wezterm.desktop` | 5, 6 | Linux .desktop file |
| `wezterm-gui/src/main.rs` | 1175 | Windows AppUserModelID (taskbar grouping) |
| `wezterm-gui-subcommands/src/lib.rs` | 7 | `DEFAULT_WINDOW_CLASS` constant |
| `wezterm-gui-subcommands/src/lib.rs` | 65, 146, 178, 216 | Doc comments |
| `wezterm-toast-notification/src/dbus.rs` | 109 | DBus notification |
| `wezterm-toast-notification/src/windows.rs` | 81 | Windows toast |

Phase 1b changes: `org.wezfurlong.wezterm` → `com.ibrahimokdadov.ibron`. Many of these files also rename (e.g., `wezterm.desktop` → `ibron.desktop`), which is a bigger scope item for Phase 1b.

## Top-level README and license files

Present at root: `README.md`, `README-DISTRO-MAINTAINER.md`, `LICENSE.md`, `CONTRIBUTING.md`, `PRIVACY.md`, `Makefile`.

Phase 1b should: rename `README.md` → `UPSTREAM-README.md`, write a new `README.md` for ibron. Leave `LICENSE.md` untouched. NOTICE created in Phase 1 Task 4.

## OSC 133 — MAJOR FINDING

**Wezterm already parses OSC 133 semantic-prompt sequences.** This changes the architecture of Phase 3 significantly (the original spec assumed we'd write our own parser).

### What wezterm already has

File: `term/src/terminalstate/performer.rs` lines 863-902.

Wezterm handles these OSC 133 variants via `FinalTermSemanticPrompt`:

| Variant | What wezterm does |
|---------|-------------------|
| `FreshLine` | tracked |
| `FreshLineAndStartPrompt` | sets pen `SemanticType::Prompt` |
| `StartPrompt` | sets pen `SemanticType::Prompt` |
| `MarkEndOfCommandWithFreshLine` | sets pen `SemanticType::Prompt` |
| `MarkEndOfPromptAndStartOfInputUntilNextMarker` | sets pen `SemanticType::Input` |
| `MarkEndOfPromptAndStartOfInputUntilEndOfLine` | sets pen `SemanticType::Input` |
| `MarkEndOfInputAndStartOfOutput` | sets pen `SemanticType::Output` |
| `CommandStatus { .. }` | **NO-OP — exit code discarded** (line 901-902: `{}`) |

### Data types already in place

File: `term/src/lib.rs` lines 115-123:

```rust
pub struct SemanticZone {
    pub start_y: StableRowIndex,
    pub start_x: usize,
    pub end_y: StableRowIndex,
    pub end_x: usize,
    pub semantic_type: SemanticType,
}
```

`SemanticType` has Prompt, Input, Output variants (likely also a Default).

### APIs already exposed

File: `lua-api-crates/mux/src/pane.rs`:

- `get_semantic_zones(of_type)` — returns `Vec<SemanticZone>`, filterable by type
- `get_semantic_zone_at(x, y)` — zone lookup by screen coords
- `get_text_from_semantic_zone(zone)` — extracts text

File: `config/src/keyassignment.rs`:

- `MoveBackwardSemanticZone` / `MoveForwardSemanticZone` — keyboard nav between zones already exists

### What this changes for Phase 3

**Spec addendum (to write after Phase 1):**

- **Drop:** our own OSC 133 parser (`crates/ibron-blocks/src/parser.rs` from spec section 3). Upstream already does it.
- **Drop:** our own shell-integration delivery scheme (wezterm already ships `assets/shell-integration/` — examine in Phase 2).
- **Add:** teach upstream's `CommandStatus` handler to store the exit code. Minimal patch — one file.
- **Add:** a new "Block" concept = grouping of (Prompt zone, Input zone, Output zone, exit code). `ibron-blocks` crate builds blocks by walking the existing semantic zones.
- **Keep as designed:** block chrome rendering, the 8 operations, search UI, fold/bookmark state.

The win: Phase 3 shrinks from "build parser + state machine + renderer" to "walk existing zones + add exit-code storage + render chrome + implement ops."

### Shell integration — defer the dive

Wezterm already ships `assets/shell-integration/`. Before writing Phase 2, survey those files and reuse where possible. The original plan's "write ibron.ps1 / .sh / .zsh / .fish from scratch" is likely overkill — we may only need to rebrand filenames and tweak a few prompt sequences.

## TODO entries for future phases

- Phase 1b: apply all renames listed in "Window class / app ID references" and "Config path references" sections.
- Phase 2: survey `assets/shell-integration/` and decide what to reuse vs. rewrite.
- Phase 3 spec-update: simplify `ibron-blocks` crate scope given existing semantic-zone infrastructure.
- Phase 3 first code: patch `term/src/terminalstate/performer.rs` line 901 `CommandStatus { .. }` no-op to record exit code.
