use postgres::{Client, NoTls, Error as PostgresError};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

const DB_URL: &str = env!("DATABASE_URL");

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

fn main() {
    if let Err(e) = set_database() {
        println!("Error: {}", e);
        return;
    }

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server running on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request = String::from_utf8_lossy(&buffer[..size]).to_string();

            let (status_line, content) = if request.starts_with("POST /users") {
                handle_post_request(&request)
            } else if request.starts_with("GET /users/") {
                handle_get_request(&request)
            } else if request.starts_with("GET /users") {
                handle_get_all_request(&request)
            } else if request.starts_with("PUT /users") {
                handle_put_request(&request)
            } else if request.starts_with("DELETE /users") {
                handle_delete_request(&request)
            } else {
                (NOT_FOUND.to_string(), "Not Found".to_string())
            };

            if let Err(e) = stream.write_all(format!("{}{}", status_line, content).as_bytes()) {
                eprintln!("Write error: {}", e);
            }
        }
        Err(e) => {
            println!("Read error: {}", e);
        }
    }
}

fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(request), Client::connect(DB_URL, NoTls)) {
        (Ok(user), Ok(mut client)) => {
            if let Err(_) = client.execute(
                "INSERT INTO users (name, email) VALUES ($1, $2)",
                &[&user.name, &user.email],
            ) {
                return (INTERNAL_SERVER_ERROR.to_string(), "Insert failed".to_string());
            }

            (OK_RESPONSE.to_string(), "User created".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

fn handle_get_request(request: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            match client.query("SELECT id, name, email FROM users WHERE id = $1", &[&id]) {
                Ok(rows) if !rows.is_empty() => {
                    let row = &rows[0];
                    let user = User {
                        id: Some(row.get(0)),
                        name: row.get(1),
                        email: row.get(2),
                    };
                    (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "User not found".to_string()),
            }
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(DB_URL, NoTls) {
        Ok(mut client) => {
            let query_result = client.query("SELECT id, name, email FROM users", &[]);

            match query_result {
                Ok(rows) => {
                    let users: Vec<User> = rows
                        .into_iter()
                        .map(|row| User {
                            id: Some(row.get(0)),
                            name: row.get(1),
                            email: row.get(2),
                        })
                        .collect();

                    (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
                }
                Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Query error".to_string()),
            }
        }
        Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Connection error".to_string()),
    }
}

fn handle_put_request(request: &str) -> (String, String) {
    match (
        get_id(request).parse::<i32>(),
        get_user_request_body(request),
        Client::connect(DB_URL, NoTls),
    ) {
        (Ok(id), Ok(user), Ok(mut client)) => {
            if let Err(_) = client.execute(
                "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                &[&user.name, &user.email, &id],
            ) {
                return (INTERNAL_SERVER_ERROR.to_string(), "Update failed".to_string());
            }

            (OK_RESPONSE.to_string(), "User updated".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

fn handle_delete_request(request: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            match client.execute("DELETE FROM users WHERE id = $1", &[&id]) {
                Ok(rows_affected) => {
                    if rows_affected == 0 {
                        return (NOT_FOUND.to_string(), "User not found".to_string());
                    }
                    (OK_RESPONSE.to_string(), "User deleted".to_string())
                }
                Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Delete failed".to_string()),
            }
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

fn set_database() -> Result<(), PostgresError> {
    let mut client = Client::connect(DB_URL, NoTls)?;
    client.execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE
        )",
        &[],
    )?;
    Ok(())
}

fn get_id(request: &str) -> &str {
    request
        .split('/')
        .nth(2)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
