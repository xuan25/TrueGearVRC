use std::error::Error;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::{tungstenite::protocol::Message};

use crate::true_gear_message;

#[derive(Clone)]
pub struct TrueGearWebsocketClient {
    url: String,
    sender: Option<UnboundedSender<Message>>,
}

impl TrueGearWebsocketClient {
    pub fn new(url: String) -> Self {
        Self { 
            url, 
            sender: None
        }
    }

    async fn ensure_connected(&mut self) -> Result<(), Box<dyn Error>> {

        if self.sender.is_some() {
            return Ok(());
        }
        
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url).await.expect("Failed to connect");
        let (mut write_stream, _) = ws_stream.split();

        let (tx, mut rx) = mpsc::unbounded_channel();
        self.sender = Some(tx);

        // Spawn a task to forward messages from rx to write_stream
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let _ = write_stream.send(msg.clone()).await;
                tracing::info!("Sent WebSocket message {:?}", msg);
            }
        });
        Ok(())
    }

    async fn send_text(&mut self, text: String) -> Result<(), Box<dyn Error>> {

        self.ensure_connected().await?;

        if let Some(sender) = &self.sender {
            sender.send(Message::Text(text.into())).map_err(|e| Box::new(e) as Box<dyn Error>)?;
            Ok(())
        } else {
            tracing::error!("WebSocket sender not available");
            Err("WebSocket sender not available".into())
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.ensure_connected().await?;
        Ok(())
    }

    pub async fn send_play_effect(&mut self, effect: &true_gear_message::Effect) -> Result<(), Box<dyn Error>> {
        let cmd = true_gear_message::Message {
            method: "play_no_registered".to_string(),
            body: effect.clone(),
        };

        let cmd_text = serde_json::to_string(&cmd)?;

        self.send_text(cmd_text).await
    }

    pub fn close(&mut self) {
        // Currently, tokio-tungstenite does not provide a direct method to close the connection.
        // However, dropping the WebSocket client will close the connection.
        self.sender = None;
    }

}