use std::pin::Pin;

use super::connection_info::ConnectionInfo;
use futures::Stream;

pub trait PeerDiscovery {
    type Info: ConnectionInfo;
    fn discover_peers(
        &self,
    ) -> impl std::future::Future<Output = Pin<Box<dyn Stream<Item = Self::Info> + Send + Sync>>> + Send;
}
