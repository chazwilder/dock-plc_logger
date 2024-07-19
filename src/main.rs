mod models;
mod controllers;
mod services;

use std::time::Duration;
use anyhow::Error;
use rayon::prelude::*;
use crate::controllers::load_config;
use crate::models::dock_door::DockSensor;
use log4rs;
use log::{error, warn, info, debug, trace};

fn main() -> Result<(), Box<Error>> {
    log4rs::init_file("C:\\Users\\cwilder\\RustroverProjects\\dock-door-service\\log4rs.yaml", Default::default()).unwrap();
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let sensors = runtime.block_on(load_config())?;
    match poll_sensors(sensors){
        Ok(_) => info!("Polling completed successfully"),
        Err(e) => error!("Error during polling: {}", e),
    }
    Ok(())
}


pub fn poll_sensors(mut sensors: Vec<DockSensor>) -> Result<(), anyhow::Error> {
    let poll_interval = Duration::from_secs(15); // Adjust as needed

    loop {
        sensors.par_iter_mut()
            .try_for_each(|sensor| -> Result<(), Error> {
                match sensor.poll() {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        error!("Error polling sensor: {:?}", e);
                        Ok(())
                    }
                }
            })?;

        std::thread::sleep(poll_interval);
    }
}