use super::*;
use super::c2s::SelectionRange;


#[derive(Debug)]
pub struct SelectionsS2CPacket<'l> {
    pub client_id   : u64,
    pub client_name : Cow<'l, str>,
    pub colour      : u16,
    pub selections  : Option<(u64, Vec<SelectionRange>)>
}

impl<'l> PacketMeta for SelectionsS2CPacket<'l> {
    const PREFIX : u8 = 6;
}

impl<'l> PacketEncode for SelectionsS2CPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(self.client_id);
        buf.encode_write(&self.client_name);
        buf.encode_write(self.colour);
        if let Some((file_id, selections)) = &self.selections {
            buf.encode_write(true);
            buf.encode_write(file_id);
            buf.encode_write(selections.len() as u32);
            for selection in selections {
                buf.encode_write(selection.start as u32);
                buf.encode_write(selection.end   as u32);
            }
        } else {
            buf.encode_write(false);
        }
    }
}

impl<'l> PacketDecode for SelectionsS2CPacket<'l> {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            client_id   : buf.read_decode()?,
            client_name : buf.read_decode()?,
            colour      : buf.read_decode()?,
            selections  : if (buf.read_decode::<bool>()?){
                let     file_id    = buf.read_decode::<u64>()?;
                let     len        = buf.read_decode::<u32>()? as usize;
                let mut selections = Vec::with_capacity(len);
                for _ in 0..len { selections.push(SelectionRange {
                    start : buf.read_decode::<u32>()? as usize,
                    end   : buf.read_decode::<u32>()? as usize
                }); }
                Some((file_id, selections))
            } else { None }
        })
    }
}
