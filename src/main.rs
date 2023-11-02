use clap::Parser;
use anyhow::{Result, anyhow};
use simple_can_send::ensure_result::ResultEnsure;

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
    pub len: u8,
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
        let mut len = 0;

        Self::read_hex_bytes(&cli.data, &mut data, &mut len)?;

        Ok(Self {
            can_id, data, len, trigger_can_id, extended, iface: cli.iface.to_string()
        })
    }

    fn read_hex_bytes(input: &str, data: &mut [u8], len: &mut u8) -> Result<()> {
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


fn main() -> Result<()> {
    let cli = Cli::try_parse()?;
    let settings = AppSettings::parse(&cli)?;

    println!("Cli parameters: {cli:?}");
    println!("Settings: {settings:?}");

    Ok(())
}
