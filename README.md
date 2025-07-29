# Replay:  save and replay sequences of shell commands

Replay is a lightweight CLI tool to record, replay, and manage shell command sessions. Ideal for automation, reproducibility, and quick demonstrations.

## Installation

```sh
cargo install replay
```
## Usage 
```sh
replay <COMMAND> [OPTIONS]
```

### Available Commands

```sh
replay run [OPTIONS] [SESSION_NAME]
```

Run a replay on the specified session.

#### Arguments
- `SESSION_NAME`: The name of the session to replay. If omitted, the last session will be used.

#### Options
- `-s`, `--show`: Show the commands without executing them.
- `-d`, `--delay <ms>`: Delay (in milliseconds) between commands. Default to `0`. 
- `-h`, `--help`: Show help for this commands.

---

```sh
replay record [SESSION_NAME]
```
Record a new session of shell commands. If a session name is provided, it will be used to label the recording.

--- 

```sh
replay help
```
Show help for the CLI or a specific subcommand.

### Global Options
- `-h`, `--help`: show general help.
- `-V`, `--version`: show version information.

## Git Hook

To enable a pre-configured Git hook that automatically formats your code before each commit:
```sh
git config core.hooksPath .githooks
```
> Run these commands once after cloning the repository to ensure the hooks are active.
