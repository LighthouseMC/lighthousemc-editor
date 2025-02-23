use super::*;


#[derive(Debug)]
pub struct HandshakeC2SPacket<'l> {
    pub session_code : Cow<'l, str>
}

impl<'l> PacketMeta for HandshakeC2SPacket<'l> {
    const PREFIX : u8 = 0;
}

impl<'l> PacketEncode for HandshakeC2SPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.session_code);
    }
}

impl<'l> PacketDecode for HandshakeC2SPacket<'l> {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            session_code : buf.read_decode()?
        })
    }
}
