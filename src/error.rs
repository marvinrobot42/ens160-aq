use core::fmt::Formatter;

use embedded_hal::blocking::i2c::{Write, WriteRead};

/// All possible errors
/// Display not implemented for no_std support
pub enum Ens160Error<I2C>
where
    I2C: WriteRead + Write,
    <I2C as WriteRead>::Error: core::fmt::Debug,
    <I2C as Write>::Error: core::fmt::Debug,
{
    /// Error during I2C write operation.
    WriteError(<I2C as Write>::Error),
    /// Error during I2C WriteRead operation.
    WriteReadError(<I2C as WriteRead>::Error),
    /// Got an unexpected Part Id during sensor initalization.
    UnexpectedChipId(u16),
    /// unexpected Operation Mode
    OpModeNotCorrect(u8),
}

impl<I2C> core::fmt::Debug for Ens160Error<I2C>
where
    I2C: WriteRead + Write,
    <I2C as WriteRead>::Error: core::fmt::Debug,
    <I2C as Write>::Error: core::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Ens160Error::WriteReadError(e) => f.debug_tuple("WriteReadError").field(e).finish(),
            Ens160Error::WriteError(e) => f.debug_tuple("WriteError").field(e).finish(),
            Ens160Error::UnexpectedChipId(chip_id) => f
                .debug_tuple("Expected part id 352, got : ")
                .field(chip_id)
                .finish(),
            Ens160Error::OpModeNotCorrect(expected) => f
                .debug_tuple("Incorrect ENS160 operation, got :")
                .field(expected)
                .finish(),
        }
    }
}
