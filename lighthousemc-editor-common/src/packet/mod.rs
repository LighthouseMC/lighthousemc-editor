mod buf;
pub use buf::PacketBuf;
mod encode;
pub use encode::{ PacketEncode, PrefixedPacketEncode };
mod decode;
pub use decode::{ PacketDecode, PrefixedPacketDecode, DecodeError };
mod meta;
pub use meta::PacketMeta;

pub mod s2c;
pub mod c2s;


macro packet_group(
    $vis:vis enum $ident:ident $( < $( $lt:lifetime ),* $(,)? > )? { $($variantname:ident ( $variantinner:ty )),* $(,)? }
) {
    #[derive(Debug)]
    $vis enum $ident $( < $( $lt , )* > )? {
        $($variantname ( $variantinner )),*
    }
    impl $( < $( $lt , )* > )? PrefixedPacketEncode for $ident $( < $( $lt , )* > )? {
        fn encode_prefixed(&self, buf : &mut PacketBuf) -> () {
            match (self) {
                $(Self::$variantname(inner) => <$variantinner as PrefixedPacketEncode>::encode_prefixed(inner, buf)),*
            }
        }
    }
    impl $( < $( $lt , )* > )? PrefixedPacketDecode for $ident $( < $( $lt , )* > )? {
        fn decode_prefixed(buf : &mut PacketBuf) -> Result<Self, DecodeError> {
            let prefix = buf.read_u8()?;
            $( if (prefix == <$variantinner as PacketMeta>::PREFIX) {
                return Ok(Self::$variantname(<$variantinner as PacketDecode>::decode(buf)?));
            } )*
            Err(DecodeError::UnknownPacketPrefix(prefix))
        }
    }
}


pub fn encode(packet : impl PrefixedPacketEncode) -> Vec<u8> {
    let mut buf = PacketBuf::new();
    packet.encode_prefixed(&mut buf);
    buf.to_vec()
}

pub fn decode<P : PrefixedPacketDecode>(data : &[u8]) -> Result<P, DecodeError> {
    let mut buf = PacketBuf::from(data);
    P::decode_prefixed(&mut buf)
}
