mod db;
mod handlers;
mod models;
mod utils;

use handlers::user_handler::handle_connection;
use std::net::TcpListener;

fn main() {
    if let Err(e) = db::connection::setup() {
        eprintln!("Database setup error: {}", e);
        return;
    }

    let listener = TcpListener::bind("0.0.0.0:8080").expect("Could not bind to port 8080");
    println!("Server running on port 8080");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}
