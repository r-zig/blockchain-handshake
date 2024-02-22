// lib.rs
pub mod bitcoin;
pub mod protocols;

#[cfg(test)]
mod tests {
    use crate::{bitcoin::bitcoin_factory::BitcoinPeerFactory, protocols::peer::Peer};
    use tracing::{info, warn};
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn bitcoin_handshake_single_peer() -> Result<(), Box<dyn std::error::Error>> {
        info!("This is being logged on the info level");
        let _ = tracing_subscriber::fmt::try_init();

        // set environment variables to point to the well known peer address
        let remote_peer_address = "127.0.0.1:8333";
        // Set up the environment variable
        std::env::set_var("REMOTE_PEER_ADDRESS", remote_peer_address);

        // create bitcoin peer using the factory
        let mut local_peer = BitcoinPeerFactory::new_peer();

        // connect to the peer
        local_peer.connect().await?;

        // // verify that the connection established correctly
        // todo!("how to verify?")
        // // assert_eq!(result, 4);
        warn!("This is being logged on the warn level");
        Ok(())
    }
}
