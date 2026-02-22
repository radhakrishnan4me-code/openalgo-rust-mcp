# OpenAlgo MCP Server (Rust)

A high-performance **Model Context Protocol (MCP)** server for [OpenAlgo](https://openalgo.in), built in Rust. Enables AI assistants (Claude, Windsurf, Cursor, ChatGPT) to execute trades, manage positions, and retrieve market data through supported Indian brokers.

## Features

- **30+ MCP Tools** — Full parity with the Python `mcpserver.py`
- **Blazing fast** — Single static binary (~15MB), sub-millisecond tool dispatch
- **Docker ready** — Multi-stage Dockerfile, docker-compose included
- **Secure** — API key via environment variable, Bearer token auth
- **stdio transport** — Standard MCP protocol for local AI assistant integrations

## Available Tools

### Order Management
| Tool | Description |
|------|-------------|
| `place_order` | Place market or limit orders |
| `place_smart_order` | Place orders considering current position size |
| `place_basket_order` | Place multiple orders at once |
| `place_split_order` | Split large orders into smaller chunks |
| `place_options_order` | Place single-leg options order with ATM/ITM/OTM offset |
| `place_options_multi_order` | Multi-leg options strategies (spreads, iron condor, etc.) |
| `modify_order` | Modify existing orders |
| `cancel_order` | Cancel a specific order |
| `cancel_all_orders` | Cancel all orders for a strategy |

### Position Management
| Tool | Description |
|------|-------------|
| `close_all_positions` | Close all positions for a strategy |
| `get_open_position` | Get position for a specific instrument |

### Order Status & Tracking
| Tool | Description |
|------|-------------|
| `get_order_status` | Check status of a specific order |
| `get_order_book` | View all orders |
| `get_trade_book` | View executed trades |
| `get_position_book` | View current positions |
| `get_holdings` | View long-term holdings |
| `get_funds` | Check account funds and margins |
| `calculate_margin` | Calculate margin requirements |

### Market Data
| Tool | Description |
|------|-------------|
| `get_quote` | Get current price quote |
| `get_multi_quotes` | Get quotes for multiple symbols |
| `get_market_depth` | Get order book depth |
| `get_historical_data` | Retrieve historical price data |
| `get_option_chain` | Get option chain with real-time quotes |

### Instrument Search & Info
| Tool | Description |
|------|-------------|
| `search_instruments` | Search for trading instruments |
| `get_symbol_info` | Get detailed symbol information |
| `get_expiry_dates` | Get derivative expiry dates |
| `get_available_intervals` | List available time intervals |
| `get_option_symbol` | Get option symbol for strike & expiry |
| `get_synthetic_future` | Calculate synthetic future price |
| `get_option_greeks` | Calculate option Greeks |
| `get_instruments` | Download all instruments for an exchange |
| `get_index_symbols` | Get index symbols for NSE/BSE |

### Utilities
| Tool | Description |
|------|-------------|
| `validate_order_constants` | Display valid order parameters |
| `send_telegram_alert` | Send Telegram notifications |
| `get_holidays` | Get trading holidays for a year |
| `get_timings` | Get exchange trading timings |
| `analyzer_status` | Get analyzer mode status |
| `analyzer_toggle` | Toggle analyze/live trading mode |

---

## Prerequisites

1. **OpenAlgo Server** running and accessible (e.g., `http://127.0.0.1:5000`)
2. **OpenAlgo API Key** — get from: OpenAlgo Web UI → Settings → API Keys
3. **Rust toolchain** (if building locally) or **Docker** (recommended for VPS)

---

## Option 1: Docker (Recommended for VPS)

### Quick Start

```bash
# Clone or copy the project
cd openalgo-mcp-rust

# Configure environment
cp .env.example .env
# Edit .env with your API key and OpenAlgo URL

# Build and run
docker compose build
docker compose up -d
```

### VPS Deployment

```bash
# 1. SSH into your VPS
ssh user@your-vps-ip

# 2. Install Docker (if not installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# 3. Copy the project files to VPS
# (use scp, rsync, or git clone)
scp -r openalgo-mcp-rust/ user@your-vps-ip:/opt/openalgo-mcp/

# 4. Configure
cd /opt/openalgo-mcp
cp .env.example .env
nano .env
# Set OPENALGO_API_KEY=your_actual_api_key
# Set OPENALGO_URL=http://127.0.0.1:5000  (or your OpenAlgo host)

# 5. Build and run
docker compose build
docker compose up -d

# 6. Check logs
docker compose logs -f
```

### Docker Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENALGO_API_KEY` | ✅ | — | Your OpenAlgo API key |
| `OPENALGO_URL` | ❌ | `http://host.docker.internal:5000` | OpenAlgo server URL |

> **Note**: Inside Docker, use `http://host.docker.internal:5000` to reach OpenAlgo running on the Docker host machine. On Linux VPS, the `extra_hosts` config in docker-compose.yml handles this automatically.

---

## Option 2: Build from Source

### Install Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download from https://rustup.rs
```

### Build

```bash
cd openalgo-mcp-rust

# Debug build
cargo build

# Release build (optimized, ~15MB binary)
cargo build --release
```

### Run

```bash
# Set environment variables
export OPENALGO_API_KEY="your_api_key_here"
export OPENALGO_URL="http://127.0.0.1:5000"

# Run the server
cargo run --release

# Or run the binary directly
./target/release/openalgo-mcp
```

The binary also supports command-line arguments (backward compatible with the Python version):

```bash
./target/release/openalgo-mcp YOUR_API_KEY http://127.0.0.1:5000
```

---

## MCP Client Configuration

### Claude Desktop

#### Windows
Path: `%APPDATA%\Claude\claude_desktop_config.json`

**Using binary:**
```json
{
  "mcpServers": {
    "openalgo": {
      "command": "C:\\path\\to\\openalgo-mcp.exe",
      "env": {
        "OPENALGO_API_KEY": "YOUR_API_KEY_HERE",
        "OPENALGO_URL": "http://127.0.0.1:5000"
      }
    }
  }
}
```

**Using Docker:**
```json
{
  "mcpServers": {
    "openalgo": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "OPENALGO_API_KEY=YOUR_API_KEY_HERE",
        "-e", "OPENALGO_URL=http://host.docker.internal:5000",
        "--add-host=host.docker.internal:host-gateway",
        "openalgo-mcp-rust"
      ]
    }
  }
}
```

#### macOS
Path: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "openalgo": {
      "command": "/path/to/openalgo-mcp",
      "env": {
        "OPENALGO_API_KEY": "YOUR_API_KEY_HERE",
        "OPENALGO_URL": "http://127.0.0.1:5000"
      }
    }
  }
}
```

