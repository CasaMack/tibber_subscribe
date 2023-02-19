use std::{env, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use http;
use influxdb::{Client, InfluxDbWriteable};
use tracing::{instrument, metadata::LevelFilter, Level};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    fmt::format::{DefaultFields, FmtSpan, Format},
    FmtSubscriber,
};
use tungstenite;

#[instrument]
pub fn get_db_info() -> (Arc<String>, Arc<String>) {
    let db_addr = env::var("INFLUXDB_ADDR").expect("INFLUXDB_ADDR not set");
    tracing::info!("INFLUXDB_ADDR: {}", db_addr);

    let db_name = env::var("INFLUXDB_DB_NAME").expect("INFLUXDB_DB_NAME not set");
    tracing::info!("INFLUXDB_DB_NAME: {}", db_name);

    (Arc::new(db_addr), Arc::new(db_name))
}

#[instrument]
pub fn get_api_endpoint() -> Arc<String> {
    let api_endpoint = env::var("TIBBER_API_ENDPOINT").expect("TIBBER_API_ENDPOINT not set");
    tracing::info!("TIBBER_API_ENDPOINT: {}", api_endpoint);

    Arc::new(api_endpoint)
}

pub fn get_home_id() -> String {
    let api_endpoint = env::var("HOME_ID").expect("HOME_ID not set");
    tracing::info!("HOME_ID: {}", api_endpoint);

    api_endpoint
}

#[instrument]
pub fn get_token() -> Arc<String> {
    let token;
    let token_res = env::var("TIBBER_TOKEN");
    match token_res {
        Ok(t) => {
            token = t;
        }
        Err(_) => {
            tracing::info!("TIBBER_TOKEN not set");
            tracing::info!("Attempting to read from file");
            let token_file = env::var("TOKEN_FILE").ok();
            let token_file_str = token_file.as_ref().map(|s| s.as_str());
            let credentials = local_credentials::get_credentials(token_file_str);
            match credentials {
                Ok(c) => {
                    token = c.password;
                }
                Err(e) => {
                    tracing::error!("Failed to read credentials: {}", e);
                    panic!("TIBBER_TOKEN not set and fallback failed");
                }
            }
        }
    }

    Arc::new(token)
}

pub fn get_logger() -> (
    FmtSubscriber<DefaultFields, Format, LevelFilter, NonBlocking>,
    WorkerGuard,
) {
    let appender = tracing_appender::rolling::daily("./var/log", "tibber-status-server");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);

    let level = match env::var("LOG_LEVEL") {
        Ok(l) => match l.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        },
        Err(_) => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        //.with_span_events(FmtSpan::NONE)
        .with_ansi(false)
        .with_max_level(level)
        .with_writer(non_blocking_appender)
        // completes the builder.
        .finish();

    (subscriber, guard)
}

#[derive(InfluxDbWriteable)]
struct DBPriceInfo {
    pub time: DateTime<Utc>,
    #[influxdb(tag)]
    pub field_name: String,
    pub value: f64,
}

//TODO fails
#[instrument(skip(client), level = "trace")]
pub async fn write_to_db(client: &Client, field_name: String, value: f64, measurement: &str) {
    let variable = DBPriceInfo {
        time: Utc::now(),
        field_name,
        value,
    };

    let write_result = client.query(variable.into_query(measurement)).await;
    match write_result {
        Ok(_) => {
            tracing::trace!("Writing success");
        }
        Err(e) => {
            tracing::error!("Writing failed: {}", e);
        }
    }
}

#[instrument(skip_all)]
pub async fn handle(
    api_endpoint: Arc<String>,
    db_addr: Arc<String>,
    db_name: Arc<String>,
    connection_request: String,
    subscription_request: String,
) -> Result<(), ()> {
    tracing::debug!("Building request");
    let request = http::Request::builder()
        .uri(api_endpoint.as_ref())
        .method("GET")
.header("Host", "websocket-api.tibber.com")
.header("Accept", "*/*")
.header("Accept-Language", "en-US,en;q=0.5")
.header("Accept-Encoding", "gzip, deflate, br")
.header("Sec-WebSocket-Version", "13")
.header("Origin", "https://developer.tibber.com")
.header("Sec-WebSocket-Protocol", "graphql-transport-ws")
.header("Sec-WebSocket-Extensions", "permessage-deflate")
.header("Sec-WebSocket-Key", "iNwnisOA5oCUVnR3Gn1rUA==")
.header("DNT", "1")
.header("Connection", "keep-alive, Upgrade")
.header("Sec-Fetch-Dest", "websocket")
.header("Sec-Fetch-Mode", "websocket")
.header("Sec-Fetch-Site", "same-site")
.header("Pragma", "no-cache")
.header("Cache-Control", "no-cache")
.header("Upgrade", "websocket")
.header("User-Agent", "CasaMack/1.0")


        .body(())
        .unwrap();
    tracing::debug!("Request built");

    tracing::debug!("Establishing connection");
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

        let client = Client::new(db_addr.as_str(), db_name.as_str());

        socket
            .write_message(tungstenite::Message::Text(connection_request))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to request connection: {}", e);
            });
        socket
            .write_message(tungstenite::Message::Text(subscription_request))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to request subscription: {}", e);
            });
        tracing::info!("Subscribtion request sent");
        let sock_arc = std::sync::Arc::new(std::sync::Mutex::new(socket));
        loop {
            let sock_ref = sock_arc.clone();
            let resp = tokio::time::timeout(
                Duration::from_secs(10),
                tokio::task::spawn_blocking(move || sock_ref.lock().unwrap().read_message()),
            )
            .await;
            if let Err(_) = resp {
                return Err(());
            }
            let resp = resp.unwrap();
            if let Err(_) = resp {
                return Err(());
            }
            let resp = resp.unwrap();
            if let Err(_) = resp {
                return Err(());
            }
            match resp {
                Ok(msg) => {
                    println!("MSG: {:?}", msg);
                    let msg_json: serde_json::Value = serde_json::from_str(&msg.to_string())
                        .unwrap_or_else(|e| {
                            tracing::error!("Failed to parse message: {}", e);
                            serde_json::Value::Null
                        });
                    let data = msg_json["payload"]["data"]["liveMeasurement"].as_object();
                    if data.is_none() {
                        let payload_type = msg_json["type"].as_str().unwrap();
                        if payload_type == "connection_ack" {
                            tracing::info!("Subscription request acknowledged");
                        } else {
                            tracing::warn!("Anomalous response type: {}", payload_type);
                            tracing::debug!("Response: {:?}", msg_json);
                        }
                        continue;
                    }
                    let data = data.unwrap();
                    for key in data.keys() {
                        write_to_db(
                            &client,
                            key.to_string(),
                            data[key].as_f64().unwrap(),
                            "liveMeasurement",
                        )
                        .await;
                    }
                    tracing::trace!("Received: {}", msg);
                }
                Err(e) => {
                    tracing::error!("Error on read: {}", e);
                    return Err(());
                }
            }
        }
    // socket.close(None);
    } else {
        if let Err(e) = connection_result {
            tracing::error!("Error on connect: {}", e);
            println!("Error: {}", e);
            return Err(());
        }
    }

    Ok(())
}
