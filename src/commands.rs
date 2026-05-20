use std::net::SocketAddr;

pub fn handle(decoded: &[u8], _addr: SocketAddr) {
    let _hex_payload = hex::encode(decoded);

    // Always show raw payload
    //println!("[C2 from {}] Decoded: {}", addr, hex_payload);
    // Try to interpret as known command
    let text = String::from_utf8_lossy(decoded)
        .trim_end_matches('\0')
        .trim()
        .to_lowercase();

    match text.as_str() {
        "ping" => println!("[C2] Command: PING -> Responder alive"),
        
        "abort" | "stop" | "kill" | "exit" => {
            println!("[C2] Command: ABORT received! Shutting down...");
            std::process::exit(0);
        }
        
        "rotate" | "keyrotate" => {
            println!("[C2] Command: ROTATE KEY -> TODO: add logic");
        }
        
        "status" => println!("[C2] Command: STATUS -> System healthy"),
        
        _ if !text.is_empty() && text.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()) => {
            //println!("[C2] Command: Raw text -> {}", text);
        }
        _ => {
            //println!("[C2] Unknown command received");
        }
    }
}
