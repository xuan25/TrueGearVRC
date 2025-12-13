use std::{error::Error, time::Duration};
use crate::{mapping::SharedState, websocket::TrueGearWebsocketClient};

#[derive(Clone)]
pub struct Sender {
    true_gear_websocket: crate::websocket::TrueGearWebsocketClient,
    shared_state: SharedState,
    shake_intensity: u16,
    electrical_intensity: u16,
    electrical_interval: u8,
}

impl Sender {
    pub fn new(
        true_gear_websocket: crate::websocket::TrueGearWebsocketClient,
        shared_state: SharedState,
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
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let maybe_effect = self.shared_state
                .clone()
                .build_effect(self.shake_intensity, self.electrical_intensity, self.electrical_interval)
                .await;

            if let Some(effect) = maybe_effect {
                
                // ignore send errors (reconnect will happen on next send)
                let _ = self.true_gear_websocket.send_play_effect(&effect).await;
            }
        }
    }

    pub async fn build (
        truegear_ws_url: String,
        shared_state: SharedState,
        shake_intensity: u16,
        electrical_intensity: u16,
        electrical_interval: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let mut true_gear_websocket = TrueGearWebsocketClient::new(truegear_ws_url);
        true_gear_websocket.start().await?;
        Ok(Self::new(
            true_gear_websocket,
            shared_state,
            shake_intensity,
            electrical_intensity,
            electrical_interval,
        ))
    }

    pub fn close(&mut self) {
        // Currently, tokio-tungstenite does not provide a direct method to close the connection.
        // However, dropping the WebSocket client will close the connection.
        self.true_gear_websocket.close();
    }
}
