mod client;

use client::OpenAlgoClient;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError,
};
use serde_json::{json, Value};
use std::env;

/// OpenAlgo MCP Server — exposes all OpenAlgo trading APIs as MCP tools.
#[derive(Clone)]
pub struct OpenAlgoMcp {
    client: OpenAlgoClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl OpenAlgoMcp {
    fn new(client: OpenAlgoClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    // ─── Helper ───────────────────────────────────────────────────────────

    fn ok(val: Value) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&val).unwrap_or_default(),
        )]))
    }

    fn err(msg: String) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  ORDER MANAGEMENT TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Place a new order (market or limit).

Args:
  symbol: Stock symbol (e.g. RELIANCE)
  quantity: Number of shares
  action: BUY or SELL
  exchange: NSE, NFO, CDS, BSE, BFO, BCD, MCX, NCDEX (default: NSE)
  price_type: MARKET, LIMIT, SL, SL-M (default: MARKET)
  product: CNC, NRML, MIS (default: MIS)
  strategy: Strategy name (default: Rust)
  price: Limit price (for LIMIT orders)
  trigger_price: Trigger price (for SL orders)
  disclosed_quantity: Disclosed quantity")]
    async fn place_order(
        &self,
        #[tool(param, description = "Stock symbol e.g. RELIANCE")] symbol: String,
        #[tool(param, description = "Number of shares")] quantity: i64,
        #[tool(param, description = "BUY or SELL")] action: String,
        #[tool(param, description = "Exchange: NSE, NFO, CDS, BSE, BFO, BCD, MCX, NCDEX")] exchange: Option<String>,
        #[tool(param, description = "MARKET, LIMIT, SL, SL-M")] price_type: Option<String>,
        #[tool(param, description = "CNC, NRML, MIS")] product: Option<String>,
        #[tool(param, description = "Strategy name")] strategy: Option<String>,
        #[tool(param, description = "Limit price for LIMIT orders")] price: Option<f64>,
        #[tool(param, description = "Trigger price for SL orders")] trigger_price: Option<f64>,
        #[tool(param, description = "Disclosed quantity")] disclosed_quantity: Option<i64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": symbol.to_uppercase(),
            "action": action.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
            "price_type": price_type.unwrap_or_else(|| "MARKET".into()).to_uppercase(),
            "product": product.unwrap_or_else(|| "MIS".into()).to_uppercase(),
            "strategy": strategy.unwrap_or_else(|| "Rust".into()),
            "quantity": quantity.to_string(),
        });
        if let Some(p) = price { body["price"] = json!(p.to_string()); }
        if let Some(tp) = trigger_price { body["trigger_price"] = json!(tp.to_string()); }
        if let Some(dq) = disclosed_quantity { body["disclosed_quantity"] = json!(dq.to_string()); }

        match self.client.post("/placeorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing order: {e}")),
        }
    }

    #[tool(description = "Place a smart order considering current position size.

Args:
  symbol: Stock symbol
  quantity: Number of shares
  action: BUY or SELL
  position_size: Current position size
  exchange: Exchange (default: NSE)
  price_type: Order type (default: MARKET)
  product: Product type (default: MIS)
  strategy: Strategy name (default: Rust)
  price: Limit price (optional)")]
    async fn place_smart_order(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Number of shares")] quantity: i64,
        #[tool(param, description = "BUY or SELL")] action: String,
        #[tool(param, description = "Current position size")] position_size: i64,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
        #[tool(param, description = "MARKET, LIMIT, SL, SL-M")] price_type: Option<String>,
        #[tool(param, description = "CNC, NRML, MIS")] product: Option<String>,
        #[tool(param, description = "Strategy name")] strategy: Option<String>,
        #[tool(param, description = "Limit price")] price: Option<f64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": symbol.to_uppercase(),
            "action": action.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
            "price_type": price_type.unwrap_or_else(|| "MARKET".into()).to_uppercase(),
            "product": product.unwrap_or_else(|| "MIS".into()).to_uppercase(),
            "strategy": strategy.unwrap_or_else(|| "Rust".into()),
            "quantity": quantity.to_string(),
            "position_size": position_size.to_string(),
        });
        if let Some(p) = price { body["price"] = json!(p.to_string()); }

        match self.client.post("/placesmartorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing smart order: {e}")),
        }
    }

    #[tool(description = "Place multiple orders in a basket.

