use log::{error, trace};
use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::{read_file, Client, Endpoint, SensorUpdate};

pub async fn get_client(endpoint: Endpoint, state: Arc<Mutex<bool>>) -> Client {
    // connect to mqtt broker
    let mut mqttoptions = MqttOptions::new(&endpoint.name, &endpoint.host, endpoint.port);
    let max_packet_size = 10 * 1024;

    // set mqtt options
    mqttoptions
        .set_max_packet_size(max_packet_size, max_packet_size)
        .set_keep_alive(Duration::from_secs(endpoint.keepalive))
        .set_request_channel_capacity(10)
        .set_credentials(endpoint.username, endpoint.password);

    // check TLS client auth and custom ca settings
    if !endpoint.client_crt.is_empty() || !endpoint.ca.is_empty() {
        mqttoptions.set_transport(
            get_tls_transport(&endpoint.ca, &endpoint.client_crt, &endpoint.client_key).await,
        );
    }

    // get client and eventloop
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    tokio::spawn(async move {
        // handle coinnection's eventloop
        while let Ok(notification) = eventloop.poll().await {
            trace!("Got notification: {:?}", &notification);
        }

        // set connection state to false
        let mut connection = state.lock().await;
        *connection = false;

        error!("Connection to {} lost.", &endpoint.name);
    });

    Client::MqttClient(client)
}

pub async fn send(endpoint: Endpoint, update: SensorUpdate, client: Client) -> bool {
    // get sensor update data
    let post_data = match endpoint.raw {
        // raw = true, send raw sensor value
        true => update.get_json().await["state"]
            .to_string()
            .replace('"', ""),
        // raw = false send json sensor value
        false => update.get_json().await.to_string(),
    };

    // set mqtt topic, true without prefix.
    let topic = match endpoint.prefix.is_empty() {
        true => format!("{}/{}", &update.device_name, &update.sensor.name),
        false => format!(
            "{}/{}/{}",
            &endpoint.prefix, &update.device_name, &update.sensor.name
        ),
    };

    // spawn publish request
    match client {
        Client::MqttClient(client) => client
            .publish(&topic, QoS::AtLeastOnce, true, post_data)
            .await
            .is_ok(), // return a bool
        _ => false,
    }
}

async fn get_tls_transport(
    ca: &String,
    client_crt: &String,
    client_key: &String,
) -> rumqttc::Transport {
    // Get a Tls Transport
    rumqttc::Transport::Tls(TlsConfiguration::Simple {
        ca: match ca.is_empty() {
            true => vec![],
            false => read_file(ca).await,
        },
        alpn: None,
        client_auth: match client_crt.is_empty() {
            true => None,
            false => Some((read_file(client_crt).await, read_file(client_key).await)),
        },
    })
}
