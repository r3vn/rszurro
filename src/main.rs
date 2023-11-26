use clap::Parser;
use log::{info, LevelFilter};
use tokio::sync::mpsc;

use rszurro::{CacheManager, Cli, ConfigFile};

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
        serde_yaml::from_str::<ConfigFile>(&configuration).unwrap()
    };

    // init logger
    env_logger::Builder::new()
        .filter_level(match cli.verbose {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5.. => LevelFilter::Trace,
        })
        .init();

    // init channel
    let (tx, rx) = mpsc::channel(256);
    let mut handles = vec![];

    info!("starting \"cache_manager\"...");
    handles.push(tokio::spawn(async move {
        // start cache manager
        let cache_manager = CacheManager {
            enabled: !cli.nocache,
            endpoints: rszurro.endpoints,
        };

        cache_manager.run(rx).await;
    }));

    #[cfg(feature = "modbus-rtu")]
    // check if modbus_rtu monitor is enabled
    if rszurro.modbus_rtu.enabled {
        info!("starting \"modbus-rtu\" watcher...");

        // start modbus_rtu monitor
        let tx2 = tx.clone();

        handles.push(tokio::spawn(async move {
            rszurro.modbus_rtu.run(tx2).await.unwrap()
        }));
    }

    #[cfg(feature = "sysinfo")]
    // check if sysinfo monitor is enabled
    if rszurro.sysinfo.enabled {
        info!("starting \"syslog\" watcher...");

        // start sysinfou monitor
        let tx2 = tx.clone();

        handles.push(tokio::spawn(async move {
            rszurro.sysinfo.run(tx2).await.unwrap()
        }));
    }

    #[cfg(feature = "gpio")]
    // check if gpio monitor is enabled
    if rszurro.gpio.enabled {
        info!("starting \"gpio\" watcher...");

        // start gpio monitor
        let tx2 = tx.clone();

        handles.push(tokio::spawn(
            async move { rszurro.gpio.run(tx2).await.unwrap() },
        ));
    }

    #[cfg(feature = "lmsensors")]
    // check if lm_sensors monitor is enabled
    if rszurro.lm_sensors.enabled {
        info!("starting \"lm_sensors\" watcher...");

        // start lm_sensors monitor
        let tx2 = tx.clone();

        handles.push(tokio::task::spawn_blocking(move || {
            rszurro.lm_sensors.run(tx2).unwrap()
        }));
    }

    futures::future::join_all(handles).await;
}
