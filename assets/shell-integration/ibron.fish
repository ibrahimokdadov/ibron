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
    # B is emitted AFTER the prompt would render; fish lacks a dedicated
    # post-prompt event, but wezterm / iTerm accept the A-then-B ordering
    # from within fish_prompt.
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