Args:
  orders: JSON array of order objects. Each: {symbol, exchange, action, quantity, pricetype?, product?, price?, trigger_price?}
  strategy: Strategy name (default: Rust)

Example orders: [{\"symbol\":\"BHEL\",\"exchange\":\"NSE\",\"action\":\"BUY\",\"quantity\":1,\"pricetype\":\"MARKET\",\"product\":\"MIS\"}]")]
    async fn place_basket_order(
        &self,
        #[tool(param, description = "JSON array of order objects")] orders: String,
        #[tool(param, description = "Strategy name")] strategy: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let parsed: Value = serde_json::from_str(&orders)
            .map_err(|e| format!("Invalid orders JSON: {e}"))
            .unwrap_or(json!([]));
        let body = json!({
            "strategy": strategy.unwrap_or_else(|| "Rust".into()),
            "orders": parsed,
        });
        match self.client.post("/basketorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing basket order: {e}")),
        }
    }

    #[tool(description = "Split a large order into smaller chunks.

Args:
  symbol: Stock symbol (e.g. YESBANK)
  quantity: Total quantity to trade
  split_size: Size of each split order
  action: BUY or SELL
  exchange: Exchange (default: NSE)
  price_type: Order type (default: MARKET)
  product: Product type (default: MIS)
  strategy: Strategy name (default: Rust)
  price: Limit price (optional)
  trigger_price: Trigger price (optional)")]
    async fn place_split_order(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Total quantity")] quantity: i64,
        #[tool(param, description = "Size per split")] split_size: i64,
        #[tool(param, description = "BUY or SELL")] action: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
        #[tool(param, description = "MARKET, LIMIT, SL, SL-M")] price_type: Option<String>,
        #[tool(param, description = "CNC, NRML, MIS")] product: Option<String>,
        #[tool(param, description = "Strategy name")] strategy: Option<String>,
        #[tool(param, description = "Limit price")] price: Option<f64>,
        #[tool(param, description = "Trigger price")] trigger_price: Option<f64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": symbol.to_uppercase(),
            "action": action.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
            "price_type": price_type.unwrap_or_else(|| "MARKET".into()).to_uppercase(),
            "product": product.unwrap_or_else(|| "MIS".into()).to_uppercase(),
            "strategy": strategy.unwrap_or_else(|| "Rust".into()),
            "quantity": quantity.to_string(),
            "splitsize": split_size.to_string(),
        });
        if let Some(p) = price { body["price"] = json!(p.to_string()); }
        if let Some(tp) = trigger_price { body["trigger_price"] = json!(tp.to_string()); }

        match self.client.post("/splitorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing split order: {e}")),
        }
    }

    #[tool(description = "Place an options order with ATM/ITM/OTM offset.

Args:
  underlying: Underlying symbol (e.g. NIFTY, BANKNIFTY, NIFTY28OCT25FUT)
  exchange: Exchange for underlying (NSE_INDEX, BSE_INDEX, NFO)
  offset: Strike offset - ATM, ITM1-ITM50, OTM1-OTM50
  option_type: CE for Call or PE for Put
  action: BUY or SELL
  quantity: Number of lots (must be multiple of lot size)
  expiry_date: Expiry in DDMMMYY format (e.g. 28OCT25). Optional if underlying includes expiry.
  strategy: Strategy name (default: Rust)
  price_type: MARKET, LIMIT, SL, SL-M (default: MARKET)
  product: MIS, NRML (default: MIS)
  price: Limit price (optional)
  trigger_price: Trigger price (optional)")]
    async fn place_options_order(
        &self,
        #[tool(param, description = "Underlying symbol")] underlying: String,
        #[tool(param, description = "Exchange for underlying")] exchange: String,
        #[tool(param, description = "ATM, ITM1-ITM50, OTM1-OTM50")] offset: String,
        #[tool(param, description = "CE or PE")] option_type: String,
        #[tool(param, description = "BUY or SELL")] action: String,
        #[tool(param, description = "Number of lots")] quantity: i64,
        #[tool(param, description = "Expiry DDMMMYY e.g. 28OCT25")] expiry_date: Option<String>,
        #[tool(param, description = "Strategy name")] strategy: Option<String>,
        #[tool(param, description = "MARKET, LIMIT, SL, SL-M")] price_type: Option<String>,
        #[tool(param, description = "MIS, NRML")] product: Option<String>,
        #[tool(param, description = "Limit price")] price: Option<f64>,
        #[tool(param, description = "Trigger price")] trigger_price: Option<f64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "underlying": underlying.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "offset": offset.to_uppercase(),
            "option_type": option_type.to_uppercase(),
            "action": action.to_uppercase(),
            "quantity": quantity.to_string(),
            "strategy": strategy.unwrap_or_else(|| "Rust".into()),
            "price_type": price_type.unwrap_or_else(|| "MARKET".into()).to_uppercase(),
            "product": product.unwrap_or_else(|| "MIS".into()).to_uppercase(),
        });
        if let Some(ed) = expiry_date { body["expiry_date"] = json!(ed); }
        if let Some(p) = price { body["price"] = json!(p.to_string()); }
        if let Some(tp) = trigger_price { body["trigger_price"] = json!(tp.to_string()); }

        match self.client.post("/optionsorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing options order: {e}")),
        }
    }

    #[tool(description = "Place a multi-leg options order (spreads, iron condor, straddles).
