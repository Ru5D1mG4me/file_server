pub const FILE_CHUNK_SIZE: u16 = 64512;
pub const EOF: u8 = 0x00;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketMethod {
    Standard = 0x00,
    HandShake = 0x01,
    Download = 0x02,
    Upload = 0x03,
    Close = 0x04,
    List = 0x05,
}

impl TryFrom<u8> for PacketMethod {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(PacketMethod::Standard),
            0x01 => Ok(PacketMethod::HandShake),
            0x02 => Ok(PacketMethod::Download),
            0x03 => Ok(PacketMethod::Upload),
            0x04 => Ok(PacketMethod::Close),
            0x05 => Ok(PacketMethod::List),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    SessionID = 0x10,
    ChunkID = 0x11,
    ChunksCount = 0x12,
    ChunkSize = 0x13,
    DataChunk = 0x14,
    Command = 0x15,
    Path = 0x16,
    Status = 0x17,
    CRC = 0x18,
    FileSize = 0x19,
    ErrorMsg = 0x1A,
}

impl TryFrom<u8> for FieldType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(FieldType::SessionID),
            0x11 => Ok(FieldType::ChunkID),
            0x12 => Ok(FieldType::ChunksCount),
            0x13 => Ok(FieldType::ChunkSize),
            0x14 => Ok(FieldType::DataChunk),
            0x15 => Ok(FieldType::Command),
            0x16 => Ok(FieldType::Path),
            0x17 => Ok(FieldType::Status),
            0x18 => Ok(FieldType::CRC),
            0x19 => Ok(FieldType::FileSize),
            0x1A => Ok(FieldType::ErrorMsg),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldStatus {
    Ready = 0x20,
    Sent = 0x21,
    Received = 0x22,
    Error = 0x23,
    Ok = 0x24,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldCommand {
    Start = 0x30,
    Next = 0x31,
    Retry = 0x32,
    End = 0x33,
    Cancel = 0x34,
    Send = 0x35,
    Continue = 0x36,
}

impl TryFrom<u8> for FieldCommand {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x30 => Ok(FieldCommand::Start),
            0x31 => Ok(FieldCommand::Next),
            0x32 => Ok(FieldCommand::Retry),
            0x33 => Ok(FieldCommand::End),
            0x34 => Ok(FieldCommand::Cancel),
            0x35 => Ok(FieldCommand::Send),
            0x36 => Ok(FieldCommand::Continue),
            _ => Err(()),
        }
    }
}

pub enum NextAction {
    None,
    Terminate,
    ReadData,
    WriteData,
    End,
    Cancel,
}

pub enum Action {
    SendResponse(NextAction),
    SendError,
    RequestFileInfoRead,
    RequestFileInfoWrite,
}