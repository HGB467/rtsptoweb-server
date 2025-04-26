mod streamer;
mod stream_manager;
mod structures;

use std::collections::HashMap;
use std::sync::Arc;
use axum::http::StatusCode;
use axum::{Router};
use axum::routing::{delete, get, get_service, post};
use gst_plugin_webrtc_signalling::handlers::Handler;
use gst_plugin_webrtc_signalling::server::Server;
use gstreamer as gst;
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use stream_manager::{add_stream,get_streams,delete_stream};
use structures::RtspStream;
use tokio::net::TcpListener;
use tokio::task;


#[tokio::main]
async fn main() {
    gst::init().unwrap();

    let streams : Arc<Mutex<HashMap<String, RtspStream>>> = Arc::new(Mutex::new(HashMap::new()));

    let serve_dir = get_service(ServeDir::new("./hls").append_index_html_on_directories(false))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Filesystem error: {}", error),
                )
            });

        let app = Router::new()
            .nest_service("/", serve_dir)
            .route("/addStream", post(add_stream))
            .route("/getStreams", get(get_streams))
            .route("/deleteStream", delete(delete_stream))
            .with_state(streams.clone())
            .layer(tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
            );

    let listener = TcpListener::bind("127.0.0.1:5005").await.unwrap();

    let server_signalling = Server::spawn(Handler::new);

    let addr_signalling = format!("{}:{}", "127.0.0.1", "8443");

    let listener_signalling = TcpListener::bind(&addr_signalling).await.unwrap();

    println!("Server started successfully!");

    tokio::select! {
        _ = axum::serve(listener, app) => {
            println!("HTTP server stopped");
        }
        _ = async {
            while let Ok((stream, _address)) = listener_signalling.accept().await {
                let mut server_clone = server_signalling.clone();
                task::spawn(async move {
                    if let Err(e) = server_clone.accept_async(stream).await {
                        eprintln!("Error accepting connection: {}", e);
                    }
                });
            }
        } => {}
    }
}
