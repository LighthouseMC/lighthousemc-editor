use super::*;


#[derive(Debug)]
pub struct SelectionsC2SPacket {
    pub selections : Option<(u32, Vec<SelectionRange>)>
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
                buf.encode_write(selection.start_line   as u32);
                buf.encode_write(selection.start_column as u32);
                buf.encode_write(selection.end_line     as u32);
                buf.encode_write(selection.end_column   as u32);
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
                    start_line   : buf.read_decode::<u32>()? as usize,
                    start_column : buf.read_decode::<u32>()? as usize,
                    end_line     : buf.read_decode::<u32>()? as usize,
                    end_column   : buf.read_decode::<u32>()? as usize
                }); }
                Some((file_id, selections))
            } else { None }
        })
    }
}


#[derive(Debug, Clone)]
pub struct SelectionRange {
    pub start_line   : usize,
    pub start_column : usize,
    pub end_line     : usize,
    pub end_column   : usize
}
