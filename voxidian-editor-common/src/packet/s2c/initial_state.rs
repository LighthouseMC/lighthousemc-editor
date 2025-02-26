use super::*;


#[derive(Debug)]
pub struct InitialStateS2CPacket<'l> {
    pub plot_id         : u64,
    pub plot_owner_name : Cow<'l, str>,
    pub tree_entries    : Cow<'l, [FileTreeEntry]>
}

impl<'l> PacketMeta for InitialStateS2CPacket<'l> {
    const PREFIX : u8 = 3;
}

impl<'l> PacketEncode for InitialStateS2CPacket<'l> {
    fn encode(&self, buf : &mut PacketBuf) -> () {
        buf.encode_write(&self.plot_id);
        buf.encode_write(&self.plot_owner_name);
        buf.encode_write(&(self.tree_entries.len() as u32));
        for file in &*self.tree_entries {
            buf.encode_write(file.entry_id);
            buf.encode_write(file.is_dir);
            buf.encode_write(&file.parent_dir);
            buf.encode_write(&file.fsname);
        }
    }
}

impl<'l> PacketDecode for InitialStateS2CPacket<'l> {
    fn decode(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
        Ok(Self {
            plot_id         : buf.read_decode::<u64>()?,
            plot_owner_name : buf.read_decode::<Cow<'l, str>>()?,
            tree_entries    : {
                let     count = buf.read_decode::<u32>()? as usize;
                let mut files = Vec::with_capacity(count);
                for _ in 0..count {
                    files.push(FileTreeEntry {
                        entry_id   : buf.read_decode::<u64>()?,
                        is_dir     : buf.read_decode::<bool>()?,
                        parent_dir : buf.read_decode::<Option<u64>>()?,
                        fsname     : buf.read_decode::<String>()?
                    })
                }
                Cow::Owned(files)
            }
        })
    }
}


#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    /// Whether this is a directory or file id depends on `is_dir`.
    pub entry_id   : u64,
    pub is_dir     : bool,
    pub parent_dir : Option<u64>,
    pub fsname     : String
}
