# ibron Phase 1: Fork and Scaffold — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge wezterm upstream into this repo, scaffold an empty `crates/ibron-blocks` crate registered in the workspace, and verify the tree still builds cleanly on Windows. No rebrand or feature code in this phase — produce a "wezterm-plus-empty-placeholder" tree that builds.

**Architecture:** This repo was initialized with a single root commit (the design spec). Phase 1 fetches `wezterm/wezterm` as `upstream`, rebases our spec commit onto `upstream/main`, then adds one new workspace crate (`ibron-blocks`) with a single empty `lib.rs`. The crate is registered in the workspace `Cargo.toml` but has no dependents yet, so its inclusion is a no-op at link time.

**Tech Stack:** Rust (stable toolchain), Cargo workspace, git, PowerShell on Windows.

---

## Prerequisites (check before starting)

- Rust toolchain installed: `rustc --version` prints 1.7x or newer
- `cargo --version` works
- `git --version` works
- ~3 GB free disk (wezterm's full history + build artifacts)
- The background `git fetch upstream main` started earlier has completed. Verify with: `git log upstream/main --oneline | head -1`. If it prints a commit, fetch is done.

## File Structure

Files this plan creates or modifies:

**Created:**
- `NOTICE` — attribution to wezterm/wezterm (MIT)
- `.gitattributes` — force LF line endings for `.rs`, `.toml`, `.md`, `.sh` to avoid CRLF churn on Windows
- `crates/ibron-blocks/Cargo.toml` — package definition
- `crates/ibron-blocks/src/lib.rs` — single empty module with a smoke test
- `docs/superpowers/plans/notes/phase1-scaffold-survey.md` — survey of wezterm's structure produced during Task 2 (reference for later phases)

**Modified:**
- `Cargo.toml` (workspace root) — add `crates/ibron-blocks` to `members`

**Important:** Do not modify any existing wezterm source file in Phase 1. If you're tempted to "quickly fix" something, stop — it belongs in a later phase.

---

## Task 1: Merge wezterm upstream into our repo

**Files:** none created; git history changes.

- [ ] **Step 1: Verify the fetch completed**

Run:
```bash
git log upstream/main --oneline | head -1
```
Expected: a commit hash + message. If no output, wait for the background fetch to finish and retry.

- [ ] **Step 2: Save our pre-merge commit hashes**

There are one or more commits on `main` that don't exist in `upstream/main` (at minimum the design spec, possibly also this plan). Capture all of them in chronological order so we can cherry-pick them back.

Run:
```bash
git log --reverse --format=%H main > /tmp/ibron-precommits.txt
cat /tmp/ibron-precommits.txt
```
Expected: one hash per line, oldest first.

- [ ] **Step 3: Reset our `main` branch to `upstream/main`**

Run:
```bash
git reset --hard upstream/main
```
Expected: `HEAD is now at <upstream-hash> <upstream-msg>`. The working tree now looks like wezterm's.

- [ ] **Step 4: Cherry-pick our pre-merge commits on top**

Run:
```bash
while IFS= read -r hash; do
  git cherry-pick "$hash" || { echo "Conflict on $hash"; exit 1; }
done < /tmp/ibron-precommits.txt
```

Expected: one `[main ...]` line per commit. If a cherry-pick conflicts (unlikely — we only added new subdirectories under `docs/`), keep our version, `git cherry-pick --continue`, and let the loop resume.

- [ ] **Step 5: Verify the tree**

Run:
```bash
ls
git log --oneline | head -5
```
Expected: `ls` shows wezterm's directory structure (`Cargo.toml`, `crates/`, `docs/`, `wezterm-gui/`, etc.). `git log` shows our spec commit on top of a recent wezterm commit.

- [ ] **Step 6: Push to origin (ibron repo) — DEFERRED**

Skip for now. `origin` isn't set up yet (the GitHub repo `ibrahimokdadov/ibron` hasn't been created). Note this as a followup. Do NOT create the GitHub repo in this plan — that's a user action with permission implications.

- [ ] **Step 7: No commit needed**

The cherry-pick already produced a commit. Nothing to add.

---

## Task 2: Survey the wezterm tree

**Files:** create `docs/superpowers/plans/notes/phase1-scaffold-survey.md`

This survey captures the information later phases will need so they don't have to re-explore. Later phases reference this file by path.

- [ ] **Step 1: Create the notes directory**

Run:
```bash
mkdir -p docs/superpowers/plans/notes
```

- [ ] **Step 2: Gather the facts**

Run each of these and record the output in the survey file:

```bash
# Workspace members
grep -A 200 '^members' Cargo.toml | head -100

# Binary crates (look for ones that produce executables)
find crates -name 'Cargo.toml' -exec grep -l '\[\[bin\]\]' {} \;

# Config file path constants
grep -rn 'wezterm.lua' --include='*.rs' | head -20

# Window class / app identifier strings
grep -rn 'org.wezfurlong' --include='*.rs' --include='*.toml' | head -20
grep -rn 'WezTerm' --include='*.rs' --include='*.toml' --include='*.md' | head -40

# Top-level README to preserve
ls README*

# License + notice files
ls LICENSE* NOTICE* 2>/dev/null

# OSC dispatch location (for later phases)
grep -rln 'osc_dispatch\|Osc::\|133' crates/wezterm-term/src crates/termwiz/src | head -10
```

