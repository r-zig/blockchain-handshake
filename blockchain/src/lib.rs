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
        // set environment variables to point to the well known peer address
        // try get address from https://bitnodes.io/nodes/?q=Satoshi:26.0.0
        let remote_peer_address = "49.13.129.99:8333";
        let user_agent = "my test user agent";
        // Set up the environment variable
        std::env::set_var("REMOTE_PEER_ADDRESS", remote_peer_address);
        std::env::set_var("USER_AGENT", user_agent);

        // create bitcoin peer using the factory
        let mut local_peer = BitcoinPeerFactory::new_peer();

        // connect to the peer
        local_peer.connect().await?;

        // // verify that the connection established correctly
        // todo!("how to verify?")
        // // assert_eq!(result, 4);
        Ok(())
    }
}
