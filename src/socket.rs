use std::sync::Arc;
use serde::Serialize;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::info;
use crate::model::{GeneralRequest, GeneralResponse, Message};
use crate::socket_state::SocketState;

#[derive(Serialize)]
struct Messages {
    messages: Vec<Message>
}

pub async fn on_connect(socket: SocketRef, ) {
    info!("Socket Connected: {:?}", socket.id);

    socket.on("join_room",|_socket: SocketRef, Data::<GeneralRequest>(data), socket_state: State<Arc<SocketState>>| async move {
        let general = GeneralRequest {
            room: data.room.clone(),
            message: data.message.clone(),
        };
        info!("General: {:?}", &general);

        _socket.leave_all().ok();
        _socket.join(general.room.clone()).ok();
        let messages = socket_state.get_messages(&general.room).await;


        let response = GeneralResponse {
            room: general.room.clone(),
            message: format!("Room joined by client: {}", _socket.id).to_owned(),
            date_time: chrono::Utc::now(),
        };

        // _socket.within(general.room.clone()).emit("response", response).ok();
        _socket.emit("messages",Messages{ messages }).ok();
    });

    socket.on("private", |_socket: SocketRef, Data::<Value>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Private: {:?}", data);
    });

    socket.on("message", |_socket: SocketRef, Data::<GeneralRequest>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Message: {:?}", data);
        let response = GeneralResponse {
            room: data.room.clone(),
            message: format!("Message By Client: {}", data.message).to_owned(),
            date_time: chrono::Utc::now(),
        };

        socket_state.insert(&data.room, Message {
            room: data.room.clone(),
            message: data.message.clone(),
            date_time: response.date_time.clone()
        }).await;

        // INSERT THE MESSAGE INTO DB
        // socket_state.db.insert_message(&data.room, Message {
        //     room: data.room.clone(),
        //     message: data.message.clone(),
        //     date_time: response.date_time.clone()
        // }).await.unwrap();

        _socket.within(data.room.clone()).emit("response", response).ok();

    });
}
