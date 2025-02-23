use super::*;


#[derive(Debug)]
pub struct DisconnectS2CPacket<'l> {
    pub reason : Cow<'l, str>
}

impl<'l> PacketMeta for DisconnectS2CPacket<'l> {
    const PREFIX : u8 = 0;
}

impl<'l> PacketEncode for DisconnectS2CPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.reason);
    }
}

impl<'l> PacketDecode for DisconnectS2CPacket<'l> {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self { reason : buf.read_decode()? })
    }
}
