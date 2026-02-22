# OpenAlgo MCP Server (Rust)

A high-performance **Model Context Protocol (MCP)** server for [OpenAlgo](https://openalgo.in), built in Rust. Enables AI assistants (Claude, n8n, Windsurf, Cursor, ChatGPT) to execute trades, manage positions, and retrieve market data through supported Indian brokers.

## Features

- **38 MCP Tools** — Full parity with the Python `mcpserver.py`
- **Blazing fast** — Single static binary (~15 MB), sub-millisecond tool dispatch
- **Docker ready** — Multi-stage Dockerfile, docker-compose included
- **HTTP Streamable transport** — Industry-standard MCP transport over HTTP with SSE
- **Bearer token auth** — Secure access via `MCP_BEARER_TOKEN` environment variable
- **n8n compatible** — Connect directly from n8n's MCP Client node

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
3. **Docker** (recommended) or **Rust toolchain** (for building locally)

---

## Quick Start (Docker)

```bash
# Clone the repo
git clone https://github.com/your-repo/openalgo-mcp-rust.git
cd openalgo-mcp-rust

# Create environment file
cat > .env << EOF
OPENALGO_API_KEY=your_openalgo_api_key_here
OPENALGO_URL=http://host.docker.internal:5000
MCP_BEARER_TOKEN=your_secret_bearer_token_here
EOF

# Build and run
docker compose build
docker compose up -d

# Check logs
docker compose logs -f
```

The server starts on **port 8000** with endpoint `/mcp`.

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENALGO_API_KEY` | ✅ | — | Your OpenAlgo API key |
| `OPENALGO_URL` | ❌ | `http://host.docker.internal:5000` | OpenAlgo server URL |
| `MCP_BEARER_TOKEN` | ❌ | `changeme` | Bearer token for MCP client authentication |

> **Note**: Inside Docker, use `http://host.docker.internal:5000` to reach OpenAlgo running on the Docker host. On Linux, the `extra_hosts` config in `docker-compose.yml` handles this automatically.

---

## n8n Integration

The Rust MCP server uses **HTTP Streamable** transport with **Bearer authentication**, making it directly compatible with n8n's MCP Client node.

### Setup (Same Docker Network)

If n8n and the MCP server are on the **same Docker network**, connect using the container name:

1. **Ensure both containers share a Docker network:**
   ```bash
   # Create a shared network (if not already)
   docker network create openalgo-net

   # Connect both containers
   docker network connect openalgo-net openalgo-mcp-rust
   docker network connect openalgo-net n8n
   ```

2. **Configure the MCP Client node in n8n:**

   | Setting | Value |
   |---------|-------|
   | **SSE or Streamable HTTP** | `Streamable HTTP` |
   | **URL** | `http://openalgo-mcp-rust:8000/mcp` |
   | **Authentication** | `Header Auth` |
   | **Header Name** | `Authorization` |
   | **Header Value** | `Bearer your_secret_bearer_token_here` |

   > Replace `your_secret_bearer_token_here` with the same value you set in `MCP_BEARER_TOKEN` in the `.env` file.

3. **Test the connection** — The MCP Client node should discover all 38 tools automatically.

### Setup (Different Hosts)

If n8n runs on a different machine:

| Setting | Value |
|---------|-------|
| **URL** | `http://your-vps-ip:8000/mcp` |
| **Authentication** | `Header Auth` |
| **Header Name** | `Authorization` |
| **Header Value** | `Bearer your_secret_bearer_token_here` |

### n8n Workflow Example

1. Add an **AI Agent** node with your preferred LLM (Gemini, OpenAI, etc.)
2. Add an **MCP Client** tool node and configure as above
3. Connect the MCP Client to the AI Agent
4. Ask: *"Show me my current positions"* or *"Place a buy order for 10 shares of RELIANCE"*

---

## VPS Deployment

```bash
# 1. SSH into your VPS
ssh user@your-vps-ip

# 2. Install Docker (if not installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# 3. Clone the repository
git clone https://github.com/your-repo/openalgo-mcp-rust.git /opt/openalgo-mcp-rust
cd /opt/openalgo-mcp-rust

# 4. Configure
cat > .env << EOF
OPENALGO_API_KEY=your_actual_api_key
OPENALGO_URL=http://your-openalgo-host:5000
MCP_BEARER_TOKEN=a_strong_random_token
EOF

# 5. Build and run
docker compose build
docker compose up -d

# 6. Verify
docker compose logs -f
# Should see: OpenAlgo MCP Server listening on 0.0.0.0:8000
```

---

## Build from Source

### Install Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows — download from https://rustup.rs
```

### Build & Run

```bash
cd openalgo-mcp-rust

# Release build (optimized, ~15 MB binary)
cargo build --release

# Set environment variables
export OPENALGO_API_KEY="your_api_key_here"
export OPENALGO_URL="http://127.0.0.1:5000"
export MCP_BEARER_TOKEN="your_bearer_token"

# Run
./target/release/openalgo-mcp
```

The server listens on `http://0.0.0.0:8000/mcp`.

---

## Architecture

```
┌─────────────────┐     HTTP Streamable      ┌──────────────────────┐
│   n8n / Claude   │ ═══════════════════════> │  OpenAlgo MCP Server │
│   AI Assistant   │    Bearer Auth + SSE     │   (Rust, port 8000)  │
└─────────────────┘                           └──────────┬───────────┘
                                                         │ POST JSON
                                                         │ (apikey in body)
                                                         ▼
                                              ┌──────────────────────┐
                                              │   OpenAlgo Platform  │
                                              │  (Flask, port 5000)  │
                                              └──────────────────────┘
```

- **Transport**: HTTP Streamable (MCP 2025 spec) with SSE for streaming responses
- **Auth**: Bearer token via `Authorization: Bearer <token>` header
- **API calls**: OpenAlgo API key is injected into every request body as `apikey` field

---
### n8n Usage
**n8n MCP Client config (both on same Docker network):**

Connection Type: Streamable HTTP
URL: http://openalgo-mcp-rust:8000/mcp
Authentication: Header Auth → Authorization: Bearer your-secret-token-here

## Usage Examples

Once connected, ask your AI assistant:

- *"Place a buy order for 100 shares of RELIANCE at market price"*
- *"Show me my current positions"*
- *"Get the latest quote for NIFTY"*
- *"Get quotes for RELIANCE, INFY, and TCS"*
- *"Cancel all my pending orders"*
- *"What are my account funds?"*
- *"Place an iron condor on NIFTY with 25NOV25 expiry"*
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
| `401 Unauthorized` on `/mcp` | Check `MCP_BEARER_TOKEN` matches between `.env` and n8n |
| `apikey missing` error | Verify `OPENALGO_API_KEY` is set correctly in `.env` |
| Connection refused on port 8000 | Ensure the container is running: `docker compose ps` |
| n8n can't reach MCP server | Both containers must be on the same Docker network |
| Docker can't reach OpenAlgo | Use `http://host.docker.internal:5000` or the correct network alias |
| Gemini schema validation error | All array properties must have `items` defined (already fixed) |
| Build fails | Run `rustup update` to get latest Rust toolchain |

---

## Security

⚠️ **Important**: Keep your credentials secure.

- Never commit `.env` to version control
- Use strong, random values for `MCP_BEARER_TOKEN`
- Restrict port 8000 access via firewall in production
- Consider using Docker secrets for VPS deployments

## License

MIT