BUY legs execute first for margin efficiency, then SELL legs.

Args:
  strategy: Strategy name (required)
  underlying: Underlying symbol (e.g. NIFTY, BANKNIFTY)
  exchange: Exchange for underlying (NSE_INDEX, BSE_INDEX, NFO)
  legs: JSON array of leg objects. Each: {offset, option_type, action, quantity, expiry_date?, pricetype?, product?, price?, trigger_price?}
  expiry_date: Default expiry DDMMMYY for all legs (optional)

Iron Condor example legs:
[{\"offset\":\"OTM10\",\"option_type\":\"CE\",\"action\":\"BUY\",\"quantity\":75},
 {\"offset\":\"OTM10\",\"option_type\":\"PE\",\"action\":\"BUY\",\"quantity\":75},
 {\"offset\":\"OTM5\",\"option_type\":\"CE\",\"action\":\"SELL\",\"quantity\":75},
 {\"offset\":\"OTM5\",\"option_type\":\"PE\",\"action\":\"SELL\",\"quantity\":75}]")]
    async fn place_options_multi_order(
        &self,
        #[tool(param, description = "Strategy name")] strategy: String,
        #[tool(param, description = "Underlying symbol")] underlying: String,
        #[tool(param, description = "Exchange for underlying")] exchange: String,
        #[tool(param, description = "JSON array of leg objects")] legs: String,
        #[tool(param, description = "Default expiry DDMMMYY")] expiry_date: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let parsed_legs: Value = serde_json::from_str(&legs)
            .unwrap_or(json!([]));
        let mut body = json!({
            "strategy": strategy,
            "underlying": underlying.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "legs": parsed_legs,
        });
        if let Some(ed) = expiry_date { body["expiry_date"] = json!(ed); }

        match self.client.post("/optionsmultiorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing options multi order: {e}")),
        }
    }

    #[tool(description = "Modify an existing order.

Args:
  order_id: Order ID to modify
  strategy: Strategy name
  symbol: Stock symbol
  action: BUY or SELL
  exchange: Exchange name
  price_type: Order type
  product: Product type
  quantity: New quantity
  price: New price (optional)")]
    async fn modify_order(
        &self,
        #[tool(param, description = "Order ID to modify")] order_id: String,
        #[tool(param, description = "Strategy name")] strategy: String,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "BUY or SELL")] action: String,
        #[tool(param, description = "Exchange")] exchange: String,
        #[tool(param, description = "MARKET, LIMIT, SL, SL-M")] price_type: String,
        #[tool(param, description = "CNC, NRML, MIS")] product: String,
        #[tool(param, description = "New quantity")] quantity: i64,
        #[tool(param, description = "New price")] price: Option<f64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "order_id": order_id,
            "strategy": strategy,
            "symbol": symbol.to_uppercase(),
            "action": action.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "price_type": price_type.to_uppercase(),
            "product": product.to_uppercase(),
            "quantity": quantity.to_string(),
        });
        if let Some(p) = price { body["price"] = json!(p.to_string()); }

        match self.client.post("/modifyorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error modifying order: {e}")),
        }
    }

    #[tool(description = "Cancel a specific order.

Args:
  order_id: Order ID to cancel
  strategy: Strategy name")]
    async fn cancel_order(
        &self,
        #[tool(param, description = "Order ID")] order_id: String,
        #[tool(param, description = "Strategy name")] strategy: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "order_id": order_id, "strategy": strategy });
        match self.client.post("/cancelorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error canceling order: {e}")),
        }
    }

    #[tool(description = "Cancel all open orders for a strategy.

