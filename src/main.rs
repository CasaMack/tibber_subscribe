use influxdb::Client;
use tibber_status::v1::{run::{
    get_api_endpoint, get_db_info, get_logger, get_token, get_home_id, write_to_db,
}, tibber::{SubscriptionQueryBuilder, Field}};
use http;
use tungstenite;

#[tokio::main]
async fn main() {
    let (subscriber, _guard) = get_logger();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
    tracing::trace!("Log setup complete");

    let (db_addr, db_name) = get_db_info();
    let api_endpoint = get_api_endpoint();
    let auth = get_token();
    let home_id = get_home_id();


    let subscription_request = SubscriptionQueryBuilder::new(auth.clone().to_string(), home_id)
        .with(Field::Power)
        .build();


    let request = http::Request::builder()
        //.uri("wss://api.tibber.com/v1-beta/gql/subscriptions")
        .uri(api_endpoint.as_str())
        .header("Upgrade", "websocket")
        .header("Connection", "keep-alive,Upgrade")
        .header("Host", "api.tibber.com")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("Sec-WebSocket-Protocol", "graphql-ws")
        .body(())
        .unwrap();

    let connection_result = tungstenite::connect(request);
    let connection;

    if let Ok(c) = connection_result {
        connection = c;
        let mut socket = connection.0;
        let response = connection.1;
        tracing::info!("Connected to the server");
        tracing::info!("Response HTTP code: {}", response.status());
        tracing::info!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            tracing::info!("* {}", header);
        }

        let client = Client::new(db_addr.clone().as_str(), db_name.clone().as_str());

        socket.write_message(tungstenite::Message::Text(subscription_request.connection())).unwrap();
        socket.write_message(tungstenite::Message::Text(subscription_request.subscription())).unwrap();
        loop {
            let resp = socket.read_message();
            match resp {
                Ok(msg) => {
                    let msg_json: serde_json::Value = serde_json::from_str(&msg.to_string()).unwrap();
                    let data = msg_json["payload"]["data"]["liveMeasurement"].as_object().unwrap();
                    for key in data.keys() {
                        write_to_db(&client, key.to_string(), data[key].as_f64().unwrap(), "liveMeasurement").await;
                    }
                    tracing::trace!("[{}] Received: {}", chrono::Local::now(), msg);
                }
                Err(e) => {
                    tracing::error!("[{}] Error on read: {}", chrono::Local::now(), e);
                    break;
                }
            }
        }
    // socket.close(None);
    } else {
        if let Err(e) = connection_result {
            println!("Error: {}", e);
            return;
        }
    }
}
