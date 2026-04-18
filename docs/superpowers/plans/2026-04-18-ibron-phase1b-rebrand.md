# ibron Phase 1b: Rebrand — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename binary crates, update config paths, swap window class and related app-identity strings, and rewrite the top-level README, leaving ibron functionally identical to wezterm but cosmetically branded as its own product.

**Architecture:** Touch only the files enumerated in the phase 1 scaffold survey (`docs/superpowers/plans/notes/phase1-scaffold-survey.md`). Keep every ibron name change alongside a preserved fallback for the wezterm name where UX continuity matters (config file discovery). No feature code in this phase.

**Tech Stack:** Rust / Cargo workspace, git. Windows is the primary verification platform.

**Prerequisite:** Phase 1 complete (`git tag -l | grep ibron-phase1-scaffold` prints the tag).

---

## Scope boundaries

**In scope:**
- Binary crate renames (`wezterm` → `ibron`, `wezterm-gui` → `ibron-gui`, `wezterm-mux-server` → `ibron-mux-server`).
- Config file discovery: add `.ibron.lua` / `ibron.lua` paths first; keep `.wezterm.lua` / `wezterm.lua` as deprecated fallbacks with a log warning.
- Window class / app ID: `org.wezfurlong.wezterm` → `com.ibrahimokdadov.ibron`.
- Linux / flatpak metadata files: rename and update contents.
- Top-level README: rename upstream's to `UPSTREAM-README.md`, write a new `README.md` for ibron.
- Cargo package `authors` metadata: leave upstream `authors` intact; do not rewrite history or claim authorship of code we didn't write.

