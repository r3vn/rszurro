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

    // start configured watchers
    for watcher in rszurro.watchers {
        info!(
            "starting {} as {} watcher ",
            &watcher.name, &watcher.platform
        );
        let tx2 = tx.clone();

        handles.push(watcher.run(tx2).await);
    }

    futures::future::join_all(handles).await;
}
