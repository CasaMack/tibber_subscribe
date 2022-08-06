use tibber_subscribe::v1::{
    run::{get_api_endpoint, get_db_info, get_home_id, get_logger, get_token, handle},
    tibber::{Field, SubscriptionQueryBuilder},
};

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
        .with(Field::LastMeterConsumption)
        .with(Field::AccumulatedConsumptionLastHour)
        .with(Field::MinPower)
        .with(Field::AveragePower)
        .with(Field::MaxPower)
        .build();

    loop {
        let res = handle(
            api_endpoint.clone(),
            db_addr.clone(),
            db_name.clone(),
            subscription_request.connection(),
            subscription_request.subscription(),
        )
        .await;
        match res {
            Ok(_) => {}
            Err(_) => {
                tracing::error!("Top level error")
            }
        }
    }
}
