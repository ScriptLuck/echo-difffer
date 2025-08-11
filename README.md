# echo-difffer
The goal of this project is to create a software that would customly modify an output sound of a device. \
This software adds around 1ms to sound latency.

## Support and libraries (Crates) used
Currently Supports only devices with `PulseAudio` sound server. \
This is an old sound system, currently being replaced by `Pipewire`. \
`Pipewire` is backward compatable with `PulseAudio` so this software can be used by most of the systems. \
Actually tested only on Ubuntu 24.04 - works good :)

#### Crates used
[`libpulse_binding`](https://crates.io/crates/libpulse-binding) - A Rust language binding for the PulseAudio `libpulse` library \
[`libpulse_simple_binding`](https://crates.io/crates/libpulse_simple_binding) - A Rust language binding for the PulseAudio `libpulse-simple` library

## How to build
Use Rust build system - `Cargo`
- Debug mode > `cargo build`
- Release mode > `cargo build --release`
- For other modes please check `Cargo` documentation

## How to run
Use Rust build system - `Cargo`
- Debug mode > `cargo run`
- Release mode > `cargo run --release`
- For other modes please check `Cargo` documentation

### Command-line arguments
This program allowes for up to 3 command-line arguments as follow:
1) `target_volume_lvl` \
A number between 0.0 and 1.0. It is expected average volume loudness level. \
0.0 - silence, ~0.1 - real-life conversations, ~0.2 - music, ~0.3 - gaming, 1.0 - super very loud. \
It is recommended to keep it below 0.4. `{Default value - 0.16}`
2) `gain_factor` \
A number between 0.0 and 1.0. This attribute is the speed of volume change. \
To reduce sound spikes due to dynamic volume changes, the volume changes dynamically too. \
0.0 - no changes would be made, 1.0 - instant changes. \
Recommended to keep below 0.1. `{Default value - 0.02}`
3) `clamp_max` \
A number from 1.0 and above. It is the maximum volume multiplier. \
It sets the allowed volume range from `1/clamp_max`x to `clamp_max`x of the original volume. \
Recommended to set between 2.0 and 10.0. `{Default value - 2.0}`

#### Usage of command-line arguments
Use > `run <target_volume_lvl> <gain_factor> <clamp_max>` \
For example > `cargo run -- 0.16 0.02 4.0` (`--` is optional but recommended when using `cargo`) \
Another example > `./echo-difffer 0.2`. \
Last example provided only `target_volume_lvl` argument. \
Other arguments would use their default values. \
Arguments are NOT required but allows for custom settings :)
