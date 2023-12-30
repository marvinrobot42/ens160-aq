#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod error;

use crate::error::Ens160Error;

pub mod data;
use crate::data::ENS160Command;
use crate::data::OperationMode;

use data::Measurements;
use data::{AirQualityIndex, Status, ECO2};

pub mod constants;

use crate::constants::DeviceAddress::{Primary, Secondary};

#[allow(unused_imports)]
use crate::constants::{
    ENS160_COMMAND, ENS160_CONFIG, ENS160_DATA_AQI, ENS160_DATA_ECO2, ENS160_DATA_ETOH,
    ENS160_DATA_MISR, ENS160_DATA_RH, ENS160_DATA_T, ENS160_DATA_TVOC, ENS160_DEVICE_STATUS,
    ENS160_GPR_READ, ENS160_GPR_WRITE, ENS160_GRP_READ6, ENS160_OPMODE, ENS160_PART_ID,
    ENS160_RH_IN, ENS160_TEMP_IN,
};

use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::{Write, WriteRead},
};
use libm::{powf, truncf};
use log::{debug, info};

/// Default I²C address, ADDR pin low
/// which is default depends on actual ENS160 board
/// the IC itself requires the ADDR to NOT be left open
//const DEFAULT_ADDRESS: u8 = 0x52;
/// the sensor's secondary address ['SECONDARY_ADDRESS']), ADDR pin high
//const SECONDARY_ADDRESS: u8 = 0x53;

/// the ENS160 device
pub struct Ens160<I2C, D> {
    /// I²C interface
    i2c: I2C,

    /// I²C device address
    address: u8,
    delayer: D,
}

