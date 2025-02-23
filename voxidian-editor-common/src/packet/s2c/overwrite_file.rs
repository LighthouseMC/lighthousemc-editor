use super::*;


#[derive(Debug)]
pub struct OverwriteFileS2CPacket<'l> {
    pub id       : u64,
    pub contents : FileContents<'l>
}

impl<'l> PacketMeta for OverwriteFileS2CPacket<'l> {
    const PREFIX : u8 = 4;
}

impl<'l> PacketEncode for OverwriteFileS2CPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(self.id);
        match (&self.contents) {
            FileContents::NonText    => buf.encode_write(false),
            FileContents::Text(text) => {
                buf.encode_write(true);
                buf.encode_write(text);
            },
        }
    }
}

impl<'l> PacketDecode for OverwriteFileS2CPacket<'l> {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            id       : buf.read_decode()?,
            contents : {
                let is_text = buf.read_decode::<bool>()?;
                if (is_text) { FileContents::Text(buf.read_decode()?) }
                else { FileContents::NonText }
            }
        })
    }
}


#[derive(Debug)]
pub enum FileContents<'l> {
    NonText,
    Text(Cow<'l, str>)
}
