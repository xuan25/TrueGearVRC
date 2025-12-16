use crate::{mapping::ProtocalMapper, websocket::TrueGearWebsocketClient};
use std::{error::Error, time::Duration};

#[derive(Clone)]
pub struct Sender {
    true_gear_websocket: crate::websocket::TrueGearWebsocketClient,
    shared_state: ProtocalMapper,
    shake_intensity: u16,
    electrical_intensity: u16,
    electrical_interval: u8,
}

impl Sender {
    pub fn new(
        true_gear_websocket: crate::websocket::TrueGearWebsocketClient,
        shared_state: ProtocalMapper,
        shake_intensity: u16,
        electrical_intensity: u16,
        electrical_interval: u8,
    ) -> Self {
        Self {
            true_gear_websocket,
            shared_state,
            shake_intensity,
            electrical_intensity,
            electrical_interval,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Start the WebSocket connection; ignore errors here (reconnect will happen on send)
        if let Err(e) = self.true_gear_websocket.start().await {
            tracing::warn!("WebSocket connection error: {}", e);
        }
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let maybe_effect = self
                .shared_state
                .clone()
                .build_effect(
                    self.shake_intensity,
                    self.electrical_intensity,
                    self.electrical_interval,
                )
                .await;

            if let Some(effect) = maybe_effect {
                // ignore send errors (reconnect will happen on next send)
                if let Err(e) = self.true_gear_websocket.send_play_effect(&effect).await {
                    tracing::error!("WebSocket connection error: {}", e);
                }
            }
        }
    }

    pub async fn build(
        truegear_ws_url: String,
        shared_state: ProtocalMapper,
        shake_intensity: u16,
        electrical_intensity: u16,
        electrical_interval: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let true_gear_websocket = TrueGearWebsocketClient::new(truegear_ws_url);
        Ok(Self::new(
            true_gear_websocket,
            shared_state,
            shake_intensity,
            electrical_intensity,
            electrical_interval,
        ))
    }

    pub async fn close(&mut self) {
        // Currently, tokio-tungstenite does not provide a direct method to close the connection.
        // However, dropping the WebSocket client will close the connection.
        self.true_gear_websocket.close().await;
    }
}
