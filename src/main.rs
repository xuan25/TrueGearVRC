use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use std::{
    error::Error,
    net::{SocketAddr},
};

mod true_gear_message;

mod websocket;

mod mapping;
use mapping::{ProtocalMapper};

use crate::{reciver::Reciver, sender::Sender};

mod reciver;
mod sender;


#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    // OSC receive port
    #[arg(short, long, default_value_t = 9001)]
    receive_port: u16,

    // OSC send port
    #[arg(short, long, default_value_t = 9002)]
    send_port: u16,

    // whether to forward received OSC messages to send_port
    #[arg(short, long, default_value_t = false)]
    forward: bool,

    // TrueGear WebSocket endpoint
    #[arg(short, long, default_value = "ws://127.0.0.1:18233/v1/tact/")]
    truegear_ws_url: String,

    // Shake intensity
    #[arg(long, default_value_t = 50)]
    shake_intensity: u16,

    // Electrical intensity
    #[arg(long, default_value_t = 30)]
    electrical_intensity: u16,

    // Electrical interval
    #[arg(long, default_value_t = 10)]
    electrical_interval: u8,

    // Feedback mode
    #[arg(long, default_value = "continuous")]
    feedback_mode: mapping::FeedbackMode,

    // show debug logs
    #[arg(short, long, default_value_t = false, help = "Enable verbose logging")]
    verbose: bool,
}

fn setup_logging(level: Level) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut log_level = Level::INFO;

    if args.verbose {
        log_level = Level::TRACE;
    }

    setup_logging(log_level);

    if args.forward && args.receive_port == args.send_port {
        return Err("receive_port and send_port must differ when forwarding is enabled".into());
    }

    let forward_addr: Option<SocketAddr> = if args.forward {
        Some(format!("127.0.0.1:{}", args.send_port).parse()?)
    } else {
        None
    };

    if let Some(a) = forward_addr {
        println!("Forwarding OSC to {}", a);
    }

    let recv_addr: SocketAddr = format!("0.0.0.0:{}", args.receive_port).parse()?;

    let protocol_mapper = ProtocalMapper::new(args.feedback_mode);

    let reciver = Reciver::build(
        recv_addr,
        protocol_mapper.clone(),
        forward_addr,
    ).await;

    let mut sender = Sender::build(
        args.truegear_ws_url,
        protocol_mapper.clone(),
        args.shake_intensity,
        args.electrical_intensity,
        args.electrical_interval,
    ).await?;

    let mut reciver_clone = reciver.clone();
    let mut sender_clone = sender.clone();

    tokio::spawn(async move {
        if let Err(e) = reciver.run().await {
            tracing::error!("Receiver error: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = sender.run().await {
            tracing::error!("Sender error: {}", e);
        }
    });

    println!("Listening OSC on {}", recv_addr);

    tokio::signal::ctrl_c().await?;

    println!("Received Ctrl+C, shutting down.");

    sender_clone.close().await;
    reciver_clone.close().await;

    Ok(())
}
