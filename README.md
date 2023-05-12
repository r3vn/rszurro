# rszurro

**'rszurro'** is a simple daemon written in [Rust](https://www.rust-lang.org) that monitors Modbus registers from a serial bus and send their values to [Home Assistant](https://www.home-assistant.io) via REST API whenever they change.

It is mainly designed to integrate serial Modbus devices that are physically unreachable from your Home Assistant server.

## Building

To build rszurro, ensure Rust is installed, clone the repository, navigate to the project directory, and run:

```sh
cargo build --release
```

This will produce a single executable file in the target/release directory.

## Configuration

rszurro load settings from a json file, an example configuration can be found on [config.json](https://github.com/r3vn/rszurro/blob/main/config.json) from this repository.

## Usage

To use rszurro, you need to provide the path of your configuration file, as follows:

```sh
target/release/rszurro /path/to/config.json
```

Once running, rszurro will continuously monitor the specified Modbus registers for changes and update the corresponding sensors in Home Assistant.

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
