# rszurro

A simple daemon written in [Rust](https://www.rust-lang.org) that read modbus registers from a serial bus and send their values to [Home Assistant](https://www.home-assistant.io) via REST API.

## build

```sh
cargo build --release
```

## Configuration

rszurro load settings from a json file, an example configuration can be found on [config.json](https://github.com/r3vn/rszurro/blob/main/config.json) from this repository.

## run

```sh
target/release/rszurro /path/to/config.json
```

## debian packaging

It is possible to build and install a debian package of rszurro, including a systemd service, using [cargo-deb](https://github.com/mmstick/cargo-deb) as follows:

```sh
cargo deb
sudo dpkg -i target/debian/rszurro_0.1.0_armhf.deb
```

Once installed the systemd service should be automatically started and can be checked via systemctl as shown below:

```
systemctl status rszurro
```
