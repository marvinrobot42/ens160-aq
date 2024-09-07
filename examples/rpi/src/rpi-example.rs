use anyhow::Result;
use linux_embedded_hal::{Delay, I2cdev};

use ens160_aq::data::{AirQualityIndex, Measurements as Measurements_aq};
use ens160_aq::Ens160;
use ens160_aq::error::Error;

use std::thread;
use std::time::Duration;

use env_logger::Builder;
use log::{LevelFilter, error, info};
use std::io::Write;

fn main() -> Result<()> {

    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();


    info!("Hello, linux Rust and ENS160-AQ world!");

    let dev_i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let delayer = Delay {};

    let mut ens160 = Ens160::new_secondary_address(dev_i2c, delayer);
    ens160.set_temp_rh_comp(21.5, 41).unwrap();
    let ens160_result = ens160.initialize();
    match ens160_result {
        Ok(what) => info!("ENS160 initialized ok: {}", what),
        Err(err) => error!("ENS160 initialize error: {:?}", err),
    }
    let _delay: Delay = Delay {};

    loop {
        info!("looping");
        thread::sleep(Duration::from_secs(10));
        // Air quality
        if let Ok(status) = ens160.get_status() {
            info!("ens160 status is {:#?}", status);
            if status.new_data_ready() {
                let measuremnts_aq: Measurements_aq = ens160.get_measurements().unwrap();
                info!("AQ nmeasurements are : {:#?}", measuremnts_aq);
            }
        }

    
    }

}
