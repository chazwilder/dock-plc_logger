use chrono::Local;
use serde::{Deserialize, Serialize};
use derive_more::Constructor;
use sqlx_oldapi::FromRow;
use plctag::builder::*;
use plctag::RawTag;
use plctag::Status;
use crate::services::db::save_to_mssql;
use log::{error, warn, info, debug, trace};

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DockSensorPartial {
    #[sqlx(rename = "DOCK_NAME")]
    pub dock_name: String,
    #[sqlx(rename = "DOCK_IP")]
    pub dock_ip: String,
    #[sqlx(rename = "SENSOR")]
    pub sensor: String,
    #[sqlx(rename = "ADDRESS")]
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Constructor, FromRow, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DockSensor {
    #[sqlx(rename = "DOCK_NAME")]
    pub dock_name: String,
    #[sqlx(rename = "DOCK_IP")]
    pub dock_ip: String,
    #[sqlx(rename = "SENSOR")]
    pub sensor: String,
    #[sqlx(rename = "ADDRESS")]
    pub address: String,
    #[sqlx(rename = "UPDATE_DTTM")]
    pub update_dttm: Option<chrono::NaiveDateTime>,
    #[sqlx(rename = "CURRENT_VALUE")]
    pub current_value: Option<u8>,
    #[sqlx(rename = "PREVIOUS_DTTM")]
    pub previous_dttm: Option<chrono::NaiveDateTime>,
    #[sqlx(rename = "PREVIOUS_VALUE")]
    pub previous_value: Option<u8>,
    #[sqlx(rename = "DOOR_STATE")]
    pub door_state: Option<String>,
    #[sqlx(rename = "PANEL_STATE")]
    pub panel_state: Option<String>,
    pub initialized: bool,
}

impl Default for DockSensor {
    fn default() -> Self {
        DockSensor {
            dock_name: String::new(),
            dock_ip: String::new(),
            sensor: String::new(),
            address: String::new(),
            update_dttm: None,
            current_value: None,
            previous_dttm: None,
            previous_value: None,
            door_state: None,
            panel_state: None,
            initialized: false,
        }
    }
}

impl DockSensor {
    pub fn poll(&mut self) -> Result<(), anyhow::Error> {
        let path = PathBuilder::default()
            .protocol(Protocol::EIP)
            .gateway(&self.dock_ip)
            .plc(PlcKind::MicroLogix)
            .name(&self.address)
            .element_size(1)
            .element_count(1)
            .path("0")
            .read_cache_ms(0)
            .build()?;

        let tag = RawTag::new(path, 1000)
        .map_err(|e| anyhow::anyhow!("Failed to create tag: {:?}", e))?;
        let status = tag.read(1000);


        match status {
            Status::Ok => {
                let new_value = tag.get_u8(0)
                    .expect("Failed to get value");

                if !self.initialized {
                    self.initialize(new_value)?;
                } else if self.current_value.map_or(true, |current| current != new_value) {
                    self.update_value(new_value)?;
                }
                ///////////////////////////////////////////////////////
            }
            _ => {
                error!("Failed to read sensor {} on dock {}: {:?}", self.sensor, self.dock_name, status);
            }
        }
        Ok(())
    }
    pub fn initialize(&mut self, value: u8) -> Result<(), anyhow::Error> {
        let now = Local::now().naive_local();
        self.current_value = Some(value);
        self.update_dttm = Some(now);
        self.initialized = true;
        info!("{} | INFO: Sensor {} on dock {} initialized with value: {:?}", now, self.sensor, self.dock_name, value);
        save_to_mssql(self)?;
        Ok(())
    }
    pub fn update_value(&mut self, new_value: u8) -> Result<(), anyhow::Error> {
        let now = Local::now().naive_local();
        self.previous_value = self.current_value;
        self.current_value = Some(new_value);
        self.previous_dttm = self.update_dttm;
        self.update_dttm = Some(now);
        info!("{} | INFO: Sensor {} on dock {} updated: {:?}", now, self.sensor, self.dock_name, new_value);
        save_to_mssql(self)?;
        info!("{} | INFO: Sensor {} on dock {} update saved to MSSQL", now, self.sensor, self.dock_name);
        Ok(())
    }
}

impl From<DockSensorPartial> for DockSensor {
    fn from(partial: DockSensorPartial) -> Self {
        DockSensor {
            dock_name: partial.dock_name,
            dock_ip: partial.dock_ip,
            sensor: partial.sensor,
            address: partial.address,
            initialized: false,
            ..Default::default()
        }
    }
}

