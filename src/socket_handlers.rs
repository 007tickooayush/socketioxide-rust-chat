use std::sync::Arc;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::info;
use crate::model::{GeneralRequest, GeneralResponse, Message, PrivateMessageReq, User};
use crate::socket_state::SocketState;
use crate::model::Messages;

// Created a separate module for the socket handlers
// because the socket event handlers are actually expressions and not functions
// so for the sake of better understanding and readability, we have created a separate module for the socket handler and
// have handlers as functions not an expression, in this module

/// Join a room and save the state of the room up to the message limit defined in the
/// `get_messages` function <br/>
/// *NOTE* The message reading is not being performed from the DB and is being read from the memory store
/// provided by the `SocketState` struct implementation through `socketioxide` >v8.0 library
pub async fn handle_join_room(_socket: SocketRef, Data(data): Data<GeneralRequest>, socket_state: State<Arc<SocketState>>) {
    let general = GeneralRequest {
        sender: data.sender.clone(),
        room: data.room.clone(),
        message: data.message.clone(),
    };
    info!("General: {:?}", &general);

    // _socket.leave_all().ok();

    // Leave all the rooms except the username group room for private chats
    socket_state.leave_all_expect_one(_socket.clone(), data.sender.clone()).await;

    _socket.join(general.room.clone()).ok();
    let messages = socket_state.get_messages(&general.room).await;


    // let response = GeneralResponse {
    //     sender: general.sender.clone(),
    //     room: general.room.clone(),
    //     message: format!("Room joined by client: {}", _socket.id).to_owned(),
    //     date_time: chrono::Utc::now(),
    // };

    // _socket.within(general.room.clone()).emit("response", response).ok();

    _socket.emit("messages", Messages { messages }).ok();
}

pub async fn handle_private(_socket: SocketRef, Data(data): Data<PrivateMessageReq>, socket_state: State<Arc<SocketState>>) {
    info!("Private: {:?}", data);
    let sender = match data.sender.clone() {
        Some(sender) => sender,
        None => _socket.id.clone().to_string()
    };
    let response = PrivateMessageReq {
        message: data.message.to_owned(),
        sender: Some(sender),
        receiver: data.receiver.clone(),
    };

    // INSERT THE MESSAGE INTO DB
    socket_state.insert_private_messages(response.clone()).await;

    _socket.to(data.receiver.clone()).emit("resp", response).ok();
}


/// Handling the message from the client <br/>
/// *NOTE:* The mechanism is not built for Ultra high throughput as OPS limit is not set and may exceed
/// if too many write operations are performed simultaneously <br/>
/// To resolve it and upgrade the server a Pub/Sub mechanism can be used to handle the ultra-high throughput requirements
pub async fn handle_message(_socket: SocketRef, Data(data): Data<GeneralRequest>, socket_state: State<Arc<SocketState>>) {
    info!("Message: {:?}", data);
    let response = GeneralResponse {
        sender: data.sender.clone(),
        room: data.room.clone(),
        // message: format!("Message By Client: {}", data.message).to_owned(),
        message: data.message.clone(),
        date_time: chrono::Utc::now(),
    };

    // INSERT THE MESSAGE INTO DB
    socket_state.insert(&data.room, Message {
        sender: data.sender.clone(),
        room: data.room.clone(),
        message: data.message.clone(),
        date_time: response.date_time.clone(),
    }).await;

    _socket.within(data.room.clone()).emit("response", response).ok();
}

/// Handle the user linking to the generated unique username <br/>
/// Enable the user to map its owned username to the generated username, which is mapped with socket id and changes with each connection
pub async fn handle_user_join(_socket: SocketRef, Data(data): Data<User>, socket_state: State<Arc<SocketState>>) {

    info!("User Join: {:?}", data);
    let user_resp = socket_state.handle_user(data.clone()).await;

    info!("User Join Own Private: {:?}", data.username.clone());
    // info: Only provided as a patch functionality
    _socket.join(data.username).ok();

    _socket.emit("user_handled", user_resp).ok();


//     let response = User {
//         username: data.username.clone(),
//         generated_username: data.generated_username.clone(),
//     };
//
//     // INSERT THE MESSAGE INTO DB
//     // socket_state.insert(&data.room, Message {
//     //     sender: data.sender.clone(),
//     //     room: data.room.clone(),
//     //     message: data.message.clone(),
//     //     date_time: response.date_time.clone(),
//     // }).await;
//
//     _socket.emit("user_joined", response).ok();
}
pub async fn handle_removal(_socket: SocketRef, Data(data): Data<GeneralRequest>, socket_state: State<Arc<SocketState>>) {
    info!("Disconnect: {:?}", data);
    // let _ = socket_state.remove_socket(_socket.id.clone().to_string()).await;
    let _ = socket_state.remove_socket(data.sender.clone()).await;

    // Will not send the data to the client as the client is already disconnected
    // _socket.emit("removed", GeneralResponse {
    //     sender: data.sender.clone(),
    //     room: data.room.clone(),
    //     message: format!("User: {} has been removed from the room", data.sender).to_owned(),
    //     date_time: chrono::Utc::now(),
    // }).ok();
    _socket.disconnect().ok();
}

pub fn handle_disconnect_socket(_socket: SocketRef, _socket_state: State<Arc<SocketState>>) {
    _socket.leave_all().ok();
    info!("Socket Disconnected: {:?}", _socket.id);
}

// /// The first and foremost event to be called when the socket is connected
// ///in order to create the map for usernames and socket id
// pub async fn handle_default(_socket: SocketRef, socket_state: State<Arc<SocketState>>) {
//     let (name, socket) = socket_state.insert_socket_name(_socket.id.clone().to_string()).await;
//     _socket.emit("name", json!({ "name": name, "socket": socket })).ok();
// }