Args:
  strategy: Strategy name")]
    async fn cancel_all_orders(
        &self,
        #[tool(param, description = "Strategy name")] strategy: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "strategy": strategy });
        match self.client.post("/cancelallorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error canceling all orders: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  POSITION MANAGEMENT TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Close all open positions for a strategy.

Args:
  strategy: Strategy name")]
    async fn close_all_positions(
        &self,
        #[tool(param, description = "Strategy name")] strategy: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "strategy": strategy });
        match self.client.post("/closeposition", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error closing positions: {e}")),
        }
    }

    #[tool(description = "Get current open position for a specific instrument.

Args:
  strategy: Strategy name
  symbol: Stock symbol
  exchange: Exchange name
  product: Product type")]
    async fn get_open_position(
        &self,
        #[tool(param, description = "Strategy name")] strategy: String,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: String,
        #[tool(param, description = "Product type")] product: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "strategy": strategy,
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "product": product.to_uppercase(),
        });
        match self.client.post("/openposition", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting open position: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  ORDER STATUS & TRACKING TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Get status of a specific order.

Args:
  order_id: Order ID
  strategy: Strategy name")]
    async fn get_order_status(
        &self,
        #[tool(param, description = "Order ID")] order_id: String,
        #[tool(param, description = "Strategy name")] strategy: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "order_id": order_id, "strategy": strategy });
        match self.client.post("/orderstatus", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting order status: {e}")),
        }
    }

    #[tool(description = "Get all orders from the order book.")]
    async fn get_order_book(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/orderbook", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting order book: {e}")),
        }
    }

    #[tool(description = "Get all executed trades.")]
    async fn get_trade_book(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/tradebook", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting trade book: {e}")),
        }
    }

    #[tool(description = "Get all current positions.")]
    async fn get_position_book(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/positionbook", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting position book: {e}")),
        }
    }

    #[tool(description = "Get all holdings (long-term investments).")]
    async fn get_holdings(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/holdings", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting holdings: {e}")),
        }
    }

    #[tool(description = "Get account funds and margin information.")]
    async fn get_funds(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/funds", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting funds: {e}")),
        }
    }

    #[tool(description = "Calculate margin requirements for positions.

Args:
  positions: JSON array of position objects.
  Example: [{\"symbol\":\"NIFTY25NOV2525000CE\",\"exchange\":\"NFO\",\"action\":\"BUY\",\"product\":\"NRML\",\"pricetype\":\"MARKET\",\"quantity\":\"75\"}]")]
    async fn calculate_margin(
        &self,
        #[tool(param, description = "JSON array of position objects")] positions: String,
    ) -> Result<CallToolResult, McpError> {
        let parsed: Value = serde_json::from_str(&positions).unwrap_or(json!([]));
        let body = json!({ "positions": parsed });
        match self.client.post("/margin", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating margin: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  MARKET DATA TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Get current quote for a symbol.

Args:
  symbol: Stock symbol
  exchange: Exchange name (default: NSE)")]
    async fn get_quote(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
        });
        match self.client.post("/quotes", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting quote: {e}")),
        }
    }

    #[tool(description = "Get real-time quotes for multiple symbols in a single request.

Args:
  symbols: JSON array of {symbol, exchange} pairs.
  Example: [{\"symbol\":\"RELIANCE\",\"exchange\":\"NSE\"},{\"symbol\":\"INFY\",\"exchange\":\"NSE\"}]")]
    async fn get_multi_quotes(
        &self,
        #[tool(param, description = "JSON array of {symbol, exchange} pairs")] symbols: String,
    ) -> Result<CallToolResult, McpError> {
        let parsed: Value = serde_json::from_str(&symbols).unwrap_or(json!([]));
        let body = json!({ "symbols": parsed });
        match self.client.post("/multiquotes", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting multi quotes: {e}")),
        }
    }

    #[tool(description = "Get option chain data with real-time quotes for all strikes.

