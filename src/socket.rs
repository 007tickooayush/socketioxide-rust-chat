use socketioxide::extract::SocketRef;
use tracing::info;
use crate::socket_handlers::{handle_join_room, handle_message, handle_private};

/// todo: INITIALIZE THE SOCKET IDS INTO A VARIABLE PAIRED TO A USERNAME <br/>
/// and store the list of key value pair in the memory store or in the DB
pub async fn on_connect(socket: SocketRef, ) {
    info!("Socket Connected: {:?}", socket.id);

    socket.join(socket.id.clone()).ok();

    socket.on("join_room",handle_join_room);

    socket.on("private", handle_private);

    socket.on("message", handle_message);
}
