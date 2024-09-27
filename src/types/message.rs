/// Received Firmata message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    ProtocolVersion,
    Analog,
    Digital,
    EmptyResponse,
    AnalogMappingResponse,
    CapabilityResponse,
    PinStateResponse,
    ReportFirmware,
    I2CReply,
}
