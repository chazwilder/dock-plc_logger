mod models;
mod controllers;
mod services;

use std::time::Duration;
use anyhow::Error;
use rayon::prelude::*;
use crate::controllers::load_config;
use crate::models::dock_door::DockSensor;


fn main() -> Result<(), Box<Error>> {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let mut sensors = runtime.block_on(load_config())?;
    poll_sensors(&mut sensors)?;

    Ok(())
}


pub fn poll_sensors(sensors: &mut [DockSensor]) -> Result<Vec<DockSensor>, anyhow::Error> {
    let poll_interval = Duration::from_secs(10); // Adjust as needed

    loop {
        sensors.par_iter_mut()
            .try_for_each(|sensor| -> Result<(), Error> {
                match sensor.poll() {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => {
                        eprintln!("Error polling sensor: {:?}", e);
                        Ok(())
                    }
                }
            })?;

        std::thread::sleep(poll_interval);
    }
}