use crate::mapping::ProtocalMapper;
use rosc::decoder;
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

#[derive(Clone)]
pub struct Reciver {
    sock: Option<Arc<UdpSocket>>,
    shared_state: ProtocalMapper,
    forward_addr: Option<SocketAddr>,
}

impl Reciver {
    pub fn new(
        sock: Arc<UdpSocket>,
        shared_state: ProtocalMapper,
        forward_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            sock: Some(sock),
            shared_state,
            forward_addr,
        }
    }

    pub async fn build(
        listening_addr: SocketAddr,
        shared_state: ProtocalMapper,
        forward_addr: Option<SocketAddr>,
    ) -> Self {
        let sock = UdpSocket::bind(listening_addr)
            .await
            .expect("Failed to bind UDP socket");
        Self::new(Arc::new(sock), shared_state, forward_addr)
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut buf = [0u8; 2048];

        loop {
            let (n, _peer) = self
                .sock
                .as_ref()
                .ok_or("Socket is closed")?
                .recv_from(&mut buf)
                .await?;

            if let Ok((_, packet)) = decoder::decode_udp(&buf[..n]) {
                {
                    self.shared_state.clone().consume_osc_packet(&packet).await;
                }

                if let Some(dst) = self.forward_addr {
                    let _ = self
                        .sock
                        .as_ref()
                        .ok_or("Socket is closed")?
                        .send_to(&buf[..n], dst)
                        .await;
                }
            }
        }
    }

    pub async fn close(&mut self) {
        // Currently, there is no explicit close method for UdpSocket in Tokio.
        // Dropping the socket will close it.

        self.sock = None;
    }
}
