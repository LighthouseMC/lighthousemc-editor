use lighthousemc_editor_common::packet::{ self, PrefixedPacketEncode, PrefixedPacketDecode, DecodeError };
use lighthousemc_editor_common::packet::s2c::*;
use std::time::Duration;
use tokio::time::timeout;
use axum::extract::ws::{ WebSocket, Message as WebSocketMessage };
use axum::body::Bytes;


pub(crate) async fn send_packet(socket : &mut WebSocket, p : impl PrefixedPacketEncode) -> Result<(), ()> {
    match (timeout(Duration::from_secs(1), socket.send(WebSocketMessage::Binary(Bytes::from(packet::encode(p))))).await) {
        Ok(out) => out.map_err(|_| ()),
        Err(_) => Err(())
    }
}

pub(crate) async fn read_packet<P : PrefixedPacketDecode>(socket : &mut WebSocket) -> Result<P, ()> {
    let out = socket.recv().await;
    handle_packet_result::<P>(socket, out).await
}

pub(crate) async fn try_read_packet<P : PrefixedPacketDecode>(socket : &mut WebSocket) -> Result<Option<P>, ()> {
    match (timeout(Duration::ZERO, socket.recv()).await) {
        Ok(out) => handle_packet_result::<P>(socket, out).await.map(|out| Some(out)),
        Err(_) => Ok(None)
    }
}

async fn handle_packet_result<P : PrefixedPacketDecode>(socket : &mut WebSocket, out : Option<Result<WebSocketMessage, axum::Error>>) -> Result<P, ()> {
    let out = match (out) {
        Some(Ok(WebSocketMessage::Binary(data))) => { match (packet::decode::<P>(&data)) {
            Ok(out) => Ok(Ok(out)),
            Err(err) => { match (err) {
                DecodeError::EndOfBuffer            => Err("incomplete packet".into()),
                DecodeError::InvalidData(_)         => Err("invalid packet data".into()),
                DecodeError::UnconsumedBuffer       => Err("invalid packet".into()),
                DecodeError::UnknownPacketPrefix(_) => Err("unknown packet".into()),
            } }
        } },
        Some(Ok(_))  => Err("bad packet format".into()),
        Some(Err(_)) => Err("connection interrupted".into()),
        None         => Ok(Err(()))
    };
    match (out) {
        Ok(out) => out,
        Err(err) => {
            let _ = send_packet(socket, DisconnectS2CPacket { reason : err }).await;
            Err(())
        }
    }
}
