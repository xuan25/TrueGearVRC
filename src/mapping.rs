use std::sync::Arc;

use rosc::{OscMessage, OscPacket, OscType};
use tokio::sync::Mutex;

use crate::true_gear_message;

/* ---------------- OSC mapping / state ---------------- */

const TARGET: [&str; 42] = [
    "TrueGearA1","TrueGearA2","TrueGearA3","TrueGearA4","TrueGearA5",
    "TrueGearB1","TrueGearB2","TrueGearB3","TrueGearB4","TrueGearB5",
    "TrueGearC1","TrueGearC2","TrueGearC3","TrueGearC4","TrueGearC5",
    "TrueGearD1","TrueGearD2","TrueGearD3","TrueGearD4","TrueGearD5",
    "TrueGearE1","TrueGearE2","TrueGearE3","TrueGearE4","TrueGearE5",
    "TrueGearF1","TrueGearF2","TrueGearF3","TrueGearF4","TrueGearF5",
    "TrueGearG1","TrueGearG2","TrueGearG3","TrueGearG4","TrueGearG5",
    "TrueGearH1","TrueGearH2","TrueGearH3","TrueGearH4","TrueGearH5",
    "TrueGearArmL","TrueGearArmR",
];

const NUMBERS: [u8; 42] = [
    1, 5, 9, 13, 17,
    0, 4, 8, 12, 16,
    100, 104, 108, 112, 116,
    101, 105, 109, 113, 117,
    102, 106, 110, 114, 118,
    103, 107, 111, 115, 119,
    3, 7, 11, 15, 19,
    2, 6, 10, 14, 18,
    0, 100,
];

#[derive(Clone)]
pub struct SharedState {
    pub percentage: Arc<Mutex<[f32; 42]>>,
    pub shake_index: Arc<Mutex<Vec<u8>>>,
    pub electrical_index: Arc<Mutex<Vec<u8>>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            percentage: Arc::new(Mutex::new([0.0; 42])),
            shake_index: Arc::new(Mutex::new(Vec::new())),
            electrical_index: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }

    fn extract_first_numeric_arg(msg: &OscMessage) -> Option<f32> {
        if msg.args.is_empty() {
            None
        } else {
            match &msg.args[0] {
                OscType::Float(f) => Some(*f),
                OscType::Double(f) => Some(*f as f32),
                OscType::Int(i) => Some(*i as f32),
                OscType::Long(i) => Some(*i as f32),
                OscType::String(s) => s.parse::<f32>().ok(),
                OscType::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            }
        }
    }

    fn scale_intensity(base: u16, factor: f32) -> u16 {
        let v = (base as f32 * factor).round();
        v.clamp(0.0, 150.0) as u16
    }


    async fn consume_osc_message(self: &mut SharedState, msg: &OscMessage) {
        let mut hit_index: Option<usize> = None;
        for (i, key) in TARGET.iter().enumerate() {
            if msg.addr.contains(key) {
                tracing::debug!("Matched OSC message to key {}", key);
                hit_index = Some(i);
                break;
            }
        }
        let Some(hit_index) = hit_index else { return; };

        let Some(value) = Self::extract_first_numeric_arg(msg) else { return; };
        {
            let mut percentage = self.percentage.lock().await;
            percentage[hit_index] = value;
        }

        let n = NUMBERS[hit_index];
        if hit_index == 40 || hit_index == 41 {
            {
                let mut electrical_index = self.electrical_index.lock().await;
                if !electrical_index.contains(&n) {
                    electrical_index.push(n);
                }
            }
            tracing::debug!("Added electrical index {}", n);
        } else {
            {
                let mut shake_index = self.shake_index.lock().await;
                if !shake_index.contains(&n) {
                    shake_index.push(n);
                }
            }
            tracing::debug!("Added shake index {}", n);
        }
    }

    pub fn consume_osc_packet<'a>(
        &'a mut self,
        packet: &'a OscPacket,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            match packet {
                OscPacket::Message(msg) => self.consume_osc_message(msg).await,
                OscPacket::Bundle(b) => {
                    for p in &b.content {
                        self.consume_osc_packet(p).await;
                    }
                }
            }
        })
    }

    
    pub async fn build_effect(&mut self, shake_intensity: u16, electrical_intensity: u16, electrical_interval: u8) -> Option<true_gear_message::Effect> {
        // Lock the mutex to access the array
        let percentage = self.percentage.lock().await;
        let max_shake_intensity = percentage[..40].iter().cloned().fold(0 as f32, f32::max);
        let max_electrical_intensity = percentage[40..].iter().cloned().fold(0 as f32, f32::max);

        drop(percentage);

        let shake_index = {
            let shake_index_guard = self.shake_index.lock().await;
            shake_index_guard.clone()
        };

        let electrical_index = {
            let electrical_index_guard = self.electrical_index.lock().await;
            electrical_index_guard.clone()
        };

        let shake_track = true_gear_message::Track {
            action_type: true_gear_message::ActionType::Shake,
            intensity_mode: true_gear_message::IntensityMode::Const,
            stop_name: "".to_string(),
            start_intensity: Self::scale_intensity(shake_intensity, max_shake_intensity),
            end_intensity: Self::scale_intensity(shake_intensity, max_shake_intensity),
            start_time: 0,
            end_time: 150,
            interval: 0,
            once: false,
            index: shake_index,
        };

        let electrical_track = true_gear_message::Track {
            action_type: true_gear_message::ActionType::Electrical,
            intensity_mode: true_gear_message::IntensityMode::Const,
            stop_name: "".to_string(),
            start_intensity: Self::scale_intensity(electrical_intensity, max_electrical_intensity),
            end_intensity: Self::scale_intensity(electrical_intensity, max_electrical_intensity),
            start_time: 0,
            end_time: 150,
            interval: electrical_interval,
            once: false,
            index: electrical_index,
        };

        let mut effect = true_gear_message::Effect {
            uuid: "VRChatMsg".to_string(),
            name: "VRChatMsg".to_string(),
            keep: false,
            priority: 0,
            tracks: Vec::new(),
        };

        // only add non-empty tracks
        if !shake_track.index.is_empty() {
            effect.tracks.push(shake_track);
        }
        if !electrical_track.index.is_empty() {
            effect.tracks.push(electrical_track);
        }

        // reset inputs every tick
        {
            let mut shake_index = self.shake_index.lock().await;
            shake_index.clear();
        }
        {
            let mut electrical_index = self.electrical_index.lock().await;
            electrical_index.clear();
        }
        // Reset the percentage array
        {
            let mut percentage = self.percentage.lock().await;
            *percentage = [0.0; 42];
        }

        // only send if there's something to send
        if !effect.tracks.is_empty() {
            Some(effect)
        } else {
            None
        }
    }

}

