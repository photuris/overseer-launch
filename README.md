# overseer-launch

`overseer` starts an [overseer](https://github.com/photuris/overseer)
session from a named profile. A profile is one TOML table naming the
command for each role: the overseer itself, the implementer, and
optionally the reviewer and judge. Launching a profile exports the
roster variables the skill reads (`OVERSEER_IMPLEMENTER`,
`OVERSEER_REVIEWER`, `OVERSEER_JUDGE`, and `HERDR_OVERSEER=1` when
inside Herdr, TUI or desktop app) and replaces itself with the
overseer command in the current pane.

It does one thing. It does not create workspaces, tabs, or tmux
sessions: run it inside the pane the overseer should own, the same way
you would run `claude` or `codex` there.

## Install

**Linux and macOS:**

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/photuris/overseer-launch/releases/latest/download/overseer-launch-installer.sh | sh
```

**Windows** (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/photuris/overseer-launch/releases/latest/download/overseer-launch-installer.ps1 | iex"
```

Both put `overseer` in `~/.local/bin`. With a Rust toolchain:
`cargo install --git https://github.com/photuris/overseer-launch --locked`.

## Quickstart

```sh
overseer init                      # writes ~/.config/overseer/profiles.toml
$EDITOR ~/.config/overseer/profiles.toml
overseer claude                    # launch the "claude" profile here
overseer                           # launch the file's default
overseer open --print | jq         # show what would launch, without launching
```

## Profile file

`$XDG_CONFIG_HOME/overseer/profiles.toml`, falling back to
`~/.config/overseer/profiles.toml` (`%APPDATA%\overseer\profiles.toml`
on Windows). `--config PATH` reads another file.

```toml
default = "claude"

[profiles.claude]
overseer    = "claude"
implementer = "claude --model claude-sonnet-5-5"
reviewer    = "codex -c model_reasoning_effort=medium --model gpt-6-astra"
judge       = "overseer-judge"
```

- `overseer` and `implementer` are required. `reviewer` and `judge`
  are optional; leaving one out means that role does not exist, and
  any value inherited from the parent shell is cleared.
- Values are full commands. Each runs under `sh -c` (`cmd /C` on
  Windows), a non-interactive shell, so aliases from your shell rc are
  not visible. Pin a model in every command; an unpinned implementer
  that defaults to an expensive model defeats the point.
- `HERDR_OVERSEER=1` is derived from `HERDR_ENV=1` (Herdr TUI) or a
  non-empty `HC_TERMINAL_ID` (set by the Herdr desktop app's core). Do
  not put it in a profile. tmux has no such signal.

[`profiles.example.toml`](profiles.example.toml) is what `init`
writes.

## Loading the skill at startup

By default the overseer agent loads the skill on its own when the
first implementation task arrives. To load it at launch instead, give
the agent a starting prompt. `claude`, `codex`, and `pi` all take one
as a trailing argument, and the `overseer` value is a whole command
line, so it goes in the profile:

```toml
[profiles.claude]
overseer = "claude 'Load the overseer skill and confirm the roster.'"

[profiles.codex]
overseer = "codex --model gpt-6-astra 'Load the overseer skill and confirm the roster.'"

[profiles.open]
overseer = "pi --model code/kimi-k3 'Load the overseer skill and confirm the roster.'"
```

Claude Code also takes the skill's slash command as that prompt,
`overseer = "claude '/overseer'"`. If your version shows it as plain
text instead of running it, use the sentence.

What this buys and costs:

- The skill is always loaded, and the roster check runs before you
  type a task, so a broken profile shows up at once.
- The agent takes a paid turn at startup, and the skill stays in
  context even in a session where you only ask questions.

A lighter option on `claude` and `pi` is
`--append-system-prompt 'Overseer mode is on. Load the overseer skill before any implementation task.'`
It costs no startup turn, but it is a nudge, not a guarantee.

There is no profile key for this, on purpose. Each agent invokes
skills its own way, and the command string already says it. On
Windows the value runs under `cmd /C`, where single quotes do not
quote; use `\"` inside the TOML string there.

## Output and exit codes

stdout carries JSON (`--pretty` indents it). On failure the last
stderr line is `{"error":{"type":…,"message":…}}`.

| Code | Meaning |
|------|---------|
| 0 | ok |
| 1 | unexpected, including `init` over an existing file |
| 2 | usage: bad flags, invalid profile file, no name and no `default` |
| 4 | not found: no profile file, or unknown profile (message lists the known names) |

After a successful launch the exit code is the overseer's own.

## License

MIT, see [LICENSE](LICENSE).
