
// *** ESP32-C6 embassy async example ************
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output},
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    timer::{timg::TimerGroup, ErasedTimer, OneShotTimer},
};


use ens160_aq::data::{InterruptPinConfig, Measurements};
use ens160_aq::Ens160;

use log::{info, debug};

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let timer0 = OneShotTimer::new(timg0.timer0.into());
    let timers = [timer0];
    let timers = mk_static!([OneShotTimer<ErasedTimer>; 1], timers);
    esp_hal_embassy::init(&clocks, timers);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c0 = I2C::new_async(
        peripherals.I2C0,
        io.pins.gpio6,   // ESP32-c6 mini
        io.pins.gpio7,   // ESP32-c6 mini
        400.kHz(),
        &clocks,
    );

   
    esp_println::logger::init_logger(log::LevelFilter::Debug);
    info!("info logging!");
    debug!("debug logging");

    // ens160-aq
    let mut ens160 = Ens160::new_secondary_address(i2c0, embassy_time::Delay);
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
    if let Ok(new_config_int) = ens160.config_interrupt_pin(int_pin_config).await {
        if (new_config_int == int_pin_config) {
            info!("config_interrrupt_pin was good, new value is {:#04x}", new_config_int);
        } else {
            log::error!("config_interrupt_pin() not good, expected {:#04x}, got {:#04x}", int_pin_config, new_config_int);
        }
    }   
    
    ens160.initialize().await.unwrap();

    // optional: usually not required
    ens160.set_temp_rh_comp(21.5, 41).await.unwrap();
    Timer::after((Duration::from_secs(1)));
    let (temp_c, rh) = ens160.get_temp_rh_comp().await.unwrap();
    info!(
        "compensation set to temperature = {} C, relative humidity = {} %", temp_c, rh
    );


    let mut led_pin23 = Output::new(io.pins.gpio23, Level::High);
    led_pin23.toggle();
    Timer::after(Duration::from_millis(2000)).await;
    led_pin23.toggle();



    loop {
        if let Ok(status) = ens160.get_status().await {
            info!("ens160 status is {:#?}", status);
            if status.new_data_ready() {
                // read all measurements
                let measuremnts: Measurements = ens160.get_measurements().await.unwrap();
                info!("measurements are : {:#?}\n\n", measuremnts);
            }
            if status.new_group_data_ready() {  // useful to see raw data values
                let group_data: [u8; 8] = ens160.get_group_data().await.unwrap();
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

        led_pin23.toggle();
        Timer::after(Duration::from_millis(2000)).await;
    }

}