Args:
  underlying: Underlying symbol (e.g. NIFTY, BANKNIFTY, RELIANCE)
  exchange: Exchange for underlying (NSE_INDEX, BSE_INDEX, NSE, BSE)
  expiry_date: Expiry date in DDMMMYY format (e.g. 30DEC25)
  strike_count: Number of strikes above and below ATM (1-100). If not provided, returns entire chain.")]
    async fn get_option_chain(
        &self,
        #[tool(param, description = "Underlying symbol")] underlying: String,
        #[tool(param, description = "Exchange for underlying")] exchange: String,
        #[tool(param, description = "Expiry DDMMMYY")] expiry_date: String,
        #[tool(param, description = "Strikes around ATM (1-100)")] strike_count: Option<i64>,
    ) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "underlying": underlying.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "expiry_date": expiry_date.to_uppercase(),
        });
        if let Some(sc) = strike_count { body["strike_count"] = json!(sc); }

        match self.client.post("/optionchain", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting option chain: {e}")),
        }
    }

    #[tool(description = "Get market depth (order book) for a symbol.

Args:
  symbol: Stock symbol
  exchange: Exchange name (default: NSE)")]
    async fn get_market_depth(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
        });
        match self.client.post("/depth", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting market depth: {e}")),
        }
    }

    #[tool(description = "Get historical price data.

Args:
  symbol: Stock symbol
  exchange: Exchange name
  interval: Time interval (1m, 3m, 5m, 10m, 15m, 30m, 1h, D)
  start_date: Start date (YYYY-MM-DD)
  end_date: End date (YYYY-MM-DD)")]
    async fn get_historical_data(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: String,
        #[tool(param, description = "1m, 3m, 5m, 10m, 15m, 30m, 1h, D")] interval: String,
        #[tool(param, description = "Start date YYYY-MM-DD")] start_date: String,
        #[tool(param, description = "End date YYYY-MM-DD")] end_date: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "interval": interval,
            "start_date": start_date,
            "end_date": end_date,
        });
        match self.client.post("/history", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting historical data: {e}")),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  INSTRUMENT SEARCH & INFO TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Search for instruments by name or symbol.

Args:
  query: Search query
  exchange: Exchange to search in (NSE, BSE, NFO, NSE_INDEX, BSE_INDEX, etc.)")]
    async fn search_instruments(
        &self,
        #[tool(param, description = "Search query")] query: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "query": query,
            "exchange": exchange.unwrap_or_else(|| "NSE".into()).to_uppercase(),
        });
        match self.client.post("/search", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error searching instruments: {e}")),
        }
    }

    #[tool(description = "Get detailed information about a symbol.

Args:
  symbol: Stock symbol
  exchange: Exchange name (default: NSE). For indices use NSE_INDEX or BSE_INDEX.")]
    async fn get_symbol_info(
        &self,
        #[tool(param, description = "Stock symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        // Auto-detect index exchanges
        let sym_upper = symbol.to_uppercase();
        let nse_indices = ["NIFTY", "NIFTYNXT50", "FINNIFTY", "BANKNIFTY", "MIDCPNIFTY", "INDIAVIX"];
        let bse_indices = ["SENSEX", "BANKEX", "SENSEX50"];

        let exch = match exchange {
            Some(e) => {
                let e_upper = e.to_uppercase();
                if nse_indices.contains(&sym_upper.as_str()) && e_upper == "NSE" {
                    "NSE_INDEX".to_string()
                } else if bse_indices.contains(&sym_upper.as_str()) && e_upper == "BSE" {
                    "BSE_INDEX".to_string()
                } else {
                    e_upper
                }
            }
            None => {
                if nse_indices.contains(&sym_upper.as_str()) {
                    "NSE_INDEX".to_string()
                } else if bse_indices.contains(&sym_upper.as_str()) {
                    "BSE_INDEX".to_string()
                } else {
                    "NSE".to_string()
                }
            }
        };

        let body = json!({ "symbol": sym_upper, "exchange": exch });
        match self.client.post("/symbol", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting symbol info: {e}")),
        }
    }

    #[tool(description = "Get expiry dates for derivatives.

Args:
  symbol: Underlying symbol
  exchange: Exchange (typically NFO)
  instrument_type: 'options' or 'futures'")]
    async fn get_expiry_dates(
        &self,
        #[tool(param, description = "Underlying symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: Option<String>,
        #[tool(param, description = "options or futures")] instrument_type: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.unwrap_or_else(|| "NFO".into()).to_uppercase(),
            "instrumenttype": instrument_type.unwrap_or_else(|| "options".into()).to_lowercase(),
        });
        match self.client.post("/expiry", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting expiry dates: {e}")),
        }
    }

    #[tool(description = "Get all available time intervals for historical data.")]
    async fn get_available_intervals(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/intervals", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting intervals: {e}")),
        }
    }

    #[tool(description = "Get option symbol for specific strike and expiry.

