use std::{error::Error, sync::Arc};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::sync::{Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

use crate::{true_gear_message};

#[derive(Clone)]
pub struct TrueGearWebsocketClient {
    url: String,
    sender_stream: Arc<Mutex<Option<SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>>>>,
}

impl TrueGearWebsocketClient {
    pub fn new(url: String) -> Self {
        Self { 
            url, 
            sender_stream: Arc::new(Mutex::new(None))
        }
    }

    async fn ensure_connected(&mut self) -> Result<(), Box<dyn Error>> {

        // Aquire lock to check if already connected
        let mut write_stream_guard = self.sender_stream.lock().await;

        if write_stream_guard.is_some() {
            return Ok(());
        }
        
        // Not connected, establish connection
        let ws_result = tokio_tungstenite::connect_async(&self.url).await;

        // Handle connection result
        let ws_stream = match ws_result {
            Err(e) => {
                return Err(Box::new(e));
            },
            Ok((ws_stream, _)) => {
                ws_stream
            }
        };
        
        // Split the WebSocket stream into write and read halves
        let (write_stream, mut read_stream) = ws_stream.split();

        // Spawn a task to read messages and handle disconnection
        let sender_stream_clone = self.sender_stream.clone();
        tokio::spawn(async move {
            while let Some(msg) = read_stream.next().await {
                tracing::debug!("Received WebSocket message {:?}", msg);
                if let Result::Ok(Message::Close(_)) = msg {
                    tracing::info!("WebSocket closed by server");
                    break;
                }
            }
            
            // disconnect and drop the sender
            let sender = sender_stream_clone.lock().await.take();
            let Some(mut sender) = sender else {
                return;
            };
            
            let _ = sender.send(Message::Close(None)).await;
            tracing::info!("WebSocket session closed");
        });

        // Store the write half in the struct
        *write_stream_guard = Some(write_stream);

        tracing::info!("WebSocket connected to {}", self.url);

        Ok(())
    }

    async fn send_text(&mut self, text: String) -> Result<(), Box<dyn Error>> {
        
        // Ensure connection is established
        self.ensure_connected().await?;

        // Acquire lock to send message
        let mut sender_guard = self.sender_stream.lock().await;
        let sender = sender_guard.as_mut();

        // check if sender is available
        let Some(sender) = sender else {
            tracing::error!("WebSocket sender not available");
            return Err("WebSocket sender not available".into());
        };

        // Send the text message
        sender.send(Message::Text(text.into())).await?;
        return Ok(());
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

    pub async fn close(&mut self) {
        let mut sender_guard = self.sender_stream.lock().await;

        if let Some(sender) = sender_guard.as_mut() {
            let _ = sender.send(Message::Close(None)).await;
        }
        *sender_guard = None;
    }

}