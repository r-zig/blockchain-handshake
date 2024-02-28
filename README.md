# blockchain-handshake
# Prerequisites
Before running this example, you must obtain a modified version of another Bitcoin implementation. For this example, we use the Rust Bitcoin library.

## Fork and Modify the Rust Bitcoin Library

The original Rust Bitcoin library can be found at: https://github.com/rust-bitcoin/rust-bitcoin
However, to enable the code to run also as a server peer, a modified version is required. Clone the forked repository:

```bash Copy code
git clone https://github.com/r-zig/rust-bitcoin
```
Then, to run the modified library, execute the following command:

```bash Copy code
cargo run --example handshake --features="std rand-std" -S 127.0.0.1:8333
```
# Getting Started
Obtain the custom Rust implementation by cloning the following repository:

```bash Copy code
git clone https://github.com/r-zig/blockchain-handshake.git
```
Run the implementation using the command below. If the connection is successful, it will print "Connected successfully to 127.0.0.1:8333".

```bash Copy code
RUST_LOG=info cargo run --example bitcoin_client_handshake -- -A 127.0.0.1:8333
```
## Connecting to Public Bitcoin Nodes
You can attempt to connect to other public Bitcoin nodes using the following resource:

https://bitnodes.io/nodes
While most peers connect successfully, some may fail due to the "sendcmpct" command being sent instead of "verack". Further investigation into the Bitcoin protocol is necessary to determine if this behavior is valid.

## Using the command line args or environment variables:
-A or --remote-address, or the DISCOVER_REMOTE_PEER_ADDRESS environment variable, is used to set the address of the remote node.
-U or --user-agent, or the USER_AGENT environment variable, is used to set the user agent string exchanged between nodes.

## Limitations
The current implementation does not discover multiple Bitcoin nodes; it only attempts to connect to a single node.
Checksum validation is not yet implemented.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgment

If you use this code or a modified version of it in your project, please credit me as a contributor by mentioning my GitHub username or linking back to the original repository. This acknowledgment is not a condition of the license but a request to foster collaboration and recognition among developers.