- [ ] **Step 3: Write the survey file**

Create `docs/superpowers/plans/notes/phase1-scaffold-survey.md` with this exact structure, filling in the values from Step 2:

```markdown
# Phase 1 Scaffold Survey

Captured 2026-04-18 from wezterm upstream at commit <upstream-hash>.
Reference for later phases. Re-run surveys before each phase in case upstream changed.

## Workspace members (from Cargo.toml)
<paste the members list>

## Binary crates (produce executables)
<paste list of crates with [[bin]] sections>

## Config path references (`wezterm.lua`)
<paste grep output>

## Window class / app ID references (`org.wezfurlong`)
<paste grep output>

## Product name references (`WezTerm`)
<paste grep output>

## Top-level README and license files
<paste ls output>

## OSC dispatch code locations (used in later phases)
<paste grep output>
```

- [ ] **Step 4: Commit the survey**

Run:
```bash
git add docs/superpowers/plans/notes/phase1-scaffold-survey.md
git commit -m "docs: add phase 1 scaffold survey of upstream wezterm

Captures workspace structure, binary crates, config path refs,
window class, and OSC dispatch locations. Reference for later
phases so we don't re-explore."
```

Expected: one new commit.

---

## Task 3: Add `.gitattributes` to normalize line endings

**Files:** create `.gitattributes` at repo root.

Rationale: on the earlier spec commit we saw `warning: LF will be replaced by CRLF`. That creates noisy diffs on Windows. Normalize everything we care about to LF.

- [ ] **Step 1: Create the file**

Create `.gitattributes` with these contents:

```
* text=auto eol=lf

# Rust
*.rs text eol=lf
*.toml text eol=lf

# Shell / scripts
*.sh text eol=lf
*.ps1 text eol=crlf
*.zsh text eol=lf
*.fish text eol=lf

# Docs
*.md text eol=lf

# Binary formats — leave alone
*.png binary
*.jpg binary
*.ico binary
*.icns binary
*.woff binary
*.woff2 binary
*.ttf binary
*.otf binary
```

- [ ] **Step 2: Renormalize existing files**

Run:
```bash
git add --renormalize .
git status
```

Expected: possibly many "modified" entries if Windows already converted some files to CRLF. This is fine — the renormalize records them with LF from now on.

- [ ] **Step 3: Commit**

Run:
```bash
git add .gitattributes
git commit -m "chore: add .gitattributes for consistent line endings

Windows was converting LF to CRLF on checkout, producing noisy
diffs. Force LF for source, markdown, and unix shell scripts;
leave .ps1 as CRLF (PowerShell convention); mark binary formats
as binary so git doesn't try to normalize them."
```

