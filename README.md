# Replay:  save and replay sequences of shell commands

Replay is a lightweight CLI tool to record, replay, and manage shell command sessions. Ideal for automation, reproducibility, and quick demonstrations.

<p align="center">
  <img src="demo.gif" alt="animated" width="80%" />
</p>

## Installation

```sh
cargo install replay_pty
```

## Usage 
💡 Run `replay help` to see all available commands.  

### Record a Session
```sh
replay record
```
Use `replay record -h` to see all the options available for this command

### Replay a Session
To run a recorded session of commands : 
```sh
replay run # runs the last recorded session by default
```
Use `replay run -h` to see all the options available for this command

## License
Replay is licenced under MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)


## Contributing

Contributions are very welcome!
Please see our [contributing guide](./CONTRIBUTING.md) for details.

Thanks to all the people who already contributed!

<a href="https://github.com/ariel-os/ariel-os/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=baptleduc/replay" alt="All contributors" />
</a>
