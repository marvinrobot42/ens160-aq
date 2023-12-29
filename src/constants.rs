// ENS160 registers
pub const ENS160_PART_ID: u8 = 0x00;
pub const ENS160_OPMODE: u8 = 0x10;
pub const ENS160_CONFIG: u8 = 0x11;
pub const ENS160_COMMAND: u8 = 0x12;
pub const ENS160_TEMP_IN: u8 = 0x13;
pub const ENS160_RH_IN: u8 = 0x15;
pub const ENS160_DEVICE_STATUS: u8 = 0x20;
pub const ENS160_DATA_AQI: u8 = 0x21;
pub const ENS160_DATA_TVOC: u8 = 0x22;
pub const ENS160_DATA_ECO2: u8 = 0x24;
pub const ENS160_DATA_ETOH: u8 = 0x22; // is this correct?
pub const ENS160_DATA_T: u8 = 0x30;
pub const ENS160_DATA_RH: u8 = 0x32;
pub const ENS160_DATA_MISR: u8 = 0x38;
pub const ENS160_GPR_WRITE: u8 = 0x40;
pub const ENS160_GPR_READ: u8 = 0x48;
pub const ENS160_GRP_READ6: u8 = 0x4e;

#[repr(u8)]
/// ENS160 I2C device address
/// do not float the ADDR pin as its value would be undefined.  Check your ENS160 board specs.
#[derive(Debug, Clone, Copy)]
pub enum DeviceAddress {
    /// ADDR pin low
    Primary = 0x52,
    /// ADDR pin high
    Secondary = 0x53,
}

impl From<DeviceAddress> for u8 {
    fn from(value: DeviceAddress) -> Self {
        match value {
            DeviceAddress::Primary => 0x52,
            DeviceAddress::Secondary => 0x53,
        }
    }
}

impl Default for DeviceAddress {
    fn default() -> Self {
        Self::Primary
    }
}
