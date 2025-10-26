mod network;
mod filesystem;
mod session;
mod utils;

use network::{Server};
use std::io::Result;
use crate::session::Session;

pub fn main() -> Result<()> {
    let server = Server::new("1998")?;
    loop {
        let client = server.accept()?;
        let mut session = Session::new(client, 1);
        session.start();
        break;
    }

    Ok(())
}