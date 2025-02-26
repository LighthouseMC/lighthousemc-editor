use super::*;


#[derive(Debug)]
pub struct SelectionsC2SPacket {
    pub selections : Option<(u64, Vec<SelectionRange>)>
}

impl PacketMeta for SelectionsC2SPacket {
    const PREFIX : u8 = 5;
}

impl PacketEncode for SelectionsC2SPacket {
    fn encode(&self, buf : &mut PacketBuf) -> () {
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

impl PacketDecode for SelectionsC2SPacket {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            selections : if (buf.read_decode::<bool>()?){
                let     file_id    = buf.read_decode()?;
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    pub start : usize,
    pub end   : usize
}
