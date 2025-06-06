use postgres::{Client, NoTls, Error};
use std::env;

pub fn get_db_client() -> Result<Client, Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Client::connect(&db_url, NoTls)
}


pub fn setup() -> Result<(), Error> {
    let mut client = get_db_client()?;
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE
        )",
        &[],
    )?;
    Ok(())
}
