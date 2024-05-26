use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use tracing::info;
use crate::model::{GeneralRequest, GeneralResponse};

pub async fn on_connect(socket: SocketRef) {
    info!("Socket Connected: {:?}", socket.id);

    // socket.on("message", |_socket: SocketRef, Data::<dyn Value>(data)| {
    //     info!("Message: {:?}", data);
    // });

    socket.on("join_room", |_socket: SocketRef, Data::<GeneralRequest>(data) | async move {
        let general = GeneralRequest {
            room: data.room.clone(),
            message: data.message.clone()
        };
        info!("General: {:?}", &general);

        _socket.join(general.room.clone()).ok();

        let response = GeneralResponse {
            room: general.room.clone(),
            message: format!("Room joined by client: {}", _socket.id).to_owned(),
            date_time: chrono::Utc::now()
        };

        _socket.within(general.room.clone()).emit("response", response).ok();
    });

    socket.on("private", |_socket: SocketRef, Data::<Value>(data)| async move {
        info!("Private: {:?}", data);

    });

    socket.on("message", |_socket: SocketRef, Data::<serde_json::Value>(data)| {
        info!("Message: {:?}", data);
    });
}
