use std::sync::Arc;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef, State};
use tracing::info;
use crate::socket_handlers::{handle_disconnect, handle_join_room, handle_message, handle_private};
use crate::socket_state::SocketState;

/// todo: INITIALIZE THE SOCKET IDS INTO A VARIABLE PAIRED TO A USERNAME <br/>
/// and store the list of key value pair in the memory store or in the DB
pub async fn on_connect(socket: SocketRef, socket_state: State<Arc<SocketState>>) {
    info!("Socket Connected: {:?}", socket.id);

    // JOIN THE SOCKET ID TO THE ROOM WITH THE SOCKET ID AS THE ROOM NAME
    socket.join(socket.id.clone()).ok();

    // generator code kept in one line else prone to MessageHandler Errors
    let name = names::Generator::default().next().unwrap();

    let mut _socket_map = socket_state.socket_map.write().await;
    _socket_map.insert(socket.id.clone().to_string(), name.clone());
    // self.socket_map.write().await.insert(name.clone(), socket_id.clone());

    socket_state.db.insert_socket_name(name.clone(), socket.id.clone().to_string()).await.unwrap();

    // The first and foremost event to be called when the socket is connected
    // socket.on("default", handle_default);

    socket.on("join_room", handle_join_room);

    socket.on("private", handle_private);

    socket.on("message", handle_message);

    socket.on("remove", handle_disconnect);

    // todo: need to store a reverse socket id map i.e, Map(socket-id,name) to get the name of the socket id and remove from memory
    socket.on("get_sockets", |socket_ref: SocketRef, Data::<Value>(data), socket_state: State<Arc<SocketState>>| async move {
        let socket_m = socket_state.socket_map.read().await;
        let mut vec = String::from("sockets:");
        socket_m.iter().for_each(|(_, v)| {

            vec.push_str(String::from(v.clone()+",").as_str());

        });
        socket_ref.emit("sockets", vec).ok();
    });
}
