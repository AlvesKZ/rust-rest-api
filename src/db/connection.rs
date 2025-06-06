use postgres::{Client, NoTls, Error};

const DB_URL: &str = env!("DATABASE_URL");

pub fn get_db_client() -> Result<Client, Error> {
    Client::connect(DB_URL, NoTls)
}

pub fn setup() -> Result<(), Error> {
    let mut client = get_db_client()?;
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE
        )",
        &[],
    )?;
    Ok(())
}
