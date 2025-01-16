use std::sync::Arc;

use tokio::net::TcpListener;

pub struct ProxyNetworking {
    listener: TcpListener
}

pub struct ProxyNetworkingRef {
    inner: Arc<ProxyNetworking>
}