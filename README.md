# Simple fileserver on Rust
## This project is abandoned
You can finish or modify this project if you want. \
But I don't want to finish it in Rust, or I will. \
I'm planning to rewrite this project in another language, but not right now.

## TODO
- Checking paths;
- Limit the size of some fields data;
- Guarantee delivery via UDP;
- The initial handshake as part of the protocol;
- Multi-client system;
- Forwarding a client to another port;
- Advanced cypher system(for each client its own key)

## How to run it?
1. Clone this project and enter the directory
2. Use ```cargo run``` or ```cargo build```
3. Run via ```.\target\debug\fileserver```