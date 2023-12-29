Rust ENS160-AQ Crate
====

A Rust crate for ScioSense ENS160 air quality sensor 

<https://github.com/marvinrobot42/ens160-aq.git>

[ENS160]: https://www.sciosense.com/wp-content/uploads/documents/SC-001224-DS-9-ENS160-Datasheet.pdf

Features

- designed for embedded use (ESP32-C3 and -S3 and STM32F3DISCOVERY)
- configurable interrupt pin
- supports both 0x52 (default) and 0x53 (secondary) I2C device addresses
- set temperature and humidity for ENS160 compensation calulation
- reads air quality index, eCO2, TVOC, ethanol concentration and raw hot plate resistance (in ohms)
- an easy to use Measurements struct
- an easy to use initialize function
- no_std embedded compatible

- (SPI not supported, yet)
  

Notes

This is my first device driver project.  It was inspired by Alexander Hübener excellent ENS160 crate.
I was not able to get that ENS160 crate working in my ESP32-C3 (unknown reason) so I created my own driver as a learning exercise.


Usage
----

Add the dependency to `Cargo.toml`.

~~~~toml
[dependencies.ens160-aq]
version = "0.1.0"
~~~~

A `Ens160` structure is created from an I²C interface and a delay function.
Configure interrupt pin properties if required.  
Initialize ENS160 (required delays are included).
set_temp_rh_comp() can be called anytime for temperature and humidity componesation.
Read the ENS160 status and check if new data or group data (if needed) is ready
then get_measurements().
Note that set_operation_mode(OperationMode::Reset) is available but it will put the ENS160
back to factory defaults including the 24 hour "burn-in" mode.  It does not need to be called
for any other reason.



~~~~rust

// ESP32-C3 style example

use ens160_aq::data::{InterruptPinConfig, Measurements, OperationMode};
use ens160_aq::Ens160;
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};


let peripherals = Peripherals::take().unwrap();

let pins = peripherals.pins;
let sda = pins.gpio0;
let scl = pins.gpio1;
let i2c = peripherals.i2c0;
let config = I2cConfig::new().baudrate(100.kHz().into());  // should not at 400 kHz
let i2c_dev = I2cDriver::new(i2c, sda, scl, &config)?;

let ens160_int = PinDriver::input(pins.gpio6).unwrap();  // connect to ENS160 INT pin if using it

let mut ens160 = Ens160::new(i2c_dev, Ets {});  // Ets is ESP32 IDF delay function


// optional: configure ENS160 interrupt pin for on new data active low, push_pull drive mode
let int_pin_config: InterruptPinConfig = InterruptPinConfig::builder()
  .active_low()
  .push_pull()
  .not_new_group_data()
  .on_new_data()
  .enable_interrupt();
ens160.config_interrupt_pin(int_pin_config.get_value()).unwrap();

ens160.initialize().unwrap();

// optional: usually not required
ens160.set_temp_rh_comp(21.5, 41).unwrap();  
let (temp_c, rh) = ens160.get_temp_rh_comp().unwrap();
info!("compensation set to temperature = {} C, relative humidity = {} %", temp_c, rh);

loop {
  if let Ok(status) = ens160.get_status() {
    info!("ens160 status is {:#?}", status);
    if status.new_data_ready() {  // read all measurements
      let measuremnts: Measurements = ens160.get_measurement().unwrap();
      info!("measurements are : {:#?}\n\n", measuremnts);      
    }
    if status.new_group_data_ready() {
      let group_data: [u8; 8] = ens160.get_group_data().unwrap();
      info!("group data = {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}",
                      group_data[0], group_data[1], group_data[2], group_data[3],
                      group_data[4], group_data[5], group_data[6], group_data[7]
          );
   }
   if !status.new_data_ready() && !status.new_group_data_ready() {
     info!("no new data or group data ready");
   }
  }
  FreeRtos::delay_ms(30000);
}
    
~~~~


License
----

You are free to copy, modify, and distribute this application with attribution under the terms of either

 * Apache License, Version 2.0
   ([LICENSE-Apache-2.0](./LICENSE-Apache-2.0) or <https://opensource.org/licenses/Apache-2.0>)
 * MIT license
   ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

This project is not affiliated with nor endorsed in any way by ScioSense.
