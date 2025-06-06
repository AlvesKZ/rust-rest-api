use crate::models::user::User;
use std::num::ParseIntError;

pub fn get_id_from_request(request: &str) -> Result<i32, ParseIntError> {
    let path_line = request.lines().next().unwrap_or("");
    
    let parts: Vec<&str> = path_line.split_whitespace().collect();
    if parts.len() < 2 {
        return "0".parse(); 
    }

    let path = parts[1]; 
    let id_str = path.split('/').nth(2).unwrap_or("");
    id_str.parse()
}

pub fn parse_user_body(request: &str) -> Result<User, serde_json::Error> {
    let body = request.split("\r\n\r\n").nth(1).unwrap_or("").trim();
    serde_json::from_str(body)
}
