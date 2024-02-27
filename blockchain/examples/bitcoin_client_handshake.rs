use blockchain::bitcoin::bitcoin_factory::BitcoinPeerFactory;
use blockchain::protocols::peer::Peer;
use clap::Parser;
use std::net::SocketAddr;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The address of the remote peer to connect to. Can be set via CLI or the REMOTE_PEER_ADDRESS env variable.
    #[clap(long)]
    address: SocketAddr,

    #[clap(long)]
    user_agent: String,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    // Set up the environment variable
    std::env::set_var("REMOTE_PEER_ADDRESS", cli.address.to_string());
    std::env::set_var("USER_AGENT", cli.user_agent);

    // create bitcoin peer using the factory
    let mut local_peer = BitcoinPeerFactory::new_peer();

    // connect to the peer
    local_peer.connect().await.unwrap();
    Ok(())
}
