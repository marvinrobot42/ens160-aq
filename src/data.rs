// no_std support
#[allow(unused_imports)]
#[warn(dead_code)]
use libm::{exp, round, trunc};

#[allow(unused_imports)] // for no_std use
//use num_traits::float::FloatCore;

//use crate::error::Ens160Error;
use bitfield::bitfield;

/// Default I²C address, ADDR pin low
pub const DEFAULT_ADDRESS: u8 = 0x52;
/// the sensor's secondary address ['SECONDARY_ADDRESS']), ADDR pin high
pub const SECONDARY_ADDRESS: u8 = 0x53;

/// A measurement result from the sensor.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Measurements {
    /// CO₂ equivalent (parts per million, ppm)
    pub co2eq_ppm: ECO2,
    /// Total Volatile Organic Compounds (parts per billion, ppb)
    pub tvoc_ppb: u16,
    /// air quality index as enum
    pub air_quality_index: AirQualityIndex,
    /// ethanol concentration in ppb
    pub etoh: u16,
    /// raw resitance value of hot plate in ohms
    pub raw_resistance: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)] // as defined in data sheet
pub enum AirQualityIndex {
    Unavailable = 0,
    Excellent = 1,
    Good = 2,
    Moderate = 3,
    Poor = 4,
    Unhealthy = 5,
    InvalidRange = 6,
}

impl From<u8> for AirQualityIndex {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Unavailable,
            1 => Self::Excellent,
            2 => Self::Good,
            3 => Self::Moderate,
            4 => Self::Poor,
            5 => Self::Unhealthy,
            _ => Self::InvalidRange,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ECO2 {
    pub value: u16,
}

impl ECO2 {
    pub fn get_value(&self) -> u16 {
        self.value
    }
}

impl From<u16> for ECO2 {
    fn from(v: u16) -> Self {
        ECO2 { value: v }
    }
}

/// Operation Mode of the sensor.
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum OperationMode {
    /// DEEP SLEEP mode (low-power standby)
    Sleep = 0x00,
    /// IDLE mode (low power)
    Idle = 0x01,
    /// SSTANDARD Gas Sensing Mode.  Normal run mode
    Standard = 0x02,
    /// Soft reset device:  this will reset the ENS160 back to factory parameters including
    /// the InitialStartupPhase for one hour which will persist until 24 hours of continuous
    /// operating.  Not required for "normal" use.
    Reset = 0xf0,
}

impl From<u8> for OperationMode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => OperationMode::Sleep,
            0x01 => OperationMode::Idle,
            0x02 => OperationMode::Standard,
            0xf0 => OperationMode::Reset, // just for completeness, cannot presist in this state
            _ => unreachable!(),
        }
    }
}

/// Commands for ENS160 command register writes
#[repr(u8)]
pub enum ENS160Command {
    /// No operation
    Nop = 0x00,
    /// Get FW (App) version
    GetAppVersion = 0x0e,
    /// Clears GPR Read Registers
    ClearGPR = 0xcc,
}

// required by bitfield below
#[derive(Debug, Clone, Copy)]
pub enum ValidityFlag {
    NormalOperation,
    WarmupPhase,
    InitialStartupPhase,
    InvalidOutput,
}

// required by bitfield
impl From<u8> for ValidityFlag {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::NormalOperation,
            0x01 => Self::WarmupPhase,
            0x02 => Self::InitialStartupPhase,
            0x03 => Self::InvalidOutput,
            _ => unreachable!(), // only four modes so should never get here
        }
    }
}

bitfield! {
    /// ENS160 status bits
    pub struct Status(u8);
    impl Debug;

    pub bool, new_group_data_ready, _: 0;
    pub bool, new_data_ready, _: 1;
    pub into ValidityFlag, validity_flag, _: 3,2;  // 2 bits
    // 5, 4 not used
    pub bool, error, _: 6;  // probably wrong opmode selected
    pub bool, running_mode, _: 7;
}

#[derive(Debug)]
/// Interrupt pin configuration value
pub struct InterruptPinConfig(pub u8);

impl InterruptPinConfig {
    /// builder sets config value to 0x00
    pub fn builder() -> InterruptPinConfig {
        InterruptPinConfig(0x00)
    }
    /// gets interrupt pin config value from struct
    pub fn get_value(&self) -> u8 {
        self.0
    }
    /// interrupt pin is high when active
    pub fn active_high(mut self) -> Self {
        self.0 |= 0b01000000;
        self
    }
    /// interrupt pin is low when active
    pub fn active_low(mut self) -> Self {
        self.0 &= 0b10111111;
        self
    }
    /// interrupt pin drive is push-pull
    pub fn push_pull(mut self) -> Self {
        self.0 |= 0b00100000;
        self
    }
    /// interrupt pin drive is open drain (not driven)
    pub fn open_drain(mut self) -> Self {
        self.0 &= 0b11011111;
        self
    }
    /// interrupt on new group data ready
    pub fn on_new_group_data(mut self) -> Self {
        self.0 |= 0b0000100;
        self
    }
    /// no interrupt when new group data ready
    pub fn not_new_group_data(mut self) -> Self {
        self.0 &= 0b1111011;
        self
    }
    /// interrupt on new data ready
    pub fn on_new_data(mut self) -> Self {
        self.0 |= 0b0000010;
        self
    }
    /// no interrupt when new data ready
    pub fn not_new_data(mut self) -> Self {
        self.0 &= 0b1111101;
        self
    }
    /// enable interrupt pin
    pub fn enable_interrupt(mut self) -> Self {
        self.0 |= 0b0000001;
        self
    }
    /// disable interrupt pin
    pub fn disable_interrupt(mut self) -> Self {
        self.0 &= 0b1111110;
        self
    }
}
