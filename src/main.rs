use clap::Parser;

use rszurro::{Cli, ConfigFile};

#[tokio::main()]
async fn main() {
    // parse cli arguments
    let cli = Cli::parse();

    // read configuration file
    let configuration = {
        let configuration = std::fs::read_to_string(&cli.config).unwrap();
        serde_json::from_str::<ConfigFile>(&configuration).unwrap()
    };

    let mut handles = vec![];

    if configuration.modbus_rtu.enabled {
        // check if modbus_rtu is enabled
        // start modbus_rtu monitor
        handles.push(tokio::spawn(async move {
            configuration
                .modbus_rtu
                .run(configuration.homeassistant.clone(), cli.verbose.clone())
                .await
                .unwrap()
        }));
    }

    futures::future::join_all(handles).await;
}