impl<I2C, D> Ens160<I2C, D>
where
    I2C: WriteRead + Write,
    <I2C as WriteRead>::Error: core::fmt::Debug,
    <I2C as Write>::Error: core::fmt::Debug,
    D: DelayMs<u8>,
{
    /// create new ENS160 driver with default I2C address: ADDR pin low
    pub fn new(i2c: I2C, delayer: D) -> Self {
        debug!("new called");
        Self {
            i2c,
            address: Primary.into(),
            delayer,
        }
    }

    /// create new ENS160 driver with secondary I2C address: ADDR pin high
    pub fn new_secondary_address(i2c: I2C, delayer: D) -> Self {
        Self {
            i2c,
            address: Secondary.into(),
            delayer,
        }
    }

    /// give back the I2C interface
    pub fn release(self) -> I2C {
        self.i2c
    }

    // command_buf is an u8 array that starts with command byte followed by command data byte(s)
    fn write_command<const N: usize>(
        &mut self,
        command_buf: [u8; N],
    ) -> Result<(), Ens160Error<I2C>> {
        // debug!("write_command : {:#?}", command_buf);
        self.i2c
            .write(self.address, &command_buf)
            .map_err(Ens160Error::WriteError)
    }

    fn read_register(
        &mut self,
        register_address: u8,
        buffer: &mut [u8],
    ) -> Result<(), Ens160Error<I2C>> {
        let mut command_buffer = [0u8; 1];
        command_buffer[0] = register_address;
        // let mut result_buffer = [0u8; N];
        self.i2c
            .write_read(self.address, &command_buffer, buffer)
            .map_err(Ens160Error::WriteReadError)?;
        Ok(())
    }

    /// set operating mode:  deep sleep, idle, normal operation or reset
    /// reset puts the ENS160 into initial start mode for an hour and it still will persist
    /// until 24 hours of continuous power on.  
    pub fn set_operation_mode(
        &mut self,
        mode: OperationMode,
    ) -> Result<OperationMode, Ens160Error<I2C>> {
        debug!("setting ens160 operation mode to {:#?}", mode);
        self.write_command([ENS160_OPMODE, mode as u8])?;
        self.delayer.delay_ms(50);
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(ENS160_OPMODE, &mut result_buf)?;
        return Ok(OperationMode::from(result_buf[0]));
    }

    /// Returns ENS160 part ID, expect 0x0160
    pub fn get_part_id(&mut self) -> Result<u16, Ens160Error<I2C>> {
        let mut result_buf = [0; 2];
        self.read_register(ENS160_PART_ID, &mut result_buf[0..2])?;
        //   .map(u16::from_le_bytes) // ENS160 returns little endian data

        Ok(u16::from_le_bytes(result_buf))
    }

    /// Gets ENS160 firmware version (this library was tested with 5.4.6)
    pub fn get_firmware_version(&mut self) -> Result<(u8, u8, u8), Ens160Error<I2C>> {
        self.write_command([ENS160_COMMAND, ENS160Command::GetAppVersion as u8])?;
        let mut result_buf: [u8; 8] = [0; 8];
        self.read_register(ENS160_GPR_READ, &mut result_buf)?;
        Ok((result_buf[4], result_buf[5], result_buf[6]))
    }

    /// Clears group data registers
    pub fn clear_command(&mut self) -> Result<(), Ens160Error<I2C>> {
        self.write_command([ENS160_COMMAND, ENS160Command::Nop as u8])?;
        self.write_command([ENS160_COMMAND, ENS160Command::ClearGPR as u8])?;
        Ok(())
    }

    /// Gets Equivalent Carbon Dioxide  measurement from the sensor in ppm, returns ECO2 enum.
    pub fn get_eco2(&mut self) -> Result<ECO2, Ens160Error<I2C>> {
        let mut result_buf = [0; 2];
        self.read_register(ENS160_DATA_ECO2, &mut result_buf)?;
        // debug!("eco2 u16 = {:#?}", result_buf);
        let eco2 = u16::from_le_bytes(result_buf);
        // debug("eco2 u16 = {:#04x}", eco2);
        Ok(ECO2::from(eco2))
    }

    /// Get Total Volitaile organic compounds in ppb.  No range for indexing given in data sheet
    pub fn get_tvoc(&mut self) -> Result<u16, Ens160Error<I2C>> {
        let mut result_buf = [0; 2];
        self.read_register(ENS160_DATA_TVOC, &mut result_buf)?;
        Ok(u16::from_le_bytes(result_buf))
        //.map(u16::from_le_bytes)
    }

    /// Gets Air Quality Index value from sensor.
    /// The air quality index value is matched to the AirQualityIndex enum (resultant)
    pub fn get_airquality_index(&mut self) -> Result<AirQualityIndex, Ens160Error<I2C>> {
        let mut result_buf = [0; 1];
        self.read_register(ENS160_DATA_AQI, &mut result_buf)?;
        debug!(" read ENS160_DATA_AQI result is {}", result_buf[0]);
        Ok(AirQualityIndex::from(result_buf[0]))
    }

    /// get ethanol concentration in ppb
    pub fn get_etoh(&mut self) -> Result<u16, Ens160Error<I2C>> {
        let mut result_buf: [u8; 2] = [0; 2];
        self.read_register(ENS160_DATA_ETOH, &mut result_buf)?;
        Ok(u16::from_le_bytes(result_buf))
    }

    /// get raw resistance value which can be used for custom calulations, in ohms
    pub fn get_raw_resistance(&mut self) -> Result<f32, Ens160Error<I2C>> {
        let mut result_buf: [u8; 2] = [0; 2];
        self.read_register(ENS160_GRP_READ6, &mut result_buf)?;
        // convert to ohm, see datasheet section 7
        let exponent: f32 = u16::from_le_bytes(result_buf) as f32;
        //debug!("raw resistance before conversion {}", exponent);
        let resistance = powf(2.0, exponent / 2048.0);
        Ok(resistance)
    }

    /// get ENS160 status flags
    pub fn get_status(&mut self) -> Result<Status, Ens160Error<I2C>> {
        let mut result_buf = [0; 1];
        self.read_register(ENS160_DEVICE_STATUS, &mut result_buf)?;
        //debug!(" raw ens160 status byte is {:#04x}", result_buf[0]);
        Ok(Status(result_buf[0]))
    }

    /// read ENS160 group data
    pub fn get_group_data(&mut self) -> Result<[u8; 8], Ens160Error<I2C>> {
        let mut result_buf: [u8; 8] = [0; 8];
        self.read_register(ENS160_GPR_READ, &mut result_buf)?;
        // debug!(" group register read results are {:#?}", result_buf);
        Ok(result_buf)
    }
    /// set the temperature in degrees C and relative humdity in percent for compensation calculation
    pub fn set_temp_rh_comp(
        &mut self,
        temp_c: f32,
        rh_percent: u16,
    ) -> Result<(), Ens160Error<I2C>> {
        let mut buffer: [u8; 2];
        let temp_val: u16 = truncf((temp_c + 273.15) * 64.0) as u16; // to Kelvin and scale it
                                                                     //info!("setting temp comp to {:#04x}", temp_val.to_le());
        buffer = temp_val.to_le_bytes(); // ???? or is it be
        self.write_command([ENS160_TEMP_IN, buffer[0], buffer[1]])?;

        buffer = rh_percent.to_le_bytes();
        //debug!("setting rh comp to {:#04x} {:#04x}", buffer[0], buffer[1]);
        self.write_command([ENS160_RH_IN, buffer[0], buffer[1]])?;

        Ok(())
    }

    pub fn get_temp_rh_comp(&mut self) -> Result<(f32, u16), Ens160Error<I2C>> {
        let mut result_buf: [u8; 2] = [0; 2];
        self.read_register(ENS160_DATA_T, &mut result_buf)?;
        let value: u16 = u16::from_le_bytes(result_buf);
        let temp_comp_c = ((value as f32) / 64.0) - 273.15;
        //debug!("temp c compensation is {}", temp_comp_c);

        self.read_register(ENS160_DATA_RH, &mut result_buf)?;
        let rh: u16 = u16::from_le_bytes(result_buf);
        //debug!("read rh back as {}", rh);
        Ok((temp_comp_c, rh))
    }

    /// configure the interrupt pin of ENS160.  See data sheet for config:u8 parameter
    /// or use the handy InterruptPinConfig::builder() and its function to generate the
    /// config:u8 parameter for you.
    /// returns the ENS160 config register read back (should equal value written)
    pub fn config_interrupt_pin(&mut self, config: u8) -> Result<u8, Ens160Error<I2C>> {
        self.write_command([ENS160_CONFIG, config])?;
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(ENS160_CONFIG, &mut result_buf)?;
        Ok(result_buf[0])
    }

    /// initialize the ENS160 device
    pub fn initialize(&mut self) -> Result<bool, Ens160Error<I2C>> {
        //self.reset()?;  NO, this will put ENS160 back to factory defaults including InitialStartUp 24 hours
        // self.set_operation_mode(OperationMode::Reset)?;
        //self.delayer.delay_ms(250);
        self.set_operation_mode(OperationMode::Idle)?;
        //self.idle_mode()?;
        debug!(
            " command to idle, ENS160 status is {:#?}",
            self.get_status().unwrap()
        );
        if let Ok(part_id) = self.get_part_id() {
            if part_id != 0x0160 {
                Err(Ens160Error::UnexpectedChipId(part_id as u16))
            } else {
                info!("ENS160 part id is good {}", part_id);
                self.delayer.delay_ms(50);
                self.clear_command()?;
                debug!(
                    " command to clear grp data, ENS160 status is {:#?}",
                    self.get_status().unwrap()
                );
                self.delayer.delay_ms(50);
                let (fw_major, fw_minor, fw_build) = self.get_firmware_version()?;
                info!("firmware version {}.{}.{}", fw_major, fw_minor, fw_build);
                self.delayer.delay_ms(10);
                // self.standard_mode()?;
                let new_mode = self.set_operation_mode(OperationMode::Standard)?;
                if new_mode != OperationMode::Standard {
                    return Err(Ens160Error::OpModeNotCorrect(new_mode as u8));
                }
                self.delayer.delay_ms(150);
                debug!(
                    " command to std mode, ENS160 status is {:#?}",
                    self.get_status().unwrap()
                );
                // read opmode register
                let mut result_buf: [u8; 1] = [0; 1];
                self.read_register(ENS160_OPMODE, &mut result_buf).unwrap();
                debug!("opmode read is {:#04x}", result_buf[0]);
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// get all measurements from sensor
    pub fn get_measurements(&mut self) -> Result<Measurements, Ens160Error<I2C>> {
        let eco2 = self.get_eco2()?;
        let tvoc = self.get_tvoc()?;
        let aqi = self.get_airquality_index()?;
        let etoh = self.get_etoh()?;
        let raw_resistance = self.get_raw_resistance()?;
        let measurements: Measurements = Measurements {
            co2eq_ppm: eco2,
            tvoc_ppb: tvoc,
            air_quality_index: aqi,
            etoh,
            raw_resistance,
        };
        Ok(measurements)
    }

    // Interrupt pin configuration
}