**Out of scope:**
- Library crate renames (`config`, `term`, `mux`, `termwiz`, `wezterm-font`, etc.). These are internal names; renaming them is a large amount of code churn for zero user-visible benefit. Defer indefinitely.
- Icon redesign. Placeholder is fine; real icons wait for product-design bandwidth.
- Documentation in `docs/` (upstream's MkDocs docs). Those get their own review later.
- CI workflows under `.github/workflows/`. Renaming the binary will break them in ways that need Windows-specific attention; handle in Phase 1c.

---

## Task 1: Survey verification

**Files:** none modified.

The phase 1 survey (`docs/superpowers/plans/notes/phase1-scaffold-survey.md`) enumerates every string that needs to change. Before editing, re-run the survey greps to catch anything new from an upstream merge between phase 1 and now.

- [ ] **Step 1: Re-grep the key strings**

```bash
grep -rn 'org\.wezfurlong' --include='*.rs' --include='*.toml' --include='*.xml' --include='*.desktop' --include='*.md' | wc -l
grep -rn '"wezterm-gui"' --include='*.rs' --include='*.toml' | wc -l
grep -rn 'wezterm\.lua\|\.wezterm\.lua' --include='*.rs' | wc -l
```

Note the counts. If any count is substantially higher than the numbers listed in the survey, re-read the survey and update it before proceeding.

- [ ] **Step 2: No commit**

Verification only.

---

## Task 2: Rename the `wezterm` (CLI) binary crate

**Files:**
- Rename: `wezterm/` → `ibron/`
- Modify: `wezterm/Cargo.toml` — package `name = "wezterm"` → `"ibron"`
- Modify: root `Cargo.toml` — `members` entry `"wezterm"` → `"ibron"`
- Modify: any `[workspace.dependencies]` entries pointing at `path = "wezterm"` (unlikely, since the CLI is a binary crate and shouldn't be a dependency)
- Modify: references in other crates' `Cargo.toml` files (grep for `wezterm = ` with `path =`)

- [ ] **Step 1: Rename the directory**

```bash
git mv wezterm ibron
```

- [ ] **Step 2: Update the package name**

Edit `ibron/Cargo.toml`: change `name = "wezterm"` to `name = "ibron"`. Leave `version`, `authors`, `edition`, all feature flags, all dependencies exactly as they were.

- [ ] **Step 3: Update workspace members**

Edit root `Cargo.toml`: in the `[workspace] members` list, change `"wezterm"` to `"ibron"`.

- [ ] **Step 4: Find referrers**

```bash
grep -rn 'path = "wezterm"' --include='*.toml'
grep -rn 'wezterm = {.*path' --include='*.toml'
```

For each match, update the path and (if the consumer imports the crate as `wezterm`) rename the import. If the binary crate is never imported by others, there should be zero matches — which is the expected case.

- [ ] **Step 5: Verify**

```bash
cargo check -p ibron 2>&1 | tail -5
```
Expected: `Checking ibron v...` followed by `Finished ...`.

- [ ] **Step 6: Commit**

```bash
git add -- ibron/ Cargo.toml
# plus any other files touched in step 4
git commit -m "refactor: rename wezterm binary crate to ibron

Rename the CLI binary's crate directory and package name. No
functional change — the binary still does what it did; only its
cargo name differs."
```

---

## Task 3: Rename the `wezterm-gui` binary crate

**Files:**
- Rename: `wezterm-gui/` → `ibron-gui/`
- Modify: `ibron-gui/Cargo.toml` — `name = "wezterm-gui"` → `"ibron-gui"`
- Modify: root `Cargo.toml` — `members` list
- Modify: any referrers

**Known referrer — the CLI spawns the GUI by name.** Before editing, grep for the exact string `wezterm-gui` (not just the path):

```bash
grep -rn '"wezterm-gui"' --include='*.rs' --include='*.toml' | head -20
```

Expect to find spawn logic in `ibron/src/` (formerly `wezterm/src/`) that launches the GUI subprocess. That call site must change to `"ibron-gui"`.

- [ ] **Step 1: Rename**

```bash
git mv wezterm-gui ibron-gui
```

- [ ] **Step 2: Update package name**

Edit `ibron-gui/Cargo.toml`: `name = "wezterm-gui"` → `"ibron-gui"`.

- [ ] **Step 3: Update workspace members**

Edit root `Cargo.toml`: `"wezterm-gui"` → `"ibron-gui"`.

- [ ] **Step 4: Update spawn references**

Edit every file containing the literal `"wezterm-gui"` as a spawn target (discovered in the pre-step grep). Change to `"ibron-gui"`. Do NOT blindly sed-replace — some matches may be documentation or historical comments where we want to preserve "wezterm-gui" as text.

- [ ] **Step 5: Verify**

```bash
cargo check -p ibron-gui 2>&1 | tail -5
```

- [ ] **Step 6: Commit**

```bash
git commit -m "refactor: rename wezterm-gui binary crate to ibron-gui

Rename the GUI binary and update every spawn call site in the
CLI so it looks for the new binary name."
```

Add the touched files individually (no `-A`).

---

## Task 4: Rename the `wezterm-mux-server` binary crate

Same pattern as Task 3.

**Files:**
- Rename: `wezterm-mux-server/` → `ibron-mux-server/`
- Modify: `ibron-mux-server/Cargo.toml` — package name
- Modify: root `Cargo.toml` — members list
- Modify: any spawn / service-name references

- [ ] **Step 1: Pre-grep**

```bash
grep -rn '"wezterm-mux-server"' --include='*.rs' --include='*.toml' --include='*.service' | head -20
```

- [ ] **Step 2: Rename**

```bash
git mv wezterm-mux-server ibron-mux-server
```

- [ ] **Step 3: Update package name**

Edit `ibron-mux-server/Cargo.toml`.

- [ ] **Step 4: Update workspace members**

Edit root `Cargo.toml`.

- [ ] **Step 5: Update referrers**

From step 1 grep. Be especially careful with systemd unit files (`.service`) — they may reference the binary by path.

- [ ] **Step 6: Verify**

```bash
cargo check -p ibron-mux-server 2>&1 | tail -5
```

- [ ] **Step 7: Commit**

```bash
git commit -m "refactor: rename wezterm-mux-server binary crate to ibron-mux-server"
```

---

## Task 5: Update config-file discovery

**File:** `config/src/config.rs`, lines 1015-1064 (verify line numbers against a fresh read — they may have shifted after upstream merges).

Current behavior: looks for `~/.wezterm.lua`, `<cwd>/wezterm.lua`, `<exe-dir>/wezterm.lua`.

Desired behavior: looks for ibron names first, falls back to wezterm names with a one-time deprecation log.

- [ ] **Step 1: Read the function**

Read lines 1000-1080 of `config/src/config.rs` to understand the current lookup function.

- [ ] **Step 2: Write the failing test**

Find the existing test module in `config/src/`. Add a test that asserts `.ibron.lua` is preferred over `.wezterm.lua` when both exist in HOME_DIR. Use `tempfile` crate for an isolated HOME.

The test must FAIL before the implementation is written. Exact test code depends on the existing test helper patterns — read the file before writing. **Don't invent types.**

- [ ] **Step 3: Implement**

Insert new `.ibron.lua` / `ibron.lua` entries at the START of each `paths` vec in the lookup function. Keep the existing `.wezterm.lua` / `wezterm.lua` entries immediately after. If an ibron path doesn't exist and a wezterm fallback does, emit a `log::warn!` once per process lifetime with text like: "Loading deprecated config path <wezterm-path>. Move to <ibron-path>."

- [ ] **Step 4: Verify**

```bash
cargo test -p config 2>&1 | tail -10
```

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(config): prefer .ibron.lua, fall back to .wezterm.lua

Adds new primary config paths (.ibron.lua in HOME, ibron.lua in
cwd/exe-dir). Keeps the old wezterm paths as fallbacks with a
one-time deprecation warning in the log, so existing users'
configs keep working across the rename."
```

---

## Task 6: Update the window class constant

**File:** `wezterm-gui-subcommands/src/lib.rs` line 7

Change `pub const DEFAULT_WINDOW_CLASS: &str = "org.wezfurlong.wezterm";` to `"com.ibrahimokdadov.ibron"`.

Also update the four doc comments that mention the default class (lines 65, 146, 178, 216 — verify before edit).

- [ ] **Step 1: Read the surrounding file**

`wezterm-gui-subcommands/src/lib.rs` full read (it's small).

- [ ] **Step 2: Edit the constant and doc comments**

Use Edit tool, one `old_string`/`new_string` pair per occurrence to preserve uniqueness.

- [ ] **Step 3: Build**

```bash
cargo check -p ibron-gui 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(ui): set default window class to com.ibrahimokdadov.ibron

This is the Windows AppUserModelID / Wayland app ID / X11
WM_CLASS that identifies ibron to the compositor and taskbar.
Users can still override it via --class on the command line."
```

---

## Task 7: Update remaining window-class references

From the survey, these files also reference `org.wezfurlong.wezterm`:

- `wezterm-gui/src/main.rs` line 1175 (Windows AppUserModelID call)
- `wezterm-toast-notification/src/dbus.rs` line 109
- `wezterm-toast-notification/src/windows.rs` line 81

And the Linux metadata files — these also need filename changes, not just content:
- `assets/flatpak/org.wezfurlong.wezterm.appdata.template.xml`
- `assets/wezterm.appdata.xml`
- `assets/wezterm.desktop`

- [ ] **Step 1: Update the three Rust files**

Each is a single string literal change. Use Edit with full context to avoid touching unrelated `org.wezfurlong` mentions (there shouldn't be any, but be deliberate).

- [ ] **Step 2: Rename the Linux metadata files**

```bash
git mv assets/flatpak/org.wezfurlong.wezterm.appdata.template.xml \
       assets/flatpak/com.ibrahimokdadov.ibron.appdata.template.xml
git mv assets/wezterm.appdata.xml assets/ibron.appdata.xml
git mv assets/wezterm.desktop assets/ibron.desktop
```

- [ ] **Step 3: Update file contents**

Inside each renamed file, replace `org.wezfurlong.wezterm` → `com.ibrahimokdadov.ibron`, `WezTerm` → `ibron` (user-visible name — keep case), `wezterm` → `ibron` (binary name in Exec= lines).

Be careful with `AppStream` categories and other non-branding content — leave those alone.

- [ ] **Step 4: Update CI / packaging scripts that reference the old filenames**

```bash
grep -rn 'wezterm\.desktop\|wezterm\.appdata\|org\.wezfurlong' ci/ Makefile assets/ --include='*.sh' --include='*.mk' 2>/dev/null
```

Update any matches. If any build scripts copy these files to system locations, the target paths may also need updating — fix at that time.

- [ ] **Step 5: Verify nothing else references the old strings**

```bash
grep -rn 'org\.wezfurlong' . 2>/dev/null | grep -v '^./\.git' | grep -v UPSTREAM-README
```

Expected: zero hits (or only historical references we preserve, like NOTICE / UPSTREAM-README).

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(ui): rename desktop/flatpak/appdata files to ibron

Renames Linux packaging metadata to the new app ID
com.ibrahimokdadov.ibron and replaces every remaining reference
to org.wezfurlong.wezterm in source with the new ID. Keeps
historical references in NOTICE / UPSTREAM-README intact."
```

---

## Task 8: Rewrite the top-level README

- [ ] **Step 1: Preserve upstream README**

```bash
git mv README.md UPSTREAM-README.md
```

- [ ] **Step 2: Write the new README**

Create a new `README.md` with this content (fill in placeholders marked `<...>`):

```markdown
# ibron

A fork of [WezTerm](https://github.com/wezterm/wezterm) aimed at
replacing Warp as a daily terminal on Windows. Adds Warp-style
command blocks, AI error assistance, and a few UX affordances wezterm
doesn't ship by default.

**Status:** Pre-alpha. The v0.1 line is command-blocks only.

## Why another terminal?

Warp is excellent but buggy on Windows + PowerShell and charges a
subscription. wezterm is rock-solid on Windows + PowerShell but
lacks Warp's block-oriented UX. ibron picks up where wezterm left
off and adds the pieces that make Warp feel modern.

## Install

Not yet. v0.1 isn't released.

## Build from source

```sh
git clone https://github.com/ibrahimokdadov/ibron
cd ibron
cargo build --release -p ibron-gui
```

Requires a recent Rust toolchain. See `UPSTREAM-README.md` for
detailed build prerequisites — they are unchanged from wezterm.

## Relationship to wezterm

ibron is a soft fork. Every upstream feature still works. Upstream
commits are periodically merged in via `git fetch upstream`. See
`NOTICE` for attribution; see `LICENSE.md` (MIT) for the license.

## Roadmap

- **v0.1** — Command blocks
- **v0.2** — AI error assistance
- **v0.3** — Smart autocomplete
- **v0.4** — Session / tab restore
- **v0.5** — Sidebar with live CPU/memory and folder tree

See `docs/superpowers/specs/` for design specs.

## License

MIT, same as upstream wezterm.
```

- [ ] **Step 3: Commit**

```bash
git commit -m "docs: replace README with ibron-specific content

Move upstream's README to UPSTREAM-README.md and write a new
top-level README describing ibron, the motivation for the fork,
the build command, and the v0.1-v0.5 roadmap."
```

---

## Task 9: Verify end-to-end

- [ ] **Step 1: Clean and full release build**

```bash
cargo clean
cargo build --release -p ibron-gui 2>&1 | tail -10
```

Expected: `Finished ...` — can take 15-30 minutes.

- [ ] **Step 2: Run the smoke test**

```bash
cargo test -p ibron-blocks 2>&1 | tail -5
```
Expected: pass.

- [ ] **Step 3: Confirm binary name**

```bash
ls target/release/ibron-gui.exe
```
Expected: file exists. If it's `wezterm-gui.exe`, the rename didn't flow through fully.

- [ ] **Step 4: Launch and sanity check (manual, user-driven)**

The engineer cannot test a GUI binary autonomously on this machine. Ask the user to:

1. Run `./target/release/ibron-gui.exe`.
2. Confirm the window opens and shows a PowerShell prompt.
3. Press `Ctrl+Shift+R` (default rebinding) or a known wezterm shortcut — confirm the keybinding still works.
4. Close the window.

If the user reports anything broken, stop and investigate before proceeding to Task 10.

---

## Task 10: Tag phase 1b completion

- [ ] **Step 1: Tag**

```bash
git tag -a ibron-phase1b-rebrand -m "Phase 1b complete: rebrand

Binary crates renamed (ibron, ibron-gui, ibron-mux-server),
config paths updated (.ibron.lua preferred, .wezterm.lua kept
as fallback), window class and desktop metadata swapped to
com.ibrahimokdadov.ibron, README rewritten.

Functionally identical to wezterm; purely cosmetic rebrand."
```

- [ ] **Step 2: Summarize**

In your response: confirm every task completed, paste the tag hash, note any deviations from this plan, and list what the user should manually verify on their next run.

---

## Self-review checklist

- [ ] No library crate was renamed (out of scope).
- [ ] Every binary crate rename includes both the directory move, the `Cargo.toml` `name` change, the workspace `members` update, and the spawn-callsite update.
- [ ] Config path change ships with a deprecation warning on the old path, not a silent fallback.
- [ ] `cargo build --release -p ibron-gui` produces `target/release/ibron-gui.exe`.
- [ ] `grep -rn 'org\.wezfurlong' .` in a clean working tree returns only historical references (NOTICE, UPSTREAM-README).
- [ ] User confirmed the GUI launches and keybindings work.