If step 2 produced a lot of "modified" entries, they should be included in this commit (since they're all just line-ending fixes). Check with `git status` after committing — if any remain modified, that's a bug in the attribute patterns; investigate before proceeding.

---

## Task 4: Add NOTICE file crediting wezterm

**Files:** create `NOTICE` at repo root.

- [ ] **Step 1: Check for existing NOTICE**

Run:
```bash
ls NOTICE* 2>/dev/null
```
Expected: no output (wezterm doesn't ship a NOTICE file).

- [ ] **Step 2: Create the NOTICE**

Create `NOTICE` with these contents:

```
ibron
Copyright 2026 Ibrahim Mokdad

This product is a fork of WezTerm (https://github.com/wezterm/wezterm),
licensed under the MIT License. See the included LICENSE.md for the
full license text.

Upstream WezTerm copyright:
  Copyright (c) 2018-present Wez Furlong
```

- [ ] **Step 3: Commit**

Run:
```bash
git add NOTICE
git commit -m "chore: add NOTICE crediting upstream wezterm

MIT only requires preservation of the license text, not a NOTICE,
but adding one makes the attribution explicit and sets the stage
for the rebrand in phase 1b."
```

---

## Task 5: Scaffold `crates/ibron-blocks` empty crate

**Files:**
- Create: `crates/ibron-blocks/Cargo.toml`
- Create: `crates/ibron-blocks/src/lib.rs`
- Modify: `Cargo.toml` (workspace root) — add member

- [ ] **Step 1: Create the crate directory**

Run:
```bash
mkdir -p crates/ibron-blocks/src
```

- [ ] **Step 2: Write the crate `Cargo.toml`**

Create `crates/ibron-blocks/Cargo.toml`:

```toml
[package]
name = "ibron-blocks"
version = "0.0.0"
edition = "2021"
license = "MIT"
description = "ibron command-blocks layer: parses OSC 133, manages block history, renders block chrome."
repository = "https://github.com/ibrahimokdadov/ibron"
publish = false

[dependencies]

[dev-dependencies]
```

- [ ] **Step 3: Write the failing smoke test**

Create `crates/ibron-blocks/src/lib.rs`:

```rust
//! ibron-blocks: command-block layer for the ibron terminal.
//!
//! This crate is empty in phase 1 — it exists only to reserve the workspace
//! slot and verify the build graph. Real code arrives in phase 3.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_exists() {
        // This is the canary: if this passes, the crate compiles,
        // is registered in the workspace, and the test harness runs.
        assert_eq!(2 + 2, 4);
    }
}
```

- [ ] **Step 4: Register the crate in the workspace**

Open the root `Cargo.toml` and locate the `[workspace]` section's `members` list. Add `"crates/ibron-blocks"` to it, preserving alphabetical order if upstream maintains one.

Use the Edit tool if available. Otherwise, manually verify the `members = [...]` block now contains `"crates/ibron-blocks"`.

Do not change any other field in the root `Cargo.toml`.

- [ ] **Step 5: Verify the crate builds**

Run:
```bash
cargo check -p ibron-blocks
```
Expected: compiles without errors. If it says "package not found," the workspace registration in Step 4 didn't land — go back and fix.

- [ ] **Step 6: Run the smoke test**

Run:
```bash
cargo test -p ibron-blocks
```
Expected:
```
running 1 test
test tests::crate_exists ... ok

test result: ok. 1 passed; 0 failed
```

If the test fails, something is very wrong with the Rust toolchain. Investigate before proceeding.

- [ ] **Step 7: Commit**

Run:
```bash
git add crates/ibron-blocks/Cargo.toml crates/ibron-blocks/src/lib.rs Cargo.toml
git commit -m "feat: scaffold empty ibron-blocks crate

Phase 1 scaffold. The crate is registered in the workspace with a
single smoke test asserting it compiles. Real code arrives in
phase 3 (block manager + OSC 133 parser).

Placed under crates/ibron-blocks/ to match upstream wezterm's
directory convention."
```

---

## Task 6: Verify the full tree still builds

This catches any damage Task 1-5 may have caused to the workspace.

- [ ] **Step 1: Clean prior artifacts**

Run:
```bash
cargo clean
```
Expected: completes without error. Frees up disk for a fresh build.

- [ ] **Step 2: Build the main binary**

Run:
```bash
cargo build --release -p wezterm-gui
```

Expected: compiles successfully. This can take 15-30 minutes on first run (wezterm has hundreds of dependencies). Do not interrupt.

If this fails with errors referring to `ibron-blocks` or `Cargo.toml`, the workspace registration is broken — go back to Task 5 Step 4.

If it fails with errors in unrelated upstream code, something went wrong in the upstream merge (Task 1). Investigate.

- [ ] **Step 3: Run the smoke test one more time**

Run:
```bash
cargo test -p ibron-blocks
```

Expected: passes.

- [ ] **Step 4: No new commit**

Nothing to commit — this task is just verification.

---

## Task 7: Tag the phase 1 completion

- [ ] **Step 1: Tag**

Run:
```bash
git tag -a ibron-phase1-scaffold -m "Phase 1 complete: fork merged, scaffold ready

Tree is wezterm upstream + docs/superpowers/{specs,plans}/ +
NOTICE + .gitattributes + empty crates/ibron-blocks.
Still identical to wezterm functionally. Binaries still named
wezterm*; rebrand arrives in phase 1b."
```

Expected: tag created. Verify with `git tag -l | grep ibron`.

- [ ] **Step 2: Summarize to the user**

In your response back, list:
- The commit hashes produced this phase (git log --oneline main ^upstream/main)
- That `cargo build --release -p wezterm-gui` succeeded
- That the smoke test passed
- Next plan to write: `phase1b-rebrand.md`

---

## Self-review checklist (run at end before declaring done)

- [ ] Every task has concrete commands, not "figure out X" instructions.
- [ ] `cargo build --release -p wezterm-gui` succeeds end-to-end.
- [ ] `cargo test -p ibron-blocks` passes.
- [ ] No source file under `crates/` other than `crates/ibron-blocks/**` was modified.
- [ ] `git log` shows a clean history: wezterm upstream + our spec commit + Phase 1 commits + tag.
- [ ] Tag `ibron-phase1-scaffold` exists.

## What comes next (not in this plan)

- **Phase 1b — Rebrand.** Rename binary crates, update config path fallback, swap window class, rewrite README. Written after Phase 1 completes and the scaffold survey exists.
- **Phase 2 — Shell integration.** OSC 133 emitters for PowerShell/bash/zsh/fish + `ibron install-shell-integration` subcommand.
- **Phase 3 — Block manager core.** Populate `crates/ibron-blocks` with Block data model, OSC 133 parser, state machine, and unit tests.
- **Phase 4 — Renderer integration.** Draw block chrome, route input.
- **Phase 5 — Operations A-H.** Each op its own sub-plan.
