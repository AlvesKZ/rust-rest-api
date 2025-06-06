use crate::db::connection::get_db_client;
use crate::models::user::User;
use crate::utils::parser::{get_id_from_request, parse_user_body};
use std::io::{Read, Write};
use std::net::TcpStream;

const OK: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const ERR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Ok(size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..size]);

        let (status, body) = match request.as_ref() {
            r if r.starts_with("POST /users") => handle_post(r),
            r if r.starts_with("GET /users/") => handle_get(r),
            r if r.starts_with("GET /users") => handle_get_all(),
            r if r.starts_with("PUT /users") => handle_put(r),
            r if r.starts_with("DELETE /users") => handle_delete(r),
            _ => (NOT_FOUND, "Route not found".to_string()),
        };

        let _ = stream.write_all(format!("{}{}", status, body).as_bytes());
    }
}

fn handle_post(request: &str) -> (&'static str, String) {
    match (parse_user_body(request), get_db_client()) {
        (Ok(user), Ok(mut client)) => {
            match client.execute(
                "INSERT INTO users (name, email) VALUES ($1, $2)",
                &[&user.name, &user.email],
            ) {
                Ok(_) => (OK, "User created".to_string()),
                Err(_) => (ERR, "Failed to insert user".to_string()),
            }
        }
        _ => (ERR, "Failed to parse or connect".to_string()),
    }
}

fn handle_get(request: &str) -> (&'static str, String) {
    match (get_id_from_request(request), get_db_client()) {
        (Ok(id), Ok(mut client)) => {
            match client.query_one("SELECT id, name, email FROM users WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let user = User {
                        id: Some(row.get(0)),
                        name: row.get(1),
                        email: row.get(2),
                    };
                    match serde_json::to_string(&user) {
                        Ok(json) => (OK, json),
                        Err(_) => (ERR, "Serialization error".to_string()),
                    }
                }
                Err(_) => (NOT_FOUND, "User not found".to_string()),
            }
        }
        _ => (ERR, "Error fetching user".to_string()),
    }
}

fn handle_get_all() -> (&'static str, String) {
    if let Ok(mut client) = get_db_client() {
        if let Ok(rows) = client.query("SELECT id, name, email FROM users", &[]) {
            let users: Vec<User> = rows
                .into_iter()
                .map(|row| User {
                    id: Some(row.get(0)),
                    name: row.get(1),
                    email: row.get(2),
                })
                .collect();
            return match serde_json::to_string(&users) {
                Ok(json) => (OK, json),
                Err(_) => (ERR, "Serialization error".to_string()),
            };
        }
    }
    (ERR, "Error listing users".to_string())
}

fn handle_put(request: &str) -> (&'static str, String) {
    match (get_id_from_request(request), parse_user_body(request), get_db_client()) {
        (Ok(id), Ok(user), Ok(mut client)) => {
            match client.execute(
                "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                &[&user.name, &user.email, &id],
            ) {
                Ok(updated) => {
                    if updated > 0 {
                        (OK, "User updated".to_string())
                    } else {
                        (NOT_FOUND, "User not found".to_string())
                    }
                }
                Err(_) => (ERR, "Failed to update".to_string()),
            }
        }
        _ => (ERR, "Invalid input or connection error".to_string()),
    }
}

fn handle_delete(request: &str) -> (&'static str, String) {
    match (get_id_from_request(request), get_db_client()) {
        (Ok(id), Ok(mut client)) => {
            match client.execute("DELETE FROM users WHERE id = $1", &[&id]) {
                Ok(count) => {
                    if count > 0 {
                        (OK, "User deleted".to_string())
                    } else {
                        (NOT_FOUND, "User not found".to_string())
                    }
                }
                Err(_) => (ERR, "Failed to delete user".to_string()),
            }
        }
        _ => (ERR, "Invalid input or connection error".to_string()),
    }
}
