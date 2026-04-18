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
