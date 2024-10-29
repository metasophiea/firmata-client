/// Received Firmata message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    ProtocolVersion(u8, u8),
    Analog(Vec<(u8, u8)>),
    Digital(Vec<(u8, bool)>),
    EmptyResponse,
    AnalogMappingResponse,
    CapabilityResponse,
    PinStateResponse,
    ReportFirmwareName(String),
    ReportFirmwareVersion(String),
    I2CReply,
}

impl Message {
    #[must_use]
    pub fn try_as_analog(&self) -> Option<&Vec<(u8, u8)>> {
        if let Message::Analog(data) = self {
            Some(data)
        } else {
            None
        }
    }
    #[must_use]
    pub fn try_as_digital(&self) -> Option<&Vec<(u8, bool)>> {
        if let Message::Digital(data) = self {
            Some(data)
        } else {
            None
        }
    }
}