Args:
  underlying: Underlying symbol (e.g. NIFTY, BANKNIFTY)
  exchange: Exchange for underlying (NSE_INDEX, BSE_INDEX)
  expiry_date: Expiry date DDMMMYY (e.g. 28OCT25)
  offset: Strike offset - ATM, ITM1-ITM10, OTM1-OTM10
  option_type: CE for Call or PE for Put")]
    async fn get_option_symbol(
        &self,
        #[tool(param, description = "Underlying symbol")] underlying: String,
        #[tool(param, description = "Exchange for underlying")] exchange: String,
        #[tool(param, description = "Expiry DDMMMYY")] expiry_date: String,
        #[tool(param, description = "ATM, ITM1-ITM10, OTM1-OTM10")] offset: String,
        #[tool(param, description = "CE or PE")] option_type: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "underlying": underlying.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "expiry_date": expiry_date,
            "offset": offset.to_uppercase(),
            "option_type": option_type.to_uppercase(),
        });
        match self.client.post("/optionsymbol", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting option symbol: {e}")),
        }
    }

    #[tool(description = "Calculate synthetic future price using put-call parity.

Args:
  underlying: Underlying symbol (e.g. NIFTY, BANKNIFTY)
  exchange: Exchange for underlying (NSE_INDEX, BSE_INDEX)
  expiry_date: Expiry date DDMMMYY (e.g. 25NOV25)")]
    async fn get_synthetic_future(
        &self,
        #[tool(param, description = "Underlying symbol")] underlying: String,
        #[tool(param, description = "Exchange for underlying")] exchange: String,
        #[tool(param, description = "Expiry DDMMMYY")] expiry_date: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "underlying": underlying.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "expiry_date": expiry_date,
        });
        match self.client.post("/syntheticfuture", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating synthetic future: {e}")),
        }
    }

    #[tool(description = "Calculate option Greeks (delta, gamma, theta, vega, rho).

Args:
  symbol: Option symbol (e.g. NIFTY25NOV2526000CE)
  exchange: Exchange (typically NFO)
  underlying_symbol: Underlying symbol (e.g. NIFTY)
  underlying_exchange: Underlying exchange (NSE_INDEX)
  interest_rate: Risk-free interest rate (default: 0.0)")]
    async fn get_option_greeks(
        &self,
        #[tool(param, description = "Option symbol")] symbol: String,
        #[tool(param, description = "Exchange")] exchange: String,
        #[tool(param, description = "Underlying symbol")] underlying_symbol: String,
        #[tool(param, description = "Underlying exchange")] underlying_exchange: String,
        #[tool(param, description = "Interest rate")] interest_rate: Option<f64>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": symbol.to_uppercase(),
            "exchange": exchange.to_uppercase(),
            "interest_rate": interest_rate.unwrap_or(0.0),
            "underlying_symbol": underlying_symbol.to_uppercase(),
            "underlying_exchange": underlying_exchange.to_uppercase(),
        });
        match self.client.post("/optiongreeks", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating option greeks: {e}")),
        }
    }

    #[tool(description = "Download all instruments for an exchange. Returns a large dataset — use search_instruments for specific queries.

Args:
  exchange: Exchange name (NSE, BSE, NFO, BFO, MCX, CDS, BCD, NCDEX)")]
    async fn get_instruments(
        &self,
        #[tool(param, description = "Exchange name")] exchange: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "exchange": exchange.to_uppercase() });
        match self.client.post("/instruments", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting instruments: {e}")),
        }
    }

    #[tool(description = "Get common index symbols for NSE or BSE.

