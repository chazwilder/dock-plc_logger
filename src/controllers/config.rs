use crate::models::dock_door::{DockSensor, DockSensorPartial};
use crate::services::get_connection;
use dotenvy::dotenv;
use std::env;
use sqlx_oldapi;


pub async fn load_config() -> Result<Vec<DockSensor>, anyhow::Error> {
    dotenv().ok();
    let pool = match get_connection().await {
        Some(pool) => pool,
        None => {
            println!("No Connection To Database");
            return Err(anyhow::anyhow!("No Connection To Database"));
        }
    };
    let sql = env::var("PLC_CONFIG_QUERY").expect("PLC_CONFIG_QUERY not set in.env");
    let partial_sensors: Vec<DockSensorPartial> = sqlx_oldapi::query_as(&sql)
        .fetch_all(&pool)
        .await?;

    let sensors: Vec<DockSensor> = partial_sensors.into_iter()
        .map(DockSensor::from)
        .collect();

    Ok(sensors)
}