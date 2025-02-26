use super::*;


#[derive(Debug)]
pub struct OpenFileC2SPacket {
    pub file_id : u64
}

impl PacketMeta for OpenFileC2SPacket {
    const PREFIX : u8 = 2;
}

impl PacketEncode for OpenFileC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.file_id);
    }
}

impl PacketDecode for OpenFileC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            file_id : buf.read_decode()?
        })
    }
}
