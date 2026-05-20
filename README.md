# CHRONOS: Covert C2 via NTP Steganography

`CHRONOS` is a C2 (Command and Control) channel that operates entirely within **Network Time Protocol (NTP)** packets.

[FLUX](https://github.com/viyle-hq/flux) defeated traffic analysis by making everything look like constant noise. `CHRONOS` takes the opposite approach: **hiding in plain sight** by perfectly mimicking a ubiquitous, highly trusted protocol.

## How It Works

Every network allows UDP Port 123 (NTP) outbound so machines can sync their clocks. `CHRONOS` embeds encrypted C2 instructions and data inside NTP packets (lowest-order bits of the "Transmit Timestamp" field), up to 6 bytes of payload per packet. 

To an adversary, the traffic looks exactly like a normal server syncing its clock once a minute. The 48-bit per packet payload is small, but it's enough to send target coordinates, an abort code, or trigger a key rotation. 

It supports both short micro-commands and longer messages through automatic fragmentation.


## License

MIT License - Created by [Viyle Technologies](https://viyle.com)
