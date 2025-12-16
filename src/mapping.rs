use crate::true_gear_message;
use rosc::{OscMessage, OscPacket, OscType};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tokio::sync::Mutex;

const NUM_SHAKES: usize = 40;
const NUM_ELECTRICAL: usize = 2;
const NUM_DOTS: usize = NUM_SHAKES + NUM_ELECTRICAL;

const DOT_NAMES: [&str; NUM_DOTS] = [
    // shake dots first
    "TrueGearA1",
    "TrueGearA2",
    "TrueGearA3",
    "TrueGearA4",
    "TrueGearA5",
    "TrueGearB1",
    "TrueGearB2",
    "TrueGearB3",
    "TrueGearB4",
    "TrueGearB5",
    "TrueGearC1",
    "TrueGearC2",
    "TrueGearC3",
    "TrueGearC4",
    "TrueGearC5",
    "TrueGearD1",
    "TrueGearD2",
    "TrueGearD3",
    "TrueGearD4",
    "TrueGearD5",
    "TrueGearE1",
    "TrueGearE2",
    "TrueGearE3",
    "TrueGearE4",
    "TrueGearE5",
    "TrueGearF1",
    "TrueGearF2",
    "TrueGearF3",
    "TrueGearF4",
    "TrueGearF5",
    "TrueGearG1",
    "TrueGearG2",
    "TrueGearG3",
    "TrueGearG4",
    "TrueGearG5",
    "TrueGearH1",
    "TrueGearH2",
    "TrueGearH3",
    "TrueGearH4",
    "TrueGearH5",
    // then electrical dots
    "TrueGearArmL",
    "TrueGearArmR",
];

const DOT_IDS: [u8; NUM_DOTS] = [
    // shake dot IDs in TrueGear's defination
    1, 5, 9, 13, 17, 0, 4, 8, 12, 16, 100, 104, 108, 112, 116, 101, 105, 109, 113, 117, 102, 106,
    110, 114, 118, 103, 107, 111, 115, 119, 3, 7, 11, 15, 19, 2, 6, 10, 14, 18,
    // electrical dot IDs in TrueGear's defination
    0, 100,
];

static DOT_NAME_COMPACT_INDEX_MAP_CELL: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();

fn get_dot_name_compact_index_map() -> &'static HashMap<&'static str, usize> {
    DOT_NAME_COMPACT_INDEX_MAP_CELL.get_or_init(|| {
        let mut map = HashMap::new();
        DOT_NAMES.iter().enumerate().for_each(|(i, &name)| {
            map.insert(name, i);
        });
        map
    })
}

#[derive(clap::ValueEnum, Clone)]
pub enum FeedbackMode {
    Once,
    Continuous,
}

#[derive(Clone)]
pub struct ProtocalMapper {
    dot_intensities: Arc<Mutex<[f32; NUM_DOTS]>>,
    dot_active_states: Arc<Mutex<[bool; NUM_DOTS]>>,
    dot_name_compact_index_map: &'static HashMap<&'static str, usize>,
    pub feedback_mode: FeedbackMode,
}

impl Default for ProtocalMapper {
    fn default() -> Self {
        Self {
            dot_intensities: Arc::new(Mutex::new([0.0; NUM_DOTS])),
            dot_active_states: Arc::new(Mutex::new([false; NUM_DOTS])),
            dot_name_compact_index_map: get_dot_name_compact_index_map(),
            feedback_mode: FeedbackMode::Continuous,
        }
    }
}

impl ProtocalMapper {
    pub fn new(feedback_mode: FeedbackMode) -> Self {
        Self {
            dot_intensities: Arc::new(Mutex::new([0.0; NUM_DOTS])),
            dot_active_states: Arc::new(Mutex::new([false; NUM_DOTS])),
            dot_name_compact_index_map: get_dot_name_compact_index_map(),
            feedback_mode,
        }
    }

    fn extract_intensity(msg: &OscMessage) -> Option<f32> {
        if msg.args.is_empty() {
            None
        } else {
            match &msg.args[0] {
                OscType::Float(f) => Some(*f),
                OscType::Double(f) => Some(*f as f32),
                OscType::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            }
        }
    }

    fn scale_intensity(base: u16, factor: f32) -> u16 {
        let v = (base as f32 * factor).round();
        v.clamp(0.0, 150.0) as u16
    }

    async fn consume_osc_message(self: &mut ProtocalMapper, msg: &OscMessage) {
        let Some(dot_key) = msg.addr.rsplit('/').next() else {
            return;
        };
        let Some(dot_index_compact) = self.dot_name_compact_index_map.get(dot_key) else {
            return;
        };

        tracing::debug!("Matched OSC message to dot key {}", dot_key);

        let Some(intensity) = Self::extract_intensity(msg) else {
            return;
        };
        {
            self.dot_intensities.lock().await[*dot_index_compact] = intensity;
        }

        tracing::debug!("Set intensity for {} to {}", dot_key, intensity);

        let is_active = intensity > 0.0;

        self.dot_active_states.lock().await[*dot_index_compact] = is_active;
        tracing::debug!("Set active state for {} to {}", dot_key, is_active);
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

    pub async fn build_effect(
        &mut self,
        shake_intensity: u16,
        electrical_intensity: u16,
        electrical_interval: u8,
    ) -> Option<true_gear_message::Effect> {
        // Lock the mutex to access the array
        let percentage = self.dot_intensities.lock().await;
        let max_shake_intensity = percentage[..40].iter().cloned().fold(0 as f32, f32::max);
        let max_electrical_intensity = percentage[40..].iter().cloned().fold(0 as f32, f32::max);

        drop(percentage);

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
            // index: shake_index.into_iter().collect(),
            index: self
                .dot_active_states
                .lock()
                .await
                .iter()
                .enumerate()
                .filter_map(|(i, &active)| {
                    if active && i < NUM_SHAKES {
                        Some(DOT_IDS[i])
                    } else {
                        None
                    }
                })
                .collect(),
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
            // index: electrical_index.into_iter().collect(),
            index: self
                .dot_active_states
                .lock()
                .await
                .iter()
                .enumerate()
                .filter_map(|(i, &active)| {
                    if active && i >= NUM_SHAKES {
                        Some(DOT_IDS[i])
                    } else {
                        None
                    }
                })
                .collect(),
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

        if let FeedbackMode::Once = self.feedback_mode {
            // Reset inputs every tick in "Once" mode
            // otherwise the effect will keep playing until intensity becomes zero
            self.dot_active_states
                .lock()
                .await
                .iter_mut()
                .for_each(|s| *s = false);
            self.dot_intensities
                .lock()
                .await
                .iter_mut()
                .for_each(|p| *p = 0.0);
        }

        // only send if there's something to send
        if !effect.tracks.is_empty() {
            Some(effect)
        } else {
            None
        }
    }
}