#### Linux
Path: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "openalgo": {
      "command": "/opt/openalgo-mcp/openalgo-mcp",
      "env": {
        "OPENALGO_API_KEY": "YOUR_API_KEY_HERE",
        "OPENALGO_URL": "http://127.0.0.1:5000"
      }
    }
  }
}
```

### Windsurf

| OS | Path |
|----|------|
| Windows | `%APPDATA%\Windsurf\mcp_config.json` |
| macOS | `~/.config/windsurf/mcp_config.json` |
| Linux | `~/.config/windsurf/mcp_config.json` |

Use the same JSON format as Claude Desktop above.

### Cursor

| OS | Path |
|----|------|
| Windows | `%APPDATA%\Cursor\User\settings.json` |
| macOS | `~/Library/Application Support/Cursor/User/settings.json` |
| Linux | `~/.config/Cursor/User/settings.json` |

Use the same JSON format as Claude Desktop above.

---
### n8n Usage
**n8n MCP Client config (both on same Docker network):**

Connection Type: Streamable HTTP
URL: http://openalgo-mcp-rust:8000/mcp
Authentication: Header Auth → Authorization: Bearer your-secret-token-here

## Usage Examples

Once configured, ask your AI assistant:

- *"Place a buy order for 100 shares of RELIANCE at market price"*
- *"Show me my current positions"*
- *"Get the latest quote for NIFTY"*
- *"Get quotes for RELIANCE, INFY, and TCS"*
- *"Cancel all my pending orders"*
- *"What are my account funds?"*
- *"Place an iron condor on NIFTY with 25NOV25 expiry using OTM4 and OTM6 strikes"*
- *"Calculate the synthetic future price for NIFTY 25NOV25 expiry"*
- *"Get option Greeks for NIFTY 26000 CE expiring on 25NOV25"*
- *"Show me the option chain for NIFTY with 30DEC25 expiry"*
- *"What are the trading holidays in 2025?"*
- *"What are the market timings for today?"*

---

## Supported Exchanges

| Code | Exchange |
|------|----------|
| `NSE` | National Stock Exchange (Equity) |
| `NFO` | NSE Futures & Options |
| `CDS` | NSE Currency Derivatives |
| `BSE` | Bombay Stock Exchange |
| `BFO` | BSE Futures & Options |
| `BCD` | BSE Currency Derivatives |
| `MCX` | Multi Commodity Exchange |
| `NCDEX` | National Commodity & Derivatives Exchange |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Verify OpenAlgo server is running on the configured URL |
| Auth errors | Check API key is correct and valid |
| Docker can't reach host | Ensure `host.docker.internal` is properly configured (Linux needs `extra_hosts`) |
| Build fails | Run `rustup update` to get latest Rust toolchain |

---

## Security

⚠️ **Important**: Keep your API key secure. Never commit `.env` to version control.

- Use environment variables for API keys
- Restrict network access in production
- Consider using Docker secrets for VPS deployments

## License

MIT
