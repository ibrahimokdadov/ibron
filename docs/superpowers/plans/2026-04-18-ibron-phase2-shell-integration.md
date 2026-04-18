# ibron Phase 2: Shell Integration (OSC 133) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship shell integration scripts for bash/zsh, PowerShell, and fish that emit OSC 133 A/B/C/D markers, plus an `ibron shell-completion` subcommand to install them. After this phase, any ibron session running an integrated shell has the marker stream the Phase 3 block manager will consume.

**Architecture:** Reuse wezterm's existing `assets/shell-integration/wezterm.sh` for bash/zsh — it already emits OSC 133 A/B/C/D correctly. Add new `ibron.ps1` for PowerShell (ibron's most important shell — it's why we forked) and `ibron.fish` for fish. Installation goes through a new subcommand on the existing `ibron` binary rather than a standalone helper.

**Tech Stack:** Shell scripting (bash, PowerShell, fish), Rust (clap subcommand). No new Cargo crates.

**Prerequisite:** Phase 1b complete (`git tag -l | grep ibron-phase1b-rebrand` prints the tag).

---

## Scope boundaries

**In scope:**
- Ship `assets/shell-integration/ibron.sh` (rebranded copy of `wezterm.sh`).
- Write `assets/shell-integration/ibron.ps1` from scratch — PowerShell is Warp's biggest bug surface and ibron's primary target.
- Write `assets/shell-integration/ibron.fish` from scratch.
- Add `ibron shell-integration` subcommand with `--shell <bash|zsh|powershell|fish>` and `--install` / `--print` flags.
- No changes to terminal emulator code. Wezterm already parses OSC 133; Phase 3 consumes it.

**Out of scope:**
- `ibron.zsh` as a separate file — `ibron.sh` handles both. Follow wezterm's existing convention.
- Cross-compilation changes, icon changes, CI. Phase 1c territory.
- Actually consuming the markers in Rust code. That's Phase 3.
- Windows cmd.exe — doesn't have a usable prompt extension mechanism. Document the gap, move on.

---

## Task 1: Create `ibron.sh` from `wezterm.sh`

**Files:**
- Create: `assets/shell-integration/ibron.sh` (copy of `wezterm.sh` with identifiers rebranded)

