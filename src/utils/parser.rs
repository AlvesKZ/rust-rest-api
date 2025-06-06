use crate::models::user::User;

pub fn get_id_from_request(request: &str) -> Result<i32, std::num::ParseIntError> {
    request
        .split('/')
        .nth(2)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
}

pub fn parse_user_body(request: &str) -> Result<User, serde_json::Error> {
    let body = request.split("\r\n\r\n").last().unwrap_or("");
    serde_json::from_str(body)
}
