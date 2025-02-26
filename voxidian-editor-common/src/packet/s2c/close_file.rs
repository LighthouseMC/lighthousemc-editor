use super::*;


#[derive(Debug)]
pub struct CloseFileS2CPacket {
    pub file_id : u64
}

impl PacketMeta for CloseFileS2CPacket {
    const PREFIX : u8 = 7;
}

impl PacketEncode for CloseFileS2CPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.file_id);
    }
}

impl PacketDecode for CloseFileS2CPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            file_id : buf.read_decode()?
        })
    }
}
