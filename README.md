# rszurro

**rszurro** is an IoT-focused daemon written in [Rust](https://www.rust-lang.org) that monitors sensors from various sources and distributes data to different endpoints.

It is formerly designed to integrate raw sensors that are physically unreachable from an [Home Assistant](https://www.home-assistant.io) server.

## Building

To build rszurro, ensure Rust is installed, also the lm_sensors monitor requires sensors.h provided on debian by the **libsensors-dev** package.

Clone the repository, navigate to the project directory, and run:

```sh
cargo build --release
```

This will produce a single executable file in the target/release directory.

## Configuration

rszurro load settings from a yaml file, an example configuration can be found on [config.yaml](https://github.com/r3vn/rszurro/blob/main/config.yaml) from this repository.

## Usage

To use rszurro, you need to provide the path of your configuration file, as follows:

```sh
target/release/rszurro /path/to/config.yaml
```

Once operational, rszurro will consistently monitor the designated sensors data for any alterations, promptly transmitting their updated information to the configured endpoints.


You can see all available command line options with:

```
rszurro --help
```

## Debian packaging

It is possible to build and install a debian package of rszurro, including a systemd service, using [cargo-deb](https://github.com/mmstick/cargo-deb) as follows:

```sh
cargo deb
sudo dpkg -i target/debian/rszurro_0.1.0_armhf.deb
```

Once installed the systemd service should be automatically started and can be checked via systemctl as shown below:

```
systemctl status rszurro
```
