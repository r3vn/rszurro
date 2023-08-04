use clap::Parser;

use rszurro::{Cli, ConfigFile};

#[tokio::main()]
async fn main() {
    // parse cli arguments
    let cli = Cli::parse();

    // read configuration file
    let rszurro = {
        let configuration = std::fs::read_to_string(&cli.config).unwrap();
        serde_json::from_str::<ConfigFile>(&configuration).unwrap()
    };

    let mut handles = vec![];

    // check if modbus_rtu monitor is enabled
    if rszurro.modbus_rtu.enabled {
        // start modbus_rtu monitor
        let ha = rszurro.homeassistant.clone();

        handles.push(tokio::spawn(async move {
            rszurro
                .modbus_rtu
                .run(ha, cli.verbose)
                .await
                .unwrap()
        }));
    }

    // check if lm_sensors monitor is enabled
    if rszurro.lm_sensors.enabled {
        // start lm_sensors monitor
        let ha = rszurro.homeassistant.clone();

        handles.push(tokio::task::spawn_blocking(move || {
            rszurro.lm_sensors.run(ha, cli.verbose).unwrap()
        }));
    }

    futures::future::join_all(handles).await;
}
