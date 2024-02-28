# blockchain-handshake
Getting Started
Prerequisites
Before running this example, you must obtain a modified version of another Bitcoin implementation. For this example, we use the Rust Bitcoin library.

Fork and Modify the Rust Bitcoin Library

The original Rust Bitcoin library can be found at:

Copy code
https://github.com/rust-bitcoin/rust-bitcoin
However, to enable the code to run also as a server peer, a modified version is required. Clone the forked repository:

Copy code
https://github.com/r-zig/rust-bitcoin
Then, to run the modified library, execute the following command:

Copy code
cargo run --example handshake --features="std rand-std" -S 127.0.0.1:8333
Clone and Run the Custom Rust Implementation

Obtain the custom Rust implementation by cloning the following repository:

Copy code
https://github.com/r-zig/blockchain-handshake.git
Run the implementation using the command below. If the connection is successful, it will print "Connected successfully to 127.0.0.1:8333".

Copy code
RUST_LOG=info cargo run --example bitcoin_client_handshake -- -A 127.0.0.1:8333
The current implementation does not discover multiple Bitcoin nodes; it only attempts to connect to a single node via command line or an environment variable named DISCOVER_REMOTE_PEER_ADDRESS.

Another environment variable, USER_AGENT, is used to set the user agent string exchanged between peers. You can also specify the remote peer address and user agent from the CLI using the -A and -U arguments, respectively.

Connecting to Public Bitcoin Nodes
You can attempt to connect to other public Bitcoin nodes using the following resource:

Copy code
https://bitnodes.io/nodes
While most peers connect successfully, some may fail due to the "sendcmpct" command being sent instead of "verack". Further investigation into the Bitcoin protocol is necessary to determine if this behavior is valid.
