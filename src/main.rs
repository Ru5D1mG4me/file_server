mod network;
mod filesystem;
mod session;
mod utils;
mod cypher;

use network::{Server};
use std::io::Result;
use session::Session;
use cypher::Cypher;

pub fn main() -> Result<()> {
    let key_str = "SUPER_SECRET_KEY1125133111444411";
    let key = match <&[u8; 32]>::try_from(key_str.as_bytes()) {
        Ok(key) => key,
        Err(_) => panic!("Key is invalid"),
    };
    let cypher = Cypher::new(key);
    let server = Server::new("1998")?;
    loop {
        let client = server.accept()?;
        let mut session = Session::new(client, 1, cypher);
        session.start();
        break;
    }

    Ok(())
}