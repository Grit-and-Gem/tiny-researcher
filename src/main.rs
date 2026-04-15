mod config;
mod doctor;
mod runtime;
mod telemetry;
mod tools;

use std::error::Error;

use crate::{config::AppConfig, runtime::validator::validate_startup};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(String::as_str);
    let config_path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("configs/offline.toml");

    let cfg = AppConfig::from_toml_file(config_path)?;
    validate_startup(&cfg)?;

    if matches!(command, Some("doctor")) {
        doctor::run_doctor(&cfg)?;
        println!("research doctor: OK");
    } else {
        println!("runtime startup validation: OK");
    }

    Ok(())
}
