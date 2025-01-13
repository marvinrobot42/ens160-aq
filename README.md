# ENS160-AQ &emsp; 
[![crates.io](https://img.shields.io/crates/v/ens160-aq)](https://crates.io/crates/ens160-aq)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/marvinrobot42/ens160-aq.git)
[![Documentation](https://docs.rs/ens160-aq/badge.svg)](https://docs.rs/ens160-aq)

## A Rust crate for ScioSense ENS160 air quality sensor 

<https://github.com/marvinrobot42/ens160-aq.git>

[ENS160-AQ]: https://www.sciosense.com/wp-content/uploads/documents/SC-001224-DS-9-ENS160-Datasheet.pdf

A benefit of using the ENS160 is the combined air quality index is calculated within the device, no external proprietary calculation programme is required. 

### Features

- updated to use embedded-hal version 1.0.x
- includes async support as a feature(s)
- designed for embedded use (ESP32-C3, -C6 and -S3) and Raspberry Pi
- configurable interrupt pin
- supports both 0x52 (default) and 0x53 (secondary) I2C device addresses
- set temperature and humidity for ENS160 compensation calulation
- reads air quality index, eCO2, TVOC, ethanol concentration and raw hot plate resistance (in ohms)
- an easy to use Measurements struct
- an easy to use initialize function
- no_std embedded compatible

- (SPI not supported, yet)
  

Notes

This is my first device driver project.  I was inspired by Alexander Hübener's excellent ENS160 crate.

STM32  :  The stm32fxx-hal crates are still embedded-hal version 0.2 based.  In order to use this ens160-aq
          crate as a dependency for a STM32 project use version 0.1.4.  For example 
          [dependencies.ens160-aq]
          version = "0.1.4"
          Newer versions of ens160-aq are embedded-hal 1.0+ based and not compatible, you will get i2c and Delay::delay trait bounds build errors.
          
Check your hardware specific hal dependencies in crate.io.  Version 1.0 is the latest.  For example the ESP32-C3 and -C6 hal is esp-idf-hal version "0.43.0", it shows that it depends on embedded-hal version ^1.  Raspberry Pi is
linux-embedded-hal and version "0.4.0" depends on embedded-hal ^1 .

### Recent version history
  - 0.2.10 Updated dependencies
  - 0.2.9 added async support
  - 0.2.8 updated README.md
  - 0.2.7 added Raspberry Pi example
  - 0.2.6 fixed README.MD
  - 0.2.5 added more documentation comments
  - 0.2.2 fixed small bug in InterruptPinConfig, added build() method which just returns the .0 property 
          as u8. Updated the examples/airquality.rs file


Usage
----

Add the dependency to `Cargo.toml`.

~~~~toml
[dependencies.ens160-aq]
version = "0.2"
~~~~

or for async
~~~~toml
[dependencies.ens160-aq]
version = "0.2", features = ["async"]
~~~~

Create an Ens160 struct from an I²C interface and a delay function.
Configure interrupt pin properties if required.  
Initialize ENS160.
set_temp_rh_comp() can be called anytime for temperature and humidity compensation.
Read the ENS160 status and check if new data or group data (if needed) is ready
then get_measurements().  Or you can get ECO2 or TVOC or air quality index separately. 
Note that set_operation_mode(OperationMode::Reset) is available but it will put the ENS160
back to factory defaults including the 24 hour "burn-in" mode.  It does not need to be called
for any other reason.


## Simple Example

A more complete example is in the repository examples path.

For async see async-embassy folder in examples folder of repository.
~~~~rust


use ens160_aq::data::{InterruptPinConfig, Measurements};
use ens160_aq::Ens160;
...


fn main() -> Result<()> {

  ...

  let peripherals = Peripherals::take().unwrap();
  let pins = peripherals.pins;
  let sda = pins.gpio0;
  let scl = pins.gpio1;
  let i2c = peripherals.i2c0;
  let config = I2cConfig::new().baudrate(100.kHz().into());
  let i2c_dev = I2cDriver::new(i2c, sda, scl, &config)?;

  let mut ens160 = Ens160::new(i2c_dev, Ets {});  // Ets is ESP32 IDF delay function

  ens160.initialize().unwrap();


  loop {
    if let Ok(status) = ens160.get_status() {
      if status.new_data_ready() {  // read all measurements
        let measuremnts: Measurements = ens160.get_measurements().unwrap();
        info!("measurements are : {:#?}\n\n", measuremnts);
      }    
      else {
        info!("no new data ready");
      }  
    }

    FreeRtos::delay_ms(30000);
  }

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
