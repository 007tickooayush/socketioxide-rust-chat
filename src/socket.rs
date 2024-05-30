use std::sync::Arc;
use socketioxide::extract::{SocketRef, State};
use tracing::info;
use crate::socket_handlers::{handle_default, handle_join_room, handle_message, handle_private};
use crate::socket_state::SocketState;

/// todo: INITIALIZE THE SOCKET IDS INTO A VARIABLE PAIRED TO A USERNAME <br/>
/// and store the list of key value pair in the memory store or in the DB
pub async fn on_connect(socket: SocketRef, socket_state: State<Arc<SocketState>>) {
    info!("Socket Connected: {:?}", socket.id);

    // JOIN THE SOCKET ID TO THE ROOM WITH THE SOCKET ID AS THE ROOM NAME
    socket.join(socket.id.clone()).ok();

    // The first and foremost event to be called when the socket is connected
    socket.on("default", handle_default);

    socket.on("join_room",handle_join_room);

    socket.on("private", handle_private);

    socket.on("message", handle_message);
}
