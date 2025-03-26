use std::fmt::Display;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    MissingPrefix,
    InvalidNumber,
    OutOfRange,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", *self as u8)
    }
}

impl TryFrom<u8> for Reg {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Reg::R0),
            1 => Ok(Reg::R1),
            2 => Ok(Reg::R2),
            3 => Ok(Reg::R3),
            _ => Err(ParseError::OutOfRange),
        }
    }
}

impl TryFrom<&str> for Reg {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let number = value.strip_prefix('r').ok_or(ParseError::MissingPrefix)?;
        let number = number
            .parse::<u8>()
            .map_err(|_| ParseError::InvalidNumber)?;
        Reg::try_from(number).map_err(|_| ParseError::OutOfRange)
    }
}
