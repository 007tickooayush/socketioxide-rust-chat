use std::sync::Arc;
use serde::Serialize;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::info;
use crate::model::{GeneralRequest, GeneralResponse, Message, PrivateMessage, PrivateMessageReq};
use crate::socket_state::SocketState;

#[derive(Serialize)]
struct Messages {
    messages: Vec<Message>
}

pub async fn on_connect(socket: SocketRef, ) {
    info!("Socket Connected: {:?}", socket.id);

    /// todo: INITIALIZE THE SOCKET IDS INTO A VARIABLE PAIRED TO A USERNAME <br/>
    /// and store the list of key value pair in the memory store or in the DB

    socket.join(socket.id.clone()).ok();

    /// Join a room and save the state of the room up to the message limit defined in the
    /// `get_messages` function <br/>
    /// *NOTE* The message reading is not being performed from the DB and is being read from the memory store
    /// provided by the `SocketState` struct implementation through `socketioxide` >v8.0 library
    socket.on("join_room",|_socket: SocketRef, Data::<GeneralRequest>(data), socket_state: State<Arc<SocketState>>| async move {
        let general = GeneralRequest {
            room: data.room.clone(),
            message: data.message.clone(),
        };
        info!("General: {:?}", &general);

        _socket.leave_all().ok();
        _socket.join(general.room.clone()).ok();
        let messages = socket_state.get_messages(&general.room).await;


        // let response = GeneralResponse {
        //     room: general.room.clone(),
        //     message: format!("Room joined by client: {}", _socket.id).to_owned(),
        //     date_time: chrono::Utc::now(),
        // };

        // _socket.within(general.room.clone()).emit("response", response).ok();
        _socket.emit("messages",Messages{ messages }).ok();
    });

    socket.on("private", |_socket: SocketRef, Data::<PrivateMessageReq>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Private: {:?}", data);
        let sender = match data.sender.clone() {
            Some(sender) => sender,
            None => _socket.id.clone().to_string()
        };
        let response = PrivateMessageReq {
            message:  data.message.to_owned(),
            sender: Some(sender),
            receiver: data.receiver.clone(),
        };

        // INSERT THE MESSAGE INTO DB
        socket_state.insert_private_messages(response.clone()).await;

        _socket.to(data.receiver.clone()).emit("resp",response).ok();
    });

    /// Handling the message from the client <br/>
    /// *NOTE:* The mechanism is not built for Ultra high throughput as OPS limit is not set and may exceed
    /// if too many write operations are performed simultaneously <br/>
    /// To resolve it and upgrade the server a Pub/Sub mechanism can be used to handle the ultra-high throughput requirements
    socket.on("message", |_socket: SocketRef, Data::<GeneralRequest>(data), socket_state: State<Arc<SocketState>>| async move {
        info!("Message: {:?}", data);
        let response = GeneralResponse {
            room: data.room.clone(),
            message: format!("Message By Client: {}", data.message).to_owned(),
            date_time: chrono::Utc::now(),
        };

        // INSERT THE MESSAGE INTO DB
        socket_state.insert(&data.room, Message {
            room: data.room.clone(),
            message: data.message.clone(),
            date_time: response.date_time.clone(),
        }).await;

        _socket.within(data.room.clone()).emit("response", response).ok();
    });
}
