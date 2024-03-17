// ESP32-C3 style example

use anyhow::Result;
use ens160_aq::data::{InterruptPinConfig, Measurements};
use ens160_aq::Ens160;
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys as _;
use log::*;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let sda = pins.gpio0;
    let scl = pins.gpio1;
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(100.kHz().into()); // should work at 400 kHz also
    let i2c_dev = I2cDriver::new(i2c, sda, scl, &config)?;

    let ens160_int = PinDriver::input(pins.gpio6).unwrap(); // connect to ENS160 INT pin if using it
                                                            // write an interrupt service function for the above pin (beyond the scope of this example)

    let mut ens160 = Ens160::new(i2c_dev, Ets {}); // Ets is ESP32 IDF delay function

    // optional: configure ENS160 interrupt pin for on new data active low, push_pull drive mode
    // see data sheet section 10.9
    let int_pin_config: u8 = InterruptPinConfig::builder()
        .active_low()
        .push_pull()
        .not_new_group_data()
        .on_new_data()
        .enable_interrupt()
        .build();  // same as get_value() method
    info!("int pin config is {:#04x}", int_pin_config);  
    if let Ok(new_config_int) = ens160.config_interrupt_pin(int_pin_config) {
            if (new_config_int == int_pin_config) {
                info!("config_interrrupt_pin was good, new value is {:#04x}", new_config_int);
            } else {
                log::error!("config_interrupt_pin() not good, expected {:#04x}, got {:#04x}", int_pin_config, new_config_int);
            }
        }

    ens160.initialize().unwrap();

    // optional: usually not required
    ens160.set_temp_rh_comp(21.5, 41).unwrap();
    let (temp_c, rh) = ens160.get_temp_rh_comp().unwrap();
    info!(
        "compensation set to temperature = {} C, relative humidity = {} %",
        temp_c, rh
    );

    loop {
        if let Ok(status) = ens160.get_status() {
            info!("ens160 status is {:#?}", status);
            if status.new_data_ready() {
                // read all measurements
                let measuremnts: Measurements = ens160.get_measurement().unwrap();
                info!("measurements are : {:#?}\n\n", measuremnts);
            }
            if status.new_group_data_ready() {
                let group_data: [u8; 8] = ens160.get_group_data().unwrap();
                info!(
                    "group data = {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}",
                    group_data[0],
                    group_data[1],
                    group_data[2],
                    group_data[3],
                    group_data[4],
                    group_data[5],
                    group_data[6],
                    group_data[7]
                );
            }
            if !status.new_data_ready() && !status.new_group_data_ready() {
                info!("no new data or group data ready");
            }
        }
        FreeRtos::delay_ms(30000);
    }
}
