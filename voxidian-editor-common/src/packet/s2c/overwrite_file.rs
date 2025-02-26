use super::*;


#[derive(Debug)]
pub struct OverwriteFileS2CPacket<'l> {
    pub file_id  : u64,
    pub contents : FileContents<'l>
}

impl<'l> PacketMeta for OverwriteFileS2CPacket<'l> {
    const PREFIX : u8 = 4;
}

impl<'l> PacketEncode for OverwriteFileS2CPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(self.file_id);
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
            file_id  : buf.read_decode()?,
            contents : {
                let is_text = buf.read_decode::<bool>()?;
                if (is_text) { FileContents::Text(buf.read_decode()?) }
                else { FileContents::NonText }
            }
        })
    }
}


#[derive(Debug, Clone)]
pub enum FileContents<'l> {
    NonText,
    Text(Cow<'l, str>)
}

impl<'l> FileContents<'l> {
    pub fn as_ref(&'l self) -> FileContents<'l> {
        match (self) {
            Self::NonText    => Self::NonText,
            Self::Text(text) => Self::Text(Cow::Borrowed(text))
        }
    }
}
