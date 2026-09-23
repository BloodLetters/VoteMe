# VoteMe
NuVotifier/Votifier porting for Pumpkin Server

## Features
- NuVotifier v2 protocol support (HMAC-SHA256 authentication)
- Classic Votifier v1 protocol support (2048-bit RSA encryption)
- Non-blocking network listener
- HAProxy PROXY Protocol v1 and v2 support
- Rate limiting, IP throttling, and brute-force protection
- Player in-game voting links command (`/vote`)
- Administrative runtime statistics and diagnostics (`/voteme status`)
- Automated console reward commands and broadcast announcements

## Roadmap
Planned features and future enhancements:
- [x] **Inter-Plugin Communication (IPC)**
- [ ] **Offline Vote Queuing and Persistence**
- [ ] **Network Vote Forwarding**

## Installation
1. Compile the plugin using `cargo build --release` or obtain the precompiled `voteme.wasm`.
2. Place `voteme.wasm` into the `plugins/` directory of your Pumpkin server.
3. Start the server once to automatically generate default configurations and cryptographic keys.
4. Ensure the required WASI permissions are granted in `plugins/permission_cache.json`.

## Configuration

The configuration file is located at `plugins/data/voteme/config.json`.

```json
{
  "host": "0.0.0.0",
  "port": 8192,
  "tokens": {
    "default": "YourSecretTokenHere"
  },
  "disable_v1": false,
  "broadcast_message": "§a[Vote] §f{player} just voted on §e{service}§f!",
  "reward_commands": [
    "give {player} diamond 1",
    "tellraw {player} [{\"text\":\"Thank you for voting!\",\"color\":\"green\"}]"
  ],
  "vote_sites": [
    {
      "name": "PlanetMinecraft",
      "url": "https://www.planetminecraft.com/"
    },
    {
      "name": "Minecraft-MP",
      "url": "https://minecraft-mp.com/"
    }
  ],
  "throttle_enabled": true
}
```

### Configuration Options

| Option | Type | Default | Description |
|---|---|---|---|
| `host` | String | `"0.0.0.0"` | Network interface address to bind the listener. |
| `port` | Integer | `8192` | TCP port for incoming Votifier connections. |
| `tokens` | Object | `{"default": "..."}` | NuVotifier v2 service tokens mapped by service name or `default`. |
| `disable_v1` | Boolean | `false` | When set to `true`, rejects legacy RSA-encrypted V1 votes. |
| `broadcast_message` | String | Text string | Global announcement sent to all players upon valid vote. |
| `reward_commands` | Array | Command list | Server console commands executed upon receiving a valid vote. |
| `vote_sites` | Array | Site objects | List of voting sites displayed to players via `/vote`. |
| `throttle_enabled` | Boolean | `true` | Enables connection throttling and IP protection against spam. |

### Message Placeholders

The following placeholders are supported in `broadcast_message` and `reward_commands`:

| Placeholder | Description | Example Value |
|---|---|---|
| `{player}` | Sanitized Minecraft username of the voter | `SirAshesh` |
| `{service}` | Name of the voting list service | `PlanetMinecraft` |
| `{address}` | Originating IP address of the voter | `127.0.0.1` |
| `{timestamp}` | Timestamp string provided in the vote payload | `1727000000` |



## Voting List Setup

### 1. NuVotifier v2 (Modern Protocol)

Configure your server listing on the voting website with:
- **Host / IP**: Your server public IP or domain name
- **Port**: Port configured in `config.json` (default: `8192`)
- **Token**: The secret token from `config.json` (`tokens.default` or matching service name)

### 2. Classic Votifier v1 (Legacy Protocol)

If a voting site only supports V1:
- **Host / IP**: Your server public IP or domain name
- **Port**: Port configured in `config.json` (default: `8192`)
- **Public Key**: Copy the complete content of `plugins/data/voteme/rsa/public.pem`

## Developer Guide
For the full developer integration guide, JSON schemas, and rust listener example, see [docs/Listener.md](docs/Listener.md).

## Commands and In-Game Permissions

| Command | Aliases | Permission | Default | Description |
|---|---|---|---|---|
| `/vote` | `/votes` | `voteme:vote` | Everyone (`allow`) | Displays configured server voting websites. |
| `/vote list` | `/vote sites` | `voteme:vote` | Everyone (`allow`) | Displays configured server voting websites. |
| `/voteme` | `/votifier` | `voteme:admin` | OP | Displays plugin runtime status and listener details. |
| `/voteme status` | - | `voteme:admin` | OP | Displays real-time statistics (total inbound, V1, V2, failed). |
| `/voteme help` | - | `voteme:admin` | OP | Displays administrator help guide. |

## Building from Source
To build an optimized release binary:

```shell
cargo build --release --target wasm32-wasip2
```

The compiled WebAssembly component will be located at:

```text
target/wasm32-wasip2/release/voteme.wasm
```