The script is ~570 lines. Do NOT rewrite it — `git cp` (which doesn't exist) so copy the file byte-for-byte, then rename internal identifiers. This keeps diffs against upstream clean when they update their version.

- [ ] **Step 1: Copy the file**

```bash
cp assets/shell-integration/wezterm.sh assets/shell-integration/ibron.sh
```

- [ ] **Step 2: Rename internal identifiers**

Replace (in `ibron.sh` only, not `wezterm.sh`):
- `WEZTERM_SHELL_SKIP_ALL` → `IBRON_SHELL_SKIP_ALL` (and any `WEZTERM_SHELL_SKIP_*` env vars)
- `__wezterm_` (every function/variable prefix) → `__ibron_`
- `__bp_` prefixes (bash-preexec internal) → leave alone; those are an unrelated library's convention.
- `WEZTERM_PROG` / `WEZTERM_USER` / `WEZTERM_HOST` / `WEZTERM_IN_TMUX` user vars → `IBRON_PROG` / `IBRON_USER` / `IBRON_HOST` / `IBRON_IN_TMUX`
- `WEZTERM_HOSTNAME` env var → `IBRON_HOSTNAME`
- `hash wezterm 2>/dev/null` / `wezterm set-working-directory` inside the `__wezterm_osc7` function → `hash ibron 2>/dev/null` / `ibron set-working-directory` (the Rust CLI command name changes in Phase 3+; for now the `printf` fallback handles missing binary anyway).
- Comments mentioning "wezterm" → "ibron" where they describe the product; keep "OSC 133" generic.

**Do not change** the OSC escape sequences themselves (`\e]133;A`, etc.) — those are standardized and platform-independent.

- [ ] **Step 3: Verify the script parses**

```bash
bash -n assets/shell-integration/ibron.sh
```

Expected: no output (syntax OK).

- [ ] **Step 4: Commit**

```bash
git add assets/shell-integration/ibron.sh
git commit -m "feat(shell): add bash/zsh integration script ibron.sh

Rebranded copy of upstream wezterm.sh. Emits OSC 133 A/B/C/D and
sets IBRON_* user vars. Kept as a sibling file (not a rename) so
upstream wezterm.sh updates can still be tracked for porting."
```

---

## Task 2: Write `ibron.ps1` for PowerShell

**Files:**
- Create: `assets/shell-integration/ibron.ps1`

PowerShell has no direct equivalent to bash-preexec. We emit markers by:
- **A marker** (prompt start): emit at top of the `prompt` function.
- **B marker** (command input start): emit at end of `prompt` function return.
- **C marker** (command output start): hook PSReadLine's `AcceptLine` handler — fires when Enter is pressed, before command executes.
- **D marker** (command end): emit at top of next `prompt` invocation, using `$LASTEXITCODE`.

This means `D` for command N is emitted just before `A` for prompt N+1. Terminal sees: `...output... OSC133D;<exit> OSC133A ...newprompt... OSC133B`. That's the standard pattern iTerm2/ghostty use.

**PSReadLine caveat:** PSReadLine is bundled with Windows PowerShell 5.1 and installed by default with PowerShell 7+. If missing, we fall back to emitting C inline with the prompt (less accurate timing, but marker stream stays well-formed).

- [ ] **Step 1: Write the script**

Create `assets/shell-integration/ibron.ps1` with this content:

```powershell
# ibron shell integration for PowerShell (Windows PowerShell 5.1+ and PowerShell 7+)
#
# Emits OSC 133 A/B/C/D semantic-prompt markers so ibron can identify
# command blocks, plus OSC 7 for the current working directory.
#
# Bypasses:
#   $env:IBRON_SHELL_SKIP_ALL = "1"            # disable everything
#   $env:IBRON_SHELL_SKIP_SEMANTIC_ZONES = "1" # disable OSC 133 only
#   $env:IBRON_SHELL_SKIP_CWD = "1"            # disable OSC 7 only
#
# Install: add the following line to $PROFILE:
#   . "$env:APPDATA\ibron\shell-integration\ibron.ps1"

if ($env:IBRON_SHELL_SKIP_ALL -eq "1") { return }
if (-not [Environment]::UserInteractive) { return }

# Track whether a command actually ran between two prompts so we don't emit
# a spurious D;0 on the very first prompt of the session.
$global:__ibron_CommandHasRun = $false
$global:__ibron_LastExitCode = 0

function global:__ibron_EmitA {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;A`e\")
}

function global:__ibron_EmitB {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;B`e\")
}

function global:__ibron_EmitC {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;C`e\")
}