Args:
  exchange: NSE or BSE")]
    async fn get_index_symbols(
        &self,
        #[tool(param, description = "NSE or BSE")] exchange: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let exch = exchange.unwrap_or_else(|| "NSE".into()).to_uppercase();
        let result = match exch.as_str() {
            "NSE" => json!({
                "exchange": "NSE",
                "exchange_code": "NSE_INDEX",
                "indices": ["NIFTY", "NIFTYNXT50", "FINNIFTY", "BANKNIFTY", "MIDCPNIFTY", "INDIAVIX"]
            }),
            "BSE" => json!({
                "exchange": "BSE",
                "exchange_code": "BSE_INDEX",
                "indices": ["SENSEX", "BANKEX", "SENSEX50"]
            }),
            _ => json!({ "error": format!("Unknown exchange: {}. Use NSE or BSE.", exch) }),
        };
        Self::ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  UTILITY TOOLS
    // ═══════════════════════════════════════════════════════════════════════

    #[tool(description = "Display all valid order constants for reference (exchanges, product types, price types, actions, intervals).")]
    async fn validate_order_constants(&self) -> Result<CallToolResult, McpError> {
        Self::ok(json!({
            "exchanges": {
                "NSE": "NSE Equity",
                "NFO": "NSE Futures & Options",
                "CDS": "NSE Currency",
                "BSE": "BSE Equity",
                "BFO": "BSE Futures & Options",
                "BCD": "BSE Currency",
                "MCX": "MCX Commodity",
                "NCDEX": "NCDEX Commodity"
            },
            "product_types": {
                "CNC": "Cash & Carry for equity",
                "NRML": "Normal for futures and options",
                "MIS": "Intraday Square off"
            },
            "price_types": {
                "MARKET": "Market Order",
                "LIMIT": "Limit Order",
                "SL": "Stop Loss Limit Order",
                "SL-M": "Stop Loss Market Order"
            },
            "actions": { "BUY": "Buy", "SELL": "Sell" },
            "intervals": ["1m", "3m", "5m", "10m", "15m", "30m", "1h", "D"]
        }))
    }

    #[tool(description = "Send a Telegram alert notification.

Args:
  username: OpenAlgo login ID/username
  message: Alert message to send")]
    async fn send_telegram_alert(
        &self,
        #[tool(param, description = "OpenAlgo login ID")] username: String,
        #[tool(param, description = "Alert message")] message: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "username": username, "message": message });
        match self.client.post("/telegram", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error sending telegram alert: {e}")),
        }
    }

    #[tool(description = "Get trading holidays for a specific year.

Args:
  year: Year (e.g. 2025)")]
    async fn get_holidays(
        &self,
        #[tool(param, description = "Year e.g. 2025")] year: i64,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "year": year });
        match self.client.post("/holidays", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting holidays: {e}")),
        }
    }

    #[tool(description = "Get exchange trading timings for a specific date.

Args:
  date: Date in YYYY-MM-DD format (e.g. 2025-12-23)")]
    async fn get_timings(
        &self,
        #[tool(param, description = "Date YYYY-MM-DD")] date: String,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "date": date });
        match self.client.post("/timings", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting timings: {e}")),
        }
    }

    #[tool(description = "Get the current analyzer status including mode and total logs.")]
    async fn analyzer_status(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/analyzerstatus", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting analyzer status: {e}")),
        }
    }

    #[tool(description = "Toggle the analyzer mode between analyze (simulated) and live trading.

Args:
  mode: true for analyze mode (simulated), false for live mode")]
    async fn analyzer_toggle(
        &self,
        #[tool(param, description = "true=analyze, false=live")] mode: bool,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "mode": mode });
        match self.client.post("/analyzertoggle", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error toggling analyzer: {e}")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  SERVER HANDLER
// ═══════════════════════════════════════════════════════════════════════════

#[tool_handler]
impl ServerHandler for OpenAlgoMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "OpenAlgo MCP Server — AI-assisted trading via OpenAlgo. \
                 Provides tools for order management, position tracking, market data, \
                 instrument search, option chain analysis, and more."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  MAIN
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("openalgo_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    // Read configuration from environment
    let api_key = env::var("OPENALGO_API_KEY").unwrap_or_else(|_| {
        // Fallback: try command-line args (for backward compat with Python version)
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 {
            args[1].clone()
        } else {
            eprintln!("Error: OPENALGO_API_KEY environment variable or first CLI argument required");
            std::process::exit(1);
        }
    });

    let host = env::var("OPENALGO_URL").unwrap_or_else(|_| {
        let args: Vec<String> = env::args().collect();
        if args.len() >= 3 {
            args[2].clone()
        } else {
            "http://127.0.0.1:5000".to_string()
        }
    });

    eprintln!("OpenAlgo MCP Server starting...");
    eprintln!("  Host: {host}");
    eprintln!("  API Key: {}...", &api_key[..api_key.len().min(8)]);

    let client = OpenAlgoClient::new(api_key, host);
    let server = OpenAlgoMcp::new(client);

    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("Error starting MCP server: {e}"))?;

    eprintln!("OpenAlgo MCP Server running on stdio transport");
    service.waiting().await?;

    Ok(())
}
