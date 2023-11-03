use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};

use clap::Parser;
use anyhow::{Result, Context, anyhow};
use simple_can_send::ensure_result::ResultEnsure;
use socketcan::{CanSocket, Socket, Frame, ExtendedId, StandardId, CanFrame, EmbeddedFrame};

#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"), author = "Manuel Huber")]
pub struct Cli {
    /// CAN ID of the message to be sent
     #[arg(short = 'i', long, default_value = "100")]
    pub can_id: String,

    /// Private key to sign this request with
    #[arg(short = 'd', long, default_value = "A5F1")]
    pub data: String,

    /// Use extended CAN ID's for can_id and trigger?
    #[arg(short = 'e', long)]
    pub extended: bool,

    /// CAN ID that triggers the configured message
    #[arg(short = 't', long, default_value = "101")]
    pub trigger: String,

    /// CAN interface to use
    pub iface: String
}


#[derive(Debug)]
pub struct AppSettings {
    pub can_id: u32,
    pub data: [u8; 8],
    pub len: usize,
    pub trigger_can_id: u32,
    pub extended: bool,
    pub iface: String,
}
impl AppSettings {
    pub fn parse(cli: &Cli) -> Result<Self> {
        let extended = cli.extended;
        let max_id = Self::get_max_id(extended);
        let can_id = u32::from_str_radix(&cli.can_id, 16)
            .map_err(|e| e.into())
            .ensure(|&x| x <= max_id, |&x| anyhow!("{x} is too big (max: {max_id}"))?;
        let trigger_can_id = u32::from_str_radix(&cli.trigger, 16)
            .map_err(|e| e.into())
            .ensure(|&x| x <= max_id, |&x| anyhow!("{x} is too big (max: {max_id}"))?;

        let mut data = [0; 8];
        let mut len: usize = 0;

        Self::read_hex_bytes(&cli.data, &mut data, &mut len)?;

        Ok(Self {
            can_id, data, len, trigger_can_id, extended, iface: cli.iface.to_string()
        })
    }

    fn read_hex_bytes(input: &str, data: &mut [u8], len: &mut usize) -> Result<()> {
        let n = input.len();
        if n % 2 != 0 {
            anyhow::bail!("Length of the hex string declaring the data must be aligned to bytes");
        } else if n > 16 {
            anyhow::bail!("Lenght mustn't be longer than 8 bytes (16 hex characters)");
        }

        for i in (0..n).step_by(2) {
            let d = &input[i..(i+2)];
            data[i / 2] = u8::from_str_radix(d, 16)?;
            *len += 1;
        }

        Ok(())
    }

    fn get_max_id(extended: bool) -> u32 {
        if extended {
            (1 << 29) - 1
        } else {
            (1 << 11) - 1
        }
    }
}

fn create_frame(settings: &AppSettings) -> CanFrame {
    let data = &settings.data[0..settings.len];
    if settings.extended {
        let id = ExtendedId::new(settings.can_id).unwrap();
        return CanFrame::new(id, &data)
            .unwrap();
    } else {
        let id = StandardId::new(settings.can_id as u16).unwrap();
        return CanFrame::new(id, &data)
            .unwrap();
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let settings = AppSettings::parse(&cli)?;

    static RUNNING: AtomicBool = AtomicBool::new(true);

    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::Relaxed);
    })?;

    println!("Cli parameters: {cli:?}");
    println!("Settings: {settings:?}");

    let sock = CanSocket::open(&settings.iface)
        .with_context(|| format!("Failed to open socket on interface {}", settings.iface))?;

    let mut count: u64 = 0;

    while RUNNING.load(Ordering::Relaxed) {
        let triggered = sock.read_frame_timeout(Duration::from_secs(1))
            .ok()
            .map(|x| x.raw_id() == settings.trigger_can_id)
            .unwrap_or(false);

        if !triggered {
            continue;
        }

        for _i in 0..100 {
            let frame = create_frame(&settings);
            _ = sock.write_frame(&frame);
        }

        count = count.wrapping_add(1);
        println!("triggered {count}");
    }

    Ok(())
}
