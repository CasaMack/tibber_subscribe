use std::{env, sync::Arc};

use chrono::{DateTime, Utc};
use influxdb::{Client, InfluxDbWriteable};
use tracing::{instrument, metadata::LevelFilter, Level};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    fmt::format::{DefaultFields, FmtSpan, Format},
    FmtSubscriber,
};

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
        .with_span_events(FmtSpan::NONE)
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
    pub field: String,
    pub value: f64,
}

#[instrument(skip(client), level = "trace")]
pub async fn write_to_db(client: &Client, field: String, value: f64, measurement: &str) {
    let variable = DBPriceInfo {
        time: Utc::now(),
        field,
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


