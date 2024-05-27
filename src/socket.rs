use std::sync::Arc;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::info;
use crate::model::{GeneralRequest, GeneralResponse};
use crate::socket_state::SocketState;

pub async fn on_connect(socket: SocketRef, ) {
    info!("Socket Connected: {:?}", socket.id);

    socket.on("join_room",|_socket: SocketRef, Data::<GeneralRequest>(data), socket_state: State<Arc<SocketState>>| async move {
        let general = GeneralRequest {
            room: data.room.clone(),
            message: data.message.clone(),
        };
        info!("General: {:?}", &general);

        _socket.join(general.room.clone()).ok();

        let response = GeneralResponse {
            room: general.room.clone(),
            message: format!("Room joined by client: {}", _socket.id).to_owned(),
            date_time: chrono::Utc::now(),
        };

        _socket.within(general.room.clone()).emit("response", response).ok();
    });

    socket.on("private", |_socket: SocketRef, Data::<Value>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Private: {:?}", data);
    });

    socket.on("message", |_socket: SocketRef, Data::<Value>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Message: {:?}", data);
    });
}