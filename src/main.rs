

extern crate env_logger;
use client::ws_con::ws_connect;



#[tokio::main] 
async fn main() {
    // Setup logging
    env_logger::init();
    //启动websocket
    ws_connect().await;
   
}
