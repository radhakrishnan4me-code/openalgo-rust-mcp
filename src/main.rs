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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

// ═══════════════════════════════════════════════════════════════════════════
//  PARAMETER STRUCTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Deserialize, JsonSchema)]
struct PlaceOrderParams {
    /// Stock symbol (e.g. RELIANCE)
    symbol: String,
    /// Number of shares
    quantity: i64,
    /// BUY or SELL
    action: String,
    /// Exchange: NSE, NFO, CDS, BSE, BFO, BCD, MCX, NCDEX
    #[serde(default = "default_nse")]
    exchange: String,
    /// MARKET, LIMIT, SL, SL-M
    #[serde(default = "default_market")]
    price_type: String,
    /// CNC, NRML, MIS
    #[serde(default = "default_mis")]
    product: String,
    /// Strategy name
    #[serde(default = "default_rust")]
    strategy: String,
    /// Limit price (for LIMIT orders)
    price: Option<f64>,
    /// Trigger price (for SL orders)
    trigger_price: Option<f64>,
    /// Disclosed quantity
    disclosed_quantity: Option<i64>,
}

#[derive(Deserialize, JsonSchema)]
struct SmartOrderParams {
    symbol: String,
    quantity: i64,
    action: String,
    position_size: i64,
    #[serde(default = "default_nse")]
    exchange: String,
    #[serde(default = "default_market")]
    price_type: String,
    #[serde(default = "default_mis")]
    product: String,
    #[serde(default = "default_rust")]
    strategy: String,
    price: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct BasketOrderParams {
    /// JSON array of order objects
    orders: Value,
    #[serde(default = "default_rust")]
    strategy: String,
}

#[derive(Deserialize, JsonSchema)]
struct SplitOrderParams {
    symbol: String,
    quantity: i64,
    split_size: i64,
    action: String,
    #[serde(default = "default_nse")]
    exchange: String,
    #[serde(default = "default_market")]
    price_type: String,
    #[serde(default = "default_mis")]
    product: String,
    #[serde(default = "default_rust")]
    strategy: String,
    price: Option<f64>,
    trigger_price: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct OptionsOrderParams {
    /// Underlying symbol e.g. NIFTY, BANKNIFTY
    underlying: String,
    /// Exchange: NSE_INDEX, BSE_INDEX, NFO
    exchange: String,
    /// ATM, ITM1-ITM50, OTM1-OTM50
    offset: String,
    /// CE or PE
    option_type: String,
    /// BUY or SELL
    action: String,
    /// Number of lots
    quantity: i64,
    /// Expiry DDMMMYY e.g. 28OCT25
    expiry_date: Option<String>,
    #[serde(default = "default_rust")]
    strategy: String,
    #[serde(default = "default_market")]
    price_type: String,
    #[serde(default = "default_mis")]
    product: String,
    price: Option<f64>,
    trigger_price: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct OptionsMultiOrderParams {
    strategy: String,
    underlying: String,
    exchange: String,
    /// JSON array of leg objects
    legs: Value,
    expiry_date: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct ModifyOrderParams {
    order_id: String,
    strategy: String,
    symbol: String,
    action: String,
    exchange: String,
    price_type: String,
    product: String,
    quantity: i64,
    price: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct OrderIdStrategy {
    order_id: String,
    strategy: String,
}

#[derive(Deserialize, JsonSchema)]
struct StrategyOnly {
    strategy: String,
}

#[derive(Deserialize, JsonSchema)]
struct OpenPositionParams {
    strategy: String,
    symbol: String,
    exchange: String,
    product: String,
}

#[derive(Deserialize, JsonSchema)]
struct PositionsJson {
    /// JSON array of position objects
    positions: Value,
}

#[derive(Deserialize, JsonSchema)]
struct SymbolExchange {
    symbol: String,
    #[serde(default = "default_nse")]
    exchange: String,
}

#[derive(Deserialize, JsonSchema)]
struct MultiQuotesParams {
    /// Array of {symbol, exchange} objects
    symbols: Value,
}

#[derive(Deserialize, JsonSchema)]
struct OptionChainParams {
    underlying: String,
    exchange: String,
    /// Expiry DDMMMYY
    expiry_date: String,
    /// Strikes around ATM (1-100)
    strike_count: Option<i64>,
}

#[derive(Deserialize, JsonSchema)]
struct HistoryParams {
    symbol: String,
    exchange: String,
    /// 1m, 3m, 5m, 10m, 15m, 30m, 1h, D
    interval: String,
    /// YYYY-MM-DD
    start_date: String,
    /// YYYY-MM-DD
    end_date: String,
}

#[derive(Deserialize, JsonSchema)]
struct SearchParams {
    query: String,
    #[serde(default = "default_nse")]
    exchange: String,
}

#[derive(Deserialize, JsonSchema)]
struct ExpiryParams {
    symbol: String,
    #[serde(default = "default_nfo")]
    exchange: String,
    /// options or futures
    #[serde(default = "default_options")]
    instrument_type: String,
}

#[derive(Deserialize, JsonSchema)]
struct OptionSymbolParams {
    underlying: String,
    exchange: String,
    expiry_date: String,
    /// ATM, ITM1-ITM10, OTM1-OTM10
    offset: String,
    /// CE or PE
    option_type: String,
}

#[derive(Deserialize, JsonSchema)]
struct SyntheticFutureParams {
    underlying: String,
    exchange: String,
    expiry_date: String,
}

#[derive(Deserialize, JsonSchema)]
struct OptionGreeksParams {
    /// Option symbol e.g. NIFTY25NOV2526000CE
    symbol: String,
    /// Exchange e.g. NFO
    exchange: String,
    /// Underlying symbol e.g. NIFTY
    underlying_symbol: String,
    /// Underlying exchange e.g. NSE_INDEX
    underlying_exchange: String,
    #[serde(default)]
    interest_rate: f64,
}

#[derive(Deserialize, JsonSchema)]
struct ExchangeOnly {
    exchange: String,
}

#[derive(Deserialize, JsonSchema)]
struct TelegramParams {
    username: String,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct YearParam {
    year: i64,
}

#[derive(Deserialize, JsonSchema)]
struct DateParam {
    /// YYYY-MM-DD
    date: String,
}

#[derive(Deserialize, JsonSchema)]
struct AnalyzerToggleParam {
    /// true for analyze mode, false for live
    mode: bool,
}

// Default value helpers
fn default_nse() -> String { "NSE".into() }
fn default_nfo() -> String { "NFO".into() }
fn default_market() -> String { "MARKET".into() }
fn default_mis() -> String { "MIS".into() }
fn default_rust() -> String { "Rust".into() }
fn default_options() -> String { "options".into() }

// ═══════════════════════════════════════════════════════════════════════════
//  MCP SERVER
// ═══════════════════════════════════════════════════════════════════════════

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

    fn ok(val: Value) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&val).unwrap_or_default(),
        )]))
    }

    fn err(msg: String) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // ── ORDER MANAGEMENT ──────────────────────────────────────────────────

    #[tool(description = "Place a new order (market or limit). Required: symbol, quantity, action. Optional: exchange (NSE), price_type (MARKET), product (MIS), strategy (Rust), price, trigger_price, disclosed_quantity.")]
    async fn place_order(&self, #[tool(aggr)] p: PlaceOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": p.symbol.to_uppercase(),
            "action": p.action.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "price_type": p.price_type.to_uppercase(),
            "product": p.product.to_uppercase(),
            "strategy": p.strategy,
            "quantity": p.quantity.to_string(),
        });
        if let Some(v) = p.price { body["price"] = json!(v.to_string()); }
        if let Some(v) = p.trigger_price { body["trigger_price"] = json!(v.to_string()); }
        if let Some(v) = p.disclosed_quantity { body["disclosed_quantity"] = json!(v.to_string()); }
        match self.client.post("/placeorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing order: {e}")),
        }
    }

    #[tool(description = "Place a smart order considering current position size. Required: symbol, quantity, action, position_size.")]
    async fn place_smart_order(&self, #[tool(aggr)] p: SmartOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": p.symbol.to_uppercase(),
            "action": p.action.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "price_type": p.price_type.to_uppercase(),
            "product": p.product.to_uppercase(),
            "strategy": p.strategy,
            "quantity": p.quantity.to_string(),
            "position_size": p.position_size.to_string(),
        });
        if let Some(v) = p.price { body["price"] = json!(v.to_string()); }
        match self.client.post("/placesmartorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing smart order: {e}")),
        }
    }

    #[tool(description = "Place multiple orders in a basket. orders: array of {symbol, exchange, action, quantity, pricetype?, product?}. Example: [{\"symbol\":\"BHEL\",\"exchange\":\"NSE\",\"action\":\"BUY\",\"quantity\":1}]")]
    async fn place_basket_order(&self, #[tool(aggr)] p: BasketOrderParams) -> Result<CallToolResult, McpError> {
        let body = json!({ "strategy": p.strategy, "orders": p.orders });
        match self.client.post("/basketorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing basket order: {e}")),
        }
    }

    #[tool(description = "Split a large order into smaller chunks. Required: symbol, quantity, split_size, action.")]
    async fn place_split_order(&self, #[tool(aggr)] p: SplitOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "symbol": p.symbol.to_uppercase(),
            "action": p.action.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "price_type": p.price_type.to_uppercase(),
            "product": p.product.to_uppercase(),
            "strategy": p.strategy,
            "quantity": p.quantity.to_string(),
            "splitsize": p.split_size.to_string(),
        });
        if let Some(v) = p.price { body["price"] = json!(v.to_string()); }
        if let Some(v) = p.trigger_price { body["trigger_price"] = json!(v.to_string()); }
        match self.client.post("/splitorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing split order: {e}")),
        }
    }

    #[tool(description = "Place an options order with ATM/ITM/OTM offset. Required: underlying, exchange, offset (ATM/ITM1-50/OTM1-50), option_type (CE/PE), action, quantity.")]
    async fn place_options_order(&self, #[tool(aggr)] p: OptionsOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "underlying": p.underlying.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "offset": p.offset.to_uppercase(),
            "option_type": p.option_type.to_uppercase(),
            "action": p.action.to_uppercase(),
            "quantity": p.quantity.to_string(),
            "strategy": p.strategy,
            "price_type": p.price_type.to_uppercase(),
            "product": p.product.to_uppercase(),
        });
        if let Some(v) = p.expiry_date { body["expiry_date"] = json!(v); }
        if let Some(v) = p.price { body["price"] = json!(v.to_string()); }
        if let Some(v) = p.trigger_price { body["trigger_price"] = json!(v.to_string()); }
        match self.client.post("/optionsorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing options order: {e}")),
        }
    }

    #[tool(description = "Place a multi-leg options order (spreads, iron condor). Required: strategy, underlying, exchange, legs (array of {offset, option_type, action, quantity}).")]
    async fn place_options_multi_order(&self, #[tool(aggr)] p: OptionsMultiOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "strategy": p.strategy,
            "underlying": p.underlying.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "legs": p.legs,
        });
        if let Some(v) = p.expiry_date { body["expiry_date"] = json!(v); }
        match self.client.post("/optionsmultiorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error placing options multi order: {e}")),
        }
    }

    #[tool(description = "Modify an existing order. Required: order_id, strategy, symbol, action, exchange, price_type, product, quantity.")]
    async fn modify_order(&self, #[tool(aggr)] p: ModifyOrderParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "order_id": p.order_id,
            "strategy": p.strategy,
            "symbol": p.symbol.to_uppercase(),
            "action": p.action.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "price_type": p.price_type.to_uppercase(),
            "product": p.product.to_uppercase(),
            "quantity": p.quantity.to_string(),
        });
        if let Some(v) = p.price { body["price"] = json!(v.to_string()); }
        match self.client.post("/modifyorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error modifying order: {e}")),
        }
    }

    #[tool(description = "Cancel a specific order. Required: order_id, strategy.")]
    async fn cancel_order(&self, #[tool(aggr)] p: OrderIdStrategy) -> Result<CallToolResult, McpError> {
        let body = json!({ "order_id": p.order_id, "strategy": p.strategy });
        match self.client.post("/cancelorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error canceling order: {e}")),
        }
    }

    #[tool(description = "Cancel all open orders for a strategy. Required: strategy.")]
    async fn cancel_all_orders(&self, #[tool(aggr)] p: StrategyOnly) -> Result<CallToolResult, McpError> {
        let body = json!({ "strategy": p.strategy });
        match self.client.post("/cancelallorder", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error canceling all orders: {e}")),
        }
    }

    // ── POSITION MANAGEMENT ───────────────────────────────────────────────

    #[tool(description = "Close all open positions for a strategy. Required: strategy.")]
    async fn close_all_positions(&self, #[tool(aggr)] p: StrategyOnly) -> Result<CallToolResult, McpError> {
        let body = json!({ "strategy": p.strategy });
        match self.client.post("/closeposition", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error closing positions: {e}")),
        }
    }

    #[tool(description = "Get current open position for an instrument. Required: strategy, symbol, exchange, product.")]
    async fn get_open_position(&self, #[tool(aggr)] p: OpenPositionParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "strategy": p.strategy,
            "symbol": p.symbol.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "product": p.product.to_uppercase(),
        });
        match self.client.post("/openposition", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting open position: {e}")),
        }
    }

    // ── ORDER STATUS & TRACKING ───────────────────────────────────────────

    #[tool(description = "Get status of a specific order. Required: order_id, strategy.")]
    async fn get_order_status(&self, #[tool(aggr)] p: OrderIdStrategy) -> Result<CallToolResult, McpError> {
        let body = json!({ "order_id": p.order_id, "strategy": p.strategy });
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

    #[tool(description = "Calculate margin requirements. positions: array of {symbol, exchange, action, product, pricetype, quantity}.")]
    async fn calculate_margin(&self, #[tool(aggr)] p: PositionsJson) -> Result<CallToolResult, McpError> {
        let body = json!({ "positions": p.positions });
        match self.client.post("/margin", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating margin: {e}")),
        }
    }

    // ── MARKET DATA ───────────────────────────────────────────────────────

    #[tool(description = "Get current quote for a symbol. Required: symbol. Optional: exchange (NSE).")]
    async fn get_quote(&self, #[tool(aggr)] p: SymbolExchange) -> Result<CallToolResult, McpError> {
        let body = json!({ "symbol": p.symbol.to_uppercase(), "exchange": p.exchange.to_uppercase() });
        match self.client.post("/quotes", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting quote: {e}")),
        }
    }

    #[tool(description = "Get quotes for multiple symbols. symbols: array of {symbol, exchange}.")]
    async fn get_multi_quotes(&self, #[tool(aggr)] p: MultiQuotesParams) -> Result<CallToolResult, McpError> {
        let body = json!({ "symbols": p.symbols });
        match self.client.post("/multiquotes", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting multi quotes: {e}")),
        }
    }

    #[tool(description = "Get option chain data. Required: underlying, exchange, expiry_date (DDMMMYY). Optional: strike_count (1-100).")]
    async fn get_option_chain(&self, #[tool(aggr)] p: OptionChainParams) -> Result<CallToolResult, McpError> {
        let mut body = json!({
            "underlying": p.underlying.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "expiry_date": p.expiry_date.to_uppercase(),
        });
        if let Some(sc) = p.strike_count { body["strike_count"] = json!(sc); }
        match self.client.post("/optionchain", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting option chain: {e}")),
        }
    }

    #[tool(description = "Get market depth (order book) for a symbol. Required: symbol. Optional: exchange (NSE).")]
    async fn get_market_depth(&self, #[tool(aggr)] p: SymbolExchange) -> Result<CallToolResult, McpError> {
        let body = json!({ "symbol": p.symbol.to_uppercase(), "exchange": p.exchange.to_uppercase() });
        match self.client.post("/depth", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting market depth: {e}")),
        }
    }

    #[tool(description = "Get historical price data. Required: symbol, exchange, interval (1m/3m/5m/10m/15m/30m/1h/D), start_date (YYYY-MM-DD), end_date.")]
    async fn get_historical_data(&self, #[tool(aggr)] p: HistoryParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": p.symbol.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "interval": p.interval,
            "start_date": p.start_date,
            "end_date": p.end_date,
        });
        match self.client.post("/history", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting historical data: {e}")),
        }
    }

    // ── INSTRUMENT SEARCH ─────────────────────────────────────────────────

    #[tool(description = "Search for instruments by name. Required: query. Optional: exchange (NSE).")]
    async fn search_instruments(&self, #[tool(aggr)] p: SearchParams) -> Result<CallToolResult, McpError> {
        let body = json!({ "query": p.query, "exchange": p.exchange.to_uppercase() });
        match self.client.post("/search", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error searching instruments: {e}")),
        }
    }

    #[tool(description = "Get detailed info about a symbol. Required: symbol. Optional: exchange (auto-detects indices).")]
    async fn get_symbol_info(&self, #[tool(aggr)] p: SymbolExchange) -> Result<CallToolResult, McpError> {
        let sym = p.symbol.to_uppercase();
        let nse_idx = ["NIFTY","NIFTYNXT50","FINNIFTY","BANKNIFTY","MIDCPNIFTY","INDIAVIX"];
        let bse_idx = ["SENSEX","BANKEX","SENSEX50"];
        let exch = {
            let e = p.exchange.to_uppercase();
            if nse_idx.contains(&sym.as_str()) && e == "NSE" { "NSE_INDEX".into() }
            else if bse_idx.contains(&sym.as_str()) && e == "BSE" { "BSE_INDEX".into() }
            else { e }
        };
        let body = json!({ "symbol": sym, "exchange": exch });
        match self.client.post("/symbol", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting symbol info: {e}")),
        }
    }

    #[tool(description = "Get expiry dates for derivatives. Required: symbol. Optional: exchange (NFO), instrument_type (options/futures).")]
    async fn get_expiry_dates(&self, #[tool(aggr)] p: ExpiryParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": p.symbol.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "instrumenttype": p.instrument_type.to_lowercase(),
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

    #[tool(description = "Get option symbol for specific strike/expiry. Required: underlying, exchange, expiry_date (DDMMMYY), offset (ATM/ITM1-10/OTM1-10), option_type (CE/PE).")]
    async fn get_option_symbol(&self, #[tool(aggr)] p: OptionSymbolParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "underlying": p.underlying.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "expiry_date": p.expiry_date,
            "offset": p.offset.to_uppercase(),
            "option_type": p.option_type.to_uppercase(),
        });
        match self.client.post("/optionsymbol", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting option symbol: {e}")),
        }
    }

    #[tool(description = "Calculate synthetic future price using put-call parity. Required: underlying, exchange, expiry_date (DDMMMYY).")]
    async fn get_synthetic_future(&self, #[tool(aggr)] p: SyntheticFutureParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "underlying": p.underlying.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "expiry_date": p.expiry_date,
        });
        match self.client.post("/syntheticfuture", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating synthetic future: {e}")),
        }
    }

    #[tool(description = "Calculate option Greeks (delta, gamma, theta, vega, rho). Required: symbol, exchange, underlying_symbol, underlying_exchange. Optional: interest_rate (0.0).")]
    async fn get_option_greeks(&self, #[tool(aggr)] p: OptionGreeksParams) -> Result<CallToolResult, McpError> {
        let body = json!({
            "symbol": p.symbol.to_uppercase(),
            "exchange": p.exchange.to_uppercase(),
            "interest_rate": p.interest_rate,
            "underlying_symbol": p.underlying_symbol.to_uppercase(),
            "underlying_exchange": p.underlying_exchange.to_uppercase(),
        });
        match self.client.post("/optiongreeks", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error calculating option greeks: {e}")),
        }
    }

    #[tool(description = "Download all instruments for an exchange (large dataset). Required: exchange (NSE/BSE/NFO/BFO/MCX/CDS/BCD/NCDEX).")]
    async fn get_instruments(&self, #[tool(aggr)] p: ExchangeOnly) -> Result<CallToolResult, McpError> {
        let body = json!({ "exchange": p.exchange.to_uppercase() });
        match self.client.post("/instruments", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting instruments: {e}")),
        }
    }

    #[tool(description = "Get common index symbols for an exchange. Required: exchange (NSE or BSE).")]
    async fn get_index_symbols(&self, #[tool(aggr)] p: ExchangeOnly) -> Result<CallToolResult, McpError> {
        let exch = p.exchange.to_uppercase();
        let result = match exch.as_str() {
            "NSE" => json!({"exchange":"NSE","exchange_code":"NSE_INDEX","indices":["NIFTY","NIFTYNXT50","FINNIFTY","BANKNIFTY","MIDCPNIFTY","INDIAVIX"]}),
            "BSE" => json!({"exchange":"BSE","exchange_code":"BSE_INDEX","indices":["SENSEX","BANKEX","SENSEX50"]}),
            _ => json!({"error": format!("Unknown exchange: {}. Use NSE or BSE.", exch)}),
        };
        Self::ok(result)
    }

    // ── UTILITIES ─────────────────────────────────────────────────────────

    #[tool(description = "Display all valid order constants (exchanges, product types, price types, actions, intervals).")]
    async fn validate_order_constants(&self) -> Result<CallToolResult, McpError> {
        Self::ok(json!({
            "exchanges": {"NSE":"NSE Equity","NFO":"NSE F&O","CDS":"NSE Currency","BSE":"BSE Equity","BFO":"BSE F&O","BCD":"BSE Currency","MCX":"MCX Commodity","NCDEX":"NCDEX Commodity"},
            "product_types": {"CNC":"Cash & Carry","NRML":"Normal F&O","MIS":"Intraday"},
            "price_types": {"MARKET":"Market Order","LIMIT":"Limit Order","SL":"Stop Loss Limit","SL-M":"Stop Loss Market"},
            "actions": {"BUY":"Buy","SELL":"Sell"},
            "intervals": ["1m","3m","5m","10m","15m","30m","1h","D"]
        }))
    }

    #[tool(description = "Send a Telegram alert. Required: username (OpenAlgo login ID), message.")]
    async fn send_telegram_alert(&self, #[tool(aggr)] p: TelegramParams) -> Result<CallToolResult, McpError> {
        let body = json!({ "username": p.username, "message": p.message });
        match self.client.post("/telegram", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error sending telegram alert: {e}")),
        }
    }

    #[tool(description = "Get trading holidays for a year. Required: year (e.g. 2025).")]
    async fn get_holidays(&self, #[tool(aggr)] p: YearParam) -> Result<CallToolResult, McpError> {
        let body = json!({ "year": p.year });
        match self.client.post("/holidays", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting holidays: {e}")),
        }
    }

    #[tool(description = "Get exchange trading timings for a date. Required: date (YYYY-MM-DD).")]
    async fn get_timings(&self, #[tool(aggr)] p: DateParam) -> Result<CallToolResult, McpError> {
        let body = json!({ "date": p.date });
        match self.client.post("/timings", body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting timings: {e}")),
        }
    }

    #[tool(description = "Get the current analyzer mode status.")]
    async fn analyzer_status(&self) -> Result<CallToolResult, McpError> {
        match self.client.post("/analyzerstatus", json!({})).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("Error getting analyzer status: {e}")),
        }
    }

    #[tool(description = "Toggle analyzer mode. Required: mode (true=analyze/simulated, false=live).")]
    async fn analyzer_toggle(&self, #[tool(aggr)] p: AnalyzerToggleParam) -> Result<CallToolResult, McpError> {
        let body = json!({ "mode": p.mode });
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
                 Tools for orders, positions, market data, instruments, and utilities."
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("openalgo_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    let api_key = env::var("OPENALGO_API_KEY").unwrap_or_else(|_| {
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 { args[1].clone() }
        else { eprintln!("Error: OPENALGO_API_KEY env var or first CLI arg required"); std::process::exit(1); }
    });

    let host = env::var("OPENALGO_URL").unwrap_or_else(|_| {
        let args: Vec<String> = env::args().collect();
        if args.len() >= 3 { args[2].clone() } else { "http://127.0.0.1:5000".into() }
    });

    eprintln!("OpenAlgo MCP Server starting...");
    eprintln!("  Host: {host}");
    eprintln!("  API Key: {}...", &api_key[..api_key.len().min(8)]);

    let client = OpenAlgoClient::new(api_key, host);
    let server = OpenAlgoMcp::new(client);

    let service = server.serve(stdio()).await
        .inspect_err(|e| eprintln!("Error starting MCP server: {e}"))?;

    eprintln!("OpenAlgo MCP Server running on stdio transport");
    service.waiting().await?;
    Ok(())
}
