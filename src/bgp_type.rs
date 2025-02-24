use crate::error::ConvertBytesToBgpMessageError;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct AutonomousSystemNumber(u16);

impl From<AutonomousSystemNumber> for u16 {
    fn from(as_number: AutonomousSystemNumber) -> u16 {
        as_number.0
    }
}

impl From<u16> for AutonomousSystemNumber {
    fn from(as_number: u16) -> Self {
        Self(as_number)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct HoldTime(u16);

impl From<HoldTime> for u16 {
    fn from(value: HoldTime) -> Self {
        value.0
    }
}

impl From<u16> for HoldTime {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Default for HoldTime {
    fn default() -> Self {
        Self(0)
    }
}

impl HoldTime {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Version(u8);

impl From<Version> for u8 {
    fn from(value: Version) -> Self {
        value.0
    }
}

impl TryFrom<u8> for Version {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 4 {
            Ok(Self(value))
        } else {
            Err(Self::Error::from(anyhow::anyhow!(
                "excepted version is 1~4, but got {value}",
            )))
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self(4)
    }
}

impl Version {
    pub fn new() -> Self {
        Self::default()
    }
}
