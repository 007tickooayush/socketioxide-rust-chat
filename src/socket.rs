use std::sync::Arc;
use socketioxide::extract::{SocketRef, State};
use tracing::info;
use crate::socket_handlers::{handle_removal, handle_join_room, handle_message, handle_private, handle_disconnect_socket, handle_user_join, handle_private_joined, handle_private_left};
use crate::socket_state::SocketState;

/// todo: INITIALIZE THE SOCKET IDS INTO A VARIABLE PAIRED TO A USERNAME <br/>
/// and store the list of key value pair in the memory store or in the DB
pub async fn on_connect(socket: SocketRef, socket_state: State<Arc<SocketState>>) {
    info!("Socket Connected: {:?}", socket.id);

    // generator code kept in one line else prone to MessageHandler Errors
    let name = names::Generator::default().next().unwrap();

    // JOIN THE USERNAME AND CREATE A PRIVATE CHAT ROOM
    // AND NEVER EXPOSE THE ACTUAL SOCKET ID OF THE USER TO THE FRONTEND
    socket.join(name.clone()).ok();
    // socket.join(socket.id.clone()).ok();

    // todo: NOT Storing the socket id and the name in the memory store
    // let mut _socket_map = socket_state.socket_map.write().await;
    // _socket_map.insert(socket.id.clone().to_string(), name.clone());
    // self.socket_map.write().await.insert(name.clone(), socket_id.clone());

    socket_state.db.insert_socket_name(name.clone(), socket.id.clone().to_string()).await.unwrap();
    info!("Generated Username (private group): {:?}", name.clone());

    socket.emit("username", name.clone()).ok();
    // The first and foremost event to be called when the socket is connected
    // socket.on("default", handle_default);

    socket.on("user_handle",handle_user_join);

    socket.on("private_joined", handle_private_joined);

    socket.on("private_left", handle_private_left);

    socket.on("join_room", handle_join_room);

    socket.on("private", handle_private);

    socket.on("message", handle_message);

    socket.on("remove", handle_removal);

    socket.on_disconnect(handle_disconnect_socket);
}