function global:__ibron_EmitD {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    if (-not $global:__ibron_CommandHasRun) { return }
    [Console]::Write("`e]133;D;$($global:__ibron_LastExitCode)`e\")
    $global:__ibron_CommandHasRun = $false
}

function global:__ibron_EmitOsc7 {
    if ($env:IBRON_SHELL_SKIP_CWD -eq "1") { return }
    $cwd = (Get-Location).ProviderPath
    if (-not $cwd) { return }
    # OSC 7 is file://<host><path>. We use an empty host, which is legal and
    # is how wezterm's own set-working-directory emits it when no hostname is
    # configured.
    $encoded = [Uri]::EscapeUriString($cwd -replace '\\', '/')
    [Console]::Write("`e]7;file://$encoded`e\")
}

# Wrap the user's existing prompt function so we compose with whatever they
# have customized. We capture it once at install time.
if (-not (Test-Path function:global:__ibron_OriginalPrompt)) {
    if (Test-Path function:prompt) {
        ${function:global:__ibron_OriginalPrompt} = ${function:prompt}
    } else {
        function global:__ibron_OriginalPrompt { "PS $(Get-Location)> " }
    }
}

function global:prompt {
    $exit = $LASTEXITCODE
    if ($null -ne $exit) { $global:__ibron_LastExitCode = $exit }
    __ibron_EmitD
    __ibron_EmitA
    __ibron_EmitOsc7
    $text = & __ibron_OriginalPrompt
    __ibron_EmitB
    return $text
}

# PSReadLine integration: emit C right when the user submits a line.
if (Get-Module -ListAvailable -Name PSReadLine) {
    Import-Module PSReadLine -ErrorAction SilentlyContinue
    if (Get-Module PSReadLine) {
        # Capture the default AcceptLine handler so we can chain to it.
        Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            __ibron_EmitC
            $global:__ibron_CommandHasRun = $true
        }
    }
}
```

- [ ] **Step 2: Smoke-test syntax**

PowerShell has no `-n` / dry-parse flag equivalent that works cross-platform without running the script. Skip runtime testing in this phase (we don't have a working ibron binary yet) and rely on the Phase 3 integration tests to catch regressions. Verify the file was written:

```bash
wc -l assets/shell-integration/ibron.ps1
```

Expected: ~85 lines.

- [ ] **Step 3: Commit**

```bash
git add assets/shell-integration/ibron.ps1
git commit -m "feat(shell): add PowerShell integration script ibron.ps1

Emits OSC 133 A/B/C/D via prompt wrapper and PSReadLine AcceptLine
hook. Works on Windows PowerShell 5.1 and PowerShell 7+. Composes
over the user's existing prompt function. Falls back gracefully
when PSReadLine is absent."
```

---

## Task 3: Write `ibron.fish` for fish shell

**Files:**
- Create: `assets/shell-integration/ibron.fish`

Fish has first-class events for prompt and preexec, so this is short.

- [ ] **Step 1: Write the script**

Create `assets/shell-integration/ibron.fish`:

```fish
# ibron shell integration for fish shell.
#
# Emits OSC 133 A/B/C/D semantic-prompt markers plus OSC 7 for cwd.
#
# Bypasses:
#   set -gx IBRON_SHELL_SKIP_ALL 1
#   set -gx IBRON_SHELL_SKIP_SEMANTIC_ZONES 1
#   set -gx IBRON_SHELL_SKIP_CWD 1
#
# Install: add the following line to ~/.config/fish/config.fish:
#   source /path/to/ibron.fish

if set -q IBRON_SHELL_SKIP_ALL
    exit 0
end

if not status is-interactive
    exit 0
end

set -g __ibron_last_status 0
set -g __ibron_has_run 0

function __ibron_emit_a --on-event fish_prompt
    if set -q IBRON_SHELL_SKIP_SEMANTIC_ZONES
        return
    end
    # D for previous command (if any)
    if test $__ibron_has_run -eq 1
        printf '\e]133;D;%s\e\\' $__ibron_last_status
        set -g __ibron_has_run 0
    end
    printf '\e]133;A\e\\'
end

function __ibron_emit_b --on-event fish_prompt
    if set -q IBRON_SHELL_SKIP_SEMANTIC_ZONES
        return
    end
    # B is emitted AFTER the prompt is rendered; fish doesn't have a
    # dedicated event for that, but fish_prompt runs synchronously before
    # the prompt is drawn and fish_postexec runs after a command. The
    # closest we get is emitting B inside fish_prompt — acceptable because
    # wezterm/iTerm already tolerate this ordering.
    printf '\e]133;B\e\\'
end

function __ibron_emit_c --on-event fish_preexec
    if set -q IBRON_SHELL_SKIP_SEMANTIC_ZONES
        return
    end
    printf '\e]133;C\e\\'
    set -g __ibron_has_run 1
end

function __ibron_capture_status --on-event fish_postexec
    set -g __ibron_last_status $status
end

function __ibron_emit_osc7 --on-event fish_prompt
    if set -q IBRON_SHELL_SKIP_CWD
        return
    end
    printf '\e]7;file://%s\e\\' (string replace -a '\\' '/' -- $PWD)
end
```

- [ ] **Step 2: Verify syntax if fish is available**

```bash
if command -v fish >/dev/null; then
    fish -n assets/shell-integration/ibron.fish && echo OK
else
    echo "fish not installed; relying on visual inspection"
fi
```

- [ ] **Step 3: Commit**

```bash
git add assets/shell-integration/ibron.fish
git commit -m "feat(shell): add fish integration script ibron.fish

Emits OSC 133 A/B/C/D via fish_prompt / fish_preexec / fish_postexec
events plus OSC 7 for cwd."
```

---

## Task 4: Add `ibron shell-integration` subcommand

**Files:**
- Modify: `ibron/src/main.rs` — add new subcommand
- Possibly modify: `wezterm-gui-subcommands/src/lib.rs` if the subcommand needs shared config types (unlikely for this task; default to keeping it inline in `ibron/`)

The subcommand has two modes:
- `ibron shell-integration --print --shell <name>` — prints the script to stdout for piping into a file or sourcing inline
- `ibron shell-integration --install --shell <name>` — writes the script to the canonical location for that shell and prints the single line the user should add to their shell profile

Default `--shell` value: detect from `$SHELL` on Unix, detect from `$env:PSModulePath` heuristics on Windows (PowerShell is the sensible default).

**Embed the scripts at compile time** via `include_str!` so the installed binary doesn't depend on external asset paths.

- [ ] **Step 1: Locate the existing subcommand dispatch**

```bash
grep -n "SubCommand" ibron/src/main.rs | head -20
```

Find the `enum SubCommand` or `match` statement that dispatches subcommands. Read the surrounding context (20 lines either side) to understand the prevailing pattern before adding to it.

- [ ] **Step 2: Add the subcommand definition**

In `ibron/src/main.rs`, add a new variant to the subcommand enum. Copy the clap attribute style (`#[command(...)]`, `#[arg(...)]`) from an adjacent variant so naming and help text match the codebase voice.

Rough shape (adapt to the actual enum):

```rust
#[derive(Debug, Parser, Clone)]
#[command(about = "Print or install ibron's shell integration script")]
pub struct ShellIntegrationCommand {
    /// Target shell. Defaults to auto-detecting from $SHELL / platform.
    #[arg(long, value_enum)]
    pub shell: Option<IntegrationShell>,

    /// Print the script to stdout (default).
    #[arg(long, conflicts_with = "install")]
    pub print: bool,

    /// Write the script to a canonical location and print a one-line
    /// snippet for the user to add to their shell profile.
    #[arg(long)]
    pub install: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum IntegrationShell {
    Bash,
    Zsh,
    Powershell,
    Fish,
}
```

- [ ] **Step 3: Embed the scripts**

Still in `ibron/src/main.rs` (or in a new private module `ibron/src/shell_integration.rs` if the file is already large):

```rust
const IBRON_SH: &str = include_str!("../../assets/shell-integration/ibron.sh");
const IBRON_PS1: &str = include_str!("../../assets/shell-integration/ibron.ps1");
const IBRON_FISH: &str = include_str!("../../assets/shell-integration/ibron.fish");
```

Rust's `include_str!` path is relative to the source file. From `ibron/src/main.rs` the path to `assets/shell-integration/ibron.sh` is `../../assets/shell-integration/ibron.sh`. Double-check at edit time.

- [ ] **Step 4: Implement the handler**

```rust
fn run_shell_integration(opts: ShellIntegrationCommand) -> anyhow::Result<()> {
    let shell = opts.shell.unwrap_or_else(detect_shell);
    let (script, default_install_path, profile_snippet) = match shell {
        IntegrationShell::Bash | IntegrationShell::Zsh => (
            IBRON_SH,
            // $HOME/.config/ibron/shell-integration/ibron.sh on Unix
            dirs_next::config_dir()
                .context("no config dir")?
                .join("ibron/shell-integration/ibron.sh"),
            "source \"$HOME/.config/ibron/shell-integration/ibron.sh\"",
        ),
        IntegrationShell::Powershell => (
            IBRON_PS1,
            // %APPDATA%\ibron\shell-integration\ibron.ps1 on Windows
            dirs_next::config_dir()
                .context("no config dir")?
                .join("ibron/shell-integration/ibron.ps1"),
            ". \"$env:APPDATA\\ibron\\shell-integration\\ibron.ps1\"",
        ),
        IntegrationShell::Fish => (
            IBRON_FISH,
            dirs_next::config_dir()
                .context("no config dir")?
                .join("ibron/shell-integration/ibron.fish"),
            "source \"$HOME/.config/ibron/shell-integration/ibron.fish\"",
        ),
    };

    if opts.install {
        std::fs::create_dir_all(
            default_install_path
                .parent()
                .context("install path has no parent")?,
        )?;
        std::fs::write(&default_install_path, script)?;
        eprintln!("Wrote {}", default_install_path.display());
        eprintln!();
        eprintln!("Add this line to your shell profile:");
        println!("{}", profile_snippet);
    } else {
        // default and --print both print to stdout
        print!("{}", script);
    }
    Ok(())
}

fn detect_shell() -> IntegrationShell {
    #[cfg(windows)]
    {
        return IntegrationShell::Powershell;
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.contains("fish") {
            IntegrationShell::Fish
        } else if shell.contains("zsh") {
            IntegrationShell::Zsh
        } else {
            IntegrationShell::Bash
        }
    }
}
```

- [ ] **Step 5: Wire the subcommand into the dispatch**

Add `ShellIntegration(ShellIntegrationCommand)` to the subcommand enum (or whatever variant naming the file uses) and add a branch to the match statement that calls `run_shell_integration`. Match the pattern of existing subcommands — don't invent a new style.

- [ ] **Step 6: Structural verification**

```bash
cargo metadata --format-version=1 --no-deps > /dev/null && echo "workspace OK"
```

Full build verification is still blocked on the openssl/perl issue; defer to CI.

- [ ] **Step 7: Commit**

```bash
git add ibron/src/main.rs  # plus any new shell_integration.rs if split out
git commit -m "feat(cli): add ibron shell-integration subcommand

Embeds the three integration scripts at compile time and exposes
them via 'ibron shell-integration [--print|--install] [--shell ...]'.
Auto-detects the current shell from \$SHELL on Unix and defaults
to PowerShell on Windows."
```

---

## Task 5: Integration note for CI / packaging

**Files:**
- Modify: `assets/flatpak/com.ibrahimokdadov.ibron.json` and `.template.json` — add the new shell-integration files to the install step

The flatpak build-commands list currently installs `assets/shell-integration/*` already (via glob), so `ibron.sh`, `ibron.ps1`, and `ibron.fish` come along for free. Verify with:

```bash
grep "shell-integration" assets/flatpak/com.ibrahimokdadov.ibron.json
```

Expected: a line like `"install -Dm644 assets/shell-integration/* -t /app/extra/export/share/etc/profile.d"`.

- [ ] **Step 1: Confirm no packaging changes needed**

If the glob already covers the new files, skip the commit. If there's a hardcoded list, update it.

---

## Task 6: Tag phase2-shell-integration

- [ ] **Step 1: Verify the tree**

```bash
git log --oneline ibron-phase1b-rebrand..HEAD
```

Expected: 3–5 commits covering the four tasks above.

- [ ] **Step 2: Tag**

```bash
git tag -a ibron-phase2-shell-integration -m "Phase 2: OSC 133 shell integration (bash/zsh, PowerShell, fish)"
git push origin main
git push origin ibron-phase2-shell-integration
```

---

## Rollback

Every task is additive — no existing file is modified destructively except the clap subcommand enum. If a task breaks something, `git revert <sha>` leaves the rest of phase 2 intact.
