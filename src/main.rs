use clap::Parser;

use rszurro::{Cli, ConfigFile};

#[tokio::main()]
async fn main() {
    // parse cli arguments
    let cli = Cli::parse();

    // print version
    if cli.verbose > 0 {
        println!("- rszurro v{} -", env!("CARGO_PKG_VERSION"));
    }

    // read configuration file
    let rszurro = {
        let configuration = std::fs::read_to_string(&cli.config).unwrap();
        serde_json::from_str::<ConfigFile>(&configuration).unwrap()
    };

    let mut handles = vec![];

    #[cfg(feature = "modbus-rtu")]
    // check if modbus_rtu monitor is enabled
    if rszurro.modbus_rtu.enabled {
        if cli.verbose > 0 {
            println!("[modbus_rtu] starting...");
        }

        // start modbus_rtu monitor
        let endpoints = rszurro.endpoints.clone();

        handles.push(tokio::spawn(async move {
            rszurro
                .modbus_rtu
                .run(endpoints, cli.verbose)
                .await
                .unwrap()
        }));
    }

    #[cfg(feature = "sysinfo")]
    // check if sysinfo monitor is enabled
    if rszurro.sysinfo.enabled {
        if cli.verbose > 0 {
            println!("[sysinfo] starting...");
        }

        // start sysinfou monitor
        let endpoints = rszurro.endpoints.clone();

        handles.push(tokio::spawn(async move {
            rszurro.sysinfo.run(endpoints, cli.verbose).await.unwrap()
        }));
    }

    #[cfg(feature = "gpio")]
    // check if gpio monitor is enabled
    if rszurro.gpio.enabled {
        if cli.verbose > 0 {
            println!("[gpio] starting...");
        }

        // start gpio monitor
        let endpoints = rszurro.endpoints.clone();

        handles.push(tokio::spawn(async move {
            rszurro.gpio.run(endpoints, cli.verbose).await.unwrap()
        }));
    }

    #[cfg(feature = "lmsensors")]
    // check if lm_sensors monitor is enabled
    if rszurro.lm_sensors.enabled {
        if cli.verbose > 0 {
            println!("[lm_sensors] starting...");
        }

        // start lm_sensors monitor
        let endpoints = rszurro.endpoints.clone();

        handles.push(tokio::task::spawn_blocking(move || {
            rszurro.lm_sensors.run(endpoints, cli.verbose).unwrap()
        }));
    }

    futures::future::join_all(handles).await;
}
