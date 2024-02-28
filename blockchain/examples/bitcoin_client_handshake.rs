use blockchain::bitcoin::bitcoin_factory::BitcoinPeerFactory;
use blockchain::bitcoin::BitcoinConfiguration;
use blockchain::protocols::peer::Peer;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt::init();
    let config = BitcoinConfiguration::parse();
    let connected_address = config.discover_remote_peer_address;
    // create bitcoin peer using the factory
    let mut local_peer = BitcoinPeerFactory::new_peer(config);

    // connect to the peer
    local_peer.connect().await.unwrap();

    println!("Connected successfully to {:?}", connected_address);
    Ok(())
}
