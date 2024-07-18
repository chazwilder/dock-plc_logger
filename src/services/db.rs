use sqlx_oldapi::mssql::{MssqlPoolOptions, Mssql};
use sqlx_oldapi::Pool;
use dotenvy::dotenv;
use std::env;
use chrono::Local;
use crate::models::dock_door::DockSensor;
use log::{error, warn, info, debug, trace};



pub async fn get_connection() -> Option<Pool<Mssql>> {
    dotenv().ok();
    let db_url = match env::var("MSSQL_URL") {
        Ok(url) => url,
        Err(e) => {
            error!("Failed to get MSSQL_URL from environment: {}", e);
            return None;
        }
    };

    match MssqlPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            info!("Successfully connected to database");
            Some(pool)
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            None
        }
    }
}


pub fn save_to_mssql(sensor: &DockSensor) -> Result<(), anyhow::Error> {
    dotenv().ok();
    let update_query = env::var("UPDATE_QUERY").expect("Failed to get UPDATE_QUERY from environment");
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let pool = get_connection().await.expect("Failed to get database connection");
        let res = sqlx_oldapi::query(
            &update_query
        )
            .bind(sensor.update_dttm)
            .bind(&sensor.dock_name)
            .bind(&sensor.sensor)
            .bind(sensor.current_value)
            .bind(sensor.previous_value)
            .bind(sensor.previous_dttm)
            .execute(&pool)
            .await
            .expect("Failed to insert sensor data into database");
        info!("{} | INFO: MSSQL -  {:?}", Local::now(), res);

        Ok(())
    })
}