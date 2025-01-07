use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net::TcpListener;

use crate::player::net::RawConnection;

pub mod builder;

pub struct Server {}

impl Server {
    pub async fn start(self) {
        let listener = TcpListener::bind(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565)).await.unwrap();

        println!("Server now listening on 127.0.0.1:25565");
        loop {
            println!("iter");
            let new_client = listener.accept().await;
            match new_client {
                Ok((stream, addr)) => {
                    println!("Accepted new client: {:?}", addr);
                    tokio::spawn(RawConnection::execute_connection(
                        stream, 
                        addr.ip()
                    ));
                },
                Err(_err) => {},
            }
        }
    }
}