mod db;
mod handlers;
mod models;
mod utils;

use handlers::user_handler::handle_connection;
use std::net::TcpListener;
use std::thread;

fn main() {
    if let Err(e) = db::connection::setup() {
        eprintln!("Erro ao configurar o banco de dados: {}", e);
        return;
    }

    let listener = TcpListener::bind("0.0.0.0:8080").expect("Não foi possível abrir a porta 8080");
    println!("Servidor rodando em http://localhost:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Erro na conexão: {}", e);
            }
        }
    }
}
