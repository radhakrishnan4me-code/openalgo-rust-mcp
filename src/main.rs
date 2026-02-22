mod client;

use client::OpenAlgoClient;
use rmcp::{
    ServerHandler, ServiceExt,
    model::*,
    service::{RequestContext, RoleServer},
    transport::stdio,
};
use serde_json::{json, Value};
use std::env;

// ═══════════════════════════════════════════════════════════════════════════
//  MCP SERVER
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct OpenAlgoMcp {
    client: OpenAlgoClient,
}

impl OpenAlgoMcp {
    fn new(client: OpenAlgoClient) -> Self {
        Self { client }
    }

    fn ok(val: Value) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&val).unwrap_or_default(),
        )]))
    }

    fn err(msg: String) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    fn get_str(args: &Value, key: &str) -> String {
        args.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
    }
    fn get_str_or(args: &Value, key: &str, default: &str) -> String {
        let v = args.get(key).and_then(|v| v.as_str()).unwrap_or(default);
        if v.is_empty() { default.to_string() } else { v.to_string() }
    }
    fn get_i64(args: &Value, key: &str) -> i64 {
        args.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
    }
    fn get_f64_opt(args: &Value, key: &str) -> Option<f64> {
        args.get(key).and_then(|v| v.as_f64())
    }
    fn get_i64_opt(args: &Value, key: &str) -> Option<i64> {
        args.get(key).and_then(|v| v.as_i64())
    }
    fn get_bool(args: &Value, key: &str) -> bool {
        args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }

    // ── Tool definitions ──────────────────────────────────────────────────

    fn tool_definitions() -> Vec<Tool> {
        vec![
            // Order Management
            Self::make_tool("place_order", "Place a new order (market or limit). Required: symbol, quantity, action. Optional: exchange (NSE), price_type (MARKET), product (MIS), strategy (Rust), price, trigger_price, disclosed_quantity.", json!({"type":"object","required":["symbol","quantity","action"],"properties":{"symbol":{"type":"string"},"quantity":{"type":"integer"},"action":{"type":"string","enum":["BUY","SELL"]},"exchange":{"type":"string","default":"NSE"},"price_type":{"type":"string","default":"MARKET"},"product":{"type":"string","default":"MIS"},"strategy":{"type":"string","default":"Rust"},"price":{"type":"number"},"trigger_price":{"type":"number"},"disclosed_quantity":{"type":"integer"}}})),
            Self::make_tool("place_smart_order", "Place a smart order considering current position size. Required: symbol, quantity, action, position_size.", json!({"type":"object","required":["symbol","quantity","action","position_size"],"properties":{"symbol":{"type":"string"},"quantity":{"type":"integer"},"action":{"type":"string"},"position_size":{"type":"integer"},"exchange":{"type":"string","default":"NSE"},"price_type":{"type":"string","default":"MARKET"},"product":{"type":"string","default":"MIS"},"strategy":{"type":"string","default":"Rust"},"price":{"type":"number"}}})),
            Self::make_tool("place_basket_order", "Place multiple orders in a basket. orders: array of order objects.", json!({"type":"object","required":["orders"],"properties":{"orders":{"type":"array"},"strategy":{"type":"string","default":"Rust"}}})),
            Self::make_tool("place_split_order", "Split a large order into smaller chunks. Required: symbol, quantity, split_size, action.", json!({"type":"object","required":["symbol","quantity","split_size","action"],"properties":{"symbol":{"type":"string"},"quantity":{"type":"integer"},"split_size":{"type":"integer"},"action":{"type":"string"},"exchange":{"type":"string","default":"NSE"},"price_type":{"type":"string","default":"MARKET"},"product":{"type":"string","default":"MIS"},"strategy":{"type":"string","default":"Rust"},"price":{"type":"number"},"trigger_price":{"type":"number"}}})),
            Self::make_tool("place_options_order", "Place an options order with ATM/ITM/OTM offset. Required: underlying, exchange, offset, option_type, action, quantity.", json!({"type":"object","required":["underlying","exchange","offset","option_type","action","quantity"],"properties":{"underlying":{"type":"string"},"exchange":{"type":"string"},"offset":{"type":"string"},"option_type":{"type":"string"},"action":{"type":"string"},"quantity":{"type":"integer"},"expiry_date":{"type":"string"},"strategy":{"type":"string","default":"Rust"},"price_type":{"type":"string","default":"MARKET"},"product":{"type":"string","default":"MIS"},"price":{"type":"number"},"trigger_price":{"type":"number"}}})),
            Self::make_tool("place_options_multi_order", "Place a multi-leg options order (spreads, iron condor). Required: strategy, underlying, exchange, legs.", json!({"type":"object","required":["strategy","underlying","exchange","legs"],"properties":{"strategy":{"type":"string"},"underlying":{"type":"string"},"exchange":{"type":"string"},"legs":{"type":"array"},"expiry_date":{"type":"string"}}})),
            Self::make_tool("modify_order", "Modify an existing order. Required: order_id, strategy, symbol, action, exchange, price_type, product, quantity.", json!({"type":"object","required":["order_id","strategy","symbol","action","exchange","price_type","product","quantity"],"properties":{"order_id":{"type":"string"},"strategy":{"type":"string"},"symbol":{"type":"string"},"action":{"type":"string"},"exchange":{"type":"string"},"price_type":{"type":"string"},"product":{"type":"string"},"quantity":{"type":"integer"},"price":{"type":"number"}}})),
            Self::make_tool("cancel_order", "Cancel a specific order. Required: order_id, strategy.", json!({"type":"object","required":["order_id","strategy"],"properties":{"order_id":{"type":"string"},"strategy":{"type":"string"}}})),
            Self::make_tool("cancel_all_orders", "Cancel all open orders for a strategy. Required: strategy.", json!({"type":"object","required":["strategy"],"properties":{"strategy":{"type":"string"}}})),
            // Position Management
            Self::make_tool("close_all_positions", "Close all open positions for a strategy. Required: strategy.", json!({"type":"object","required":["strategy"],"properties":{"strategy":{"type":"string"}}})),
            Self::make_tool("get_open_position", "Get current open position. Required: strategy, symbol, exchange, product.", json!({"type":"object","required":["strategy","symbol","exchange","product"],"properties":{"strategy":{"type":"string"},"symbol":{"type":"string"},"exchange":{"type":"string"},"product":{"type":"string"}}})),
            // Order Status & Tracking
            Self::make_tool("get_order_status", "Get status of a specific order. Required: order_id, strategy.", json!({"type":"object","required":["order_id","strategy"],"properties":{"order_id":{"type":"string"},"strategy":{"type":"string"}}})),
            Self::make_tool("get_order_book", "Get all orders from the order book.", json!({"type":"object","properties":{}})),
            Self::make_tool("get_trade_book", "Get all executed trades.", json!({"type":"object","properties":{}})),
            Self::make_tool("get_position_book", "Get all current positions.", json!({"type":"object","properties":{}})),
            Self::make_tool("get_holdings", "Get all holdings (long-term investments).", json!({"type":"object","properties":{}})),
            Self::make_tool("get_funds", "Get account funds and margin information.", json!({"type":"object","properties":{}})),
            Self::make_tool("calculate_margin", "Calculate margin requirements.", json!({"type":"object","required":["positions"],"properties":{"positions":{"type":"array"}}})),
            // Market Data
            Self::make_tool("get_quote", "Get current quote for a symbol. Required: symbol. Optional: exchange (NSE).", json!({"type":"object","required":["symbol"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string","default":"NSE"}}})),
            Self::make_tool("get_multi_quotes", "Get quotes for multiple symbols.", json!({"type":"object","required":["symbols"],"properties":{"symbols":{"type":"array"}}})),
            Self::make_tool("get_option_chain", "Get option chain data. Required: underlying, exchange, expiry_date. Optional: strike_count.", json!({"type":"object","required":["underlying","exchange","expiry_date"],"properties":{"underlying":{"type":"string"},"exchange":{"type":"string"},"expiry_date":{"type":"string"},"strike_count":{"type":"integer"}}})),
            Self::make_tool("get_market_depth", "Get market depth (order book) for a symbol.", json!({"type":"object","required":["symbol"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string","default":"NSE"}}})),
            Self::make_tool("get_historical_data", "Get historical price data. Required: symbol, exchange, interval, start_date, end_date.", json!({"type":"object","required":["symbol","exchange","interval","start_date","end_date"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string"},"interval":{"type":"string"},"start_date":{"type":"string"},"end_date":{"type":"string"}}})),
            // Instrument Search
            Self::make_tool("search_instruments", "Search for instruments by name. Required: query. Optional: exchange.", json!({"type":"object","required":["query"],"properties":{"query":{"type":"string"},"exchange":{"type":"string","default":"NSE"}}})),
            Self::make_tool("get_symbol_info", "Get detailed info about a symbol.", json!({"type":"object","required":["symbol"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string","default":"NSE"}}})),
            Self::make_tool("get_expiry_dates", "Get expiry dates for derivatives. Required: symbol.", json!({"type":"object","required":["symbol"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string","default":"NFO"},"instrument_type":{"type":"string","default":"options"}}})),
            Self::make_tool("get_available_intervals", "Get all available time intervals for historical data.", json!({"type":"object","properties":{}})),
            Self::make_tool("get_option_symbol", "Get option symbol for specific strike/expiry.", json!({"type":"object","required":["underlying","exchange","expiry_date","offset","option_type"],"properties":{"underlying":{"type":"string"},"exchange":{"type":"string"},"expiry_date":{"type":"string"},"offset":{"type":"string"},"option_type":{"type":"string"}}})),
            Self::make_tool("get_synthetic_future", "Calculate synthetic future price using put-call parity.", json!({"type":"object","required":["underlying","exchange","expiry_date"],"properties":{"underlying":{"type":"string"},"exchange":{"type":"string"},"expiry_date":{"type":"string"}}})),
            Self::make_tool("get_option_greeks", "Calculate option Greeks.", json!({"type":"object","required":["symbol","exchange","underlying_symbol","underlying_exchange"],"properties":{"symbol":{"type":"string"},"exchange":{"type":"string"},"underlying_symbol":{"type":"string"},"underlying_exchange":{"type":"string"},"interest_rate":{"type":"number","default":0.0}}})),
            Self::make_tool("get_instruments", "Download all instruments for an exchange.", json!({"type":"object","required":["exchange"],"properties":{"exchange":{"type":"string"}}})),
            Self::make_tool("get_index_symbols", "Get common index symbols for an exchange (NSE or BSE).", json!({"type":"object","required":["exchange"],"properties":{"exchange":{"type":"string"}}})),
            // Utilities
            Self::make_tool("validate_order_constants", "Display all valid order constants.", json!({"type":"object","properties":{}})),
            Self::make_tool("send_telegram_alert", "Send a Telegram alert. Required: username, message.", json!({"type":"object","required":["username","message"],"properties":{"username":{"type":"string"},"message":{"type":"string"}}})),
            Self::make_tool("get_holidays", "Get trading holidays for a year. Required: year.", json!({"type":"object","required":["year"],"properties":{"year":{"type":"integer"}}})),
            Self::make_tool("get_timings", "Get exchange trading timings for a date. Required: date (YYYY-MM-DD).", json!({"type":"object","required":["date"],"properties":{"date":{"type":"string"}}})),
            Self::make_tool("analyzer_status", "Get the current analyzer mode status.", json!({"type":"object","properties":{}})),
            Self::make_tool("analyzer_toggle", "Toggle analyzer mode. Required: mode (true=analyze, false=live).", json!({"type":"object","required":["mode"],"properties":{"mode":{"type":"boolean"}}})),
        ]
    }

    fn make_tool(name: &str, description: &str, schema: Value) -> Tool {
        Tool {
            name: name.into(),
            description: Some(description.into()),
            input_schema: serde_json::from_value(schema).unwrap_or_default(),
            ..Default::default()
        }
    }

    // ── Tool dispatch ─────────────────────────────────────────────────────

    async fn dispatch(&self, name: &str, a: Value) -> Result<CallToolResult, ErrorData> {
        match name {
            "place_order" => {
                let mut body = json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "action": Self::get_str(&a, "action").to_uppercase(),
                    "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase(),
                    "price_type": Self::get_str_or(&a, "price_type", "MARKET").to_uppercase(),
                    "product": Self::get_str_or(&a, "product", "MIS").to_uppercase(),
                    "strategy": Self::get_str_or(&a, "strategy", "Rust"),
                    "quantity": Self::get_i64(&a, "quantity").to_string(),
                });
                if let Some(v) = Self::get_f64_opt(&a, "price") { body["price"] = json!(v.to_string()); }
                if let Some(v) = Self::get_f64_opt(&a, "trigger_price") { body["trigger_price"] = json!(v.to_string()); }
                if let Some(v) = Self::get_i64_opt(&a, "disclosed_quantity") { body["disclosed_quantity"] = json!(v.to_string()); }
                self.call_api("/placeorder", body).await
            }
            "place_smart_order" => {
                let mut body = json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "action": Self::get_str(&a, "action").to_uppercase(),
                    "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase(),
                    "price_type": Self::get_str_or(&a, "price_type", "MARKET").to_uppercase(),
                    "product": Self::get_str_or(&a, "product", "MIS").to_uppercase(),
                    "strategy": Self::get_str_or(&a, "strategy", "Rust"),
                    "quantity": Self::get_i64(&a, "quantity").to_string(),
                    "position_size": Self::get_i64(&a, "position_size").to_string(),
                });
                if let Some(v) = Self::get_f64_opt(&a, "price") { body["price"] = json!(v.to_string()); }
                self.call_api("/placesmartorder", body).await
            }
            "place_basket_order" => {
                let body = json!({
                    "strategy": Self::get_str_or(&a, "strategy", "Rust"),
                    "orders": a.get("orders").cloned().unwrap_or(json!([]))
                });
                self.call_api("/basketorder", body).await
            }
            "place_split_order" => {
                let mut body = json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "action": Self::get_str(&a, "action").to_uppercase(),
                    "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase(),
                    "price_type": Self::get_str_or(&a, "price_type", "MARKET").to_uppercase(),
                    "product": Self::get_str_or(&a, "product", "MIS").to_uppercase(),
                    "strategy": Self::get_str_or(&a, "strategy", "Rust"),
                    "quantity": Self::get_i64(&a, "quantity").to_string(),
                    "splitsize": Self::get_i64(&a, "split_size").to_string(),
                });
                if let Some(v) = Self::get_f64_opt(&a, "price") { body["price"] = json!(v.to_string()); }
                if let Some(v) = Self::get_f64_opt(&a, "trigger_price") { body["trigger_price"] = json!(v.to_string()); }
                self.call_api("/splitorder", body).await
            }
            "place_options_order" => {
                let mut body = json!({
                    "underlying": Self::get_str(&a, "underlying").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "offset": Self::get_str(&a, "offset").to_uppercase(),
                    "option_type": Self::get_str(&a, "option_type").to_uppercase(),
                    "action": Self::get_str(&a, "action").to_uppercase(),
                    "quantity": Self::get_i64(&a, "quantity").to_string(),
                    "strategy": Self::get_str_or(&a, "strategy", "Rust"),
                    "price_type": Self::get_str_or(&a, "price_type", "MARKET").to_uppercase(),
                    "product": Self::get_str_or(&a, "product", "MIS").to_uppercase(),
                });
                let exp = Self::get_str(&a, "expiry_date");
                if !exp.is_empty() { body["expiry_date"] = json!(exp); }
                if let Some(v) = Self::get_f64_opt(&a, "price") { body["price"] = json!(v.to_string()); }
                if let Some(v) = Self::get_f64_opt(&a, "trigger_price") { body["trigger_price"] = json!(v.to_string()); }
                self.call_api("/optionsorder", body).await
            }
            "place_options_multi_order" => {
                let mut body = json!({
                    "strategy": Self::get_str(&a, "strategy"),
                    "underlying": Self::get_str(&a, "underlying").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "legs": a.get("legs").cloned().unwrap_or(json!([])),
                });
                let exp = Self::get_str(&a, "expiry_date");
                if !exp.is_empty() { body["expiry_date"] = json!(exp); }
                self.call_api("/optionsmultiorder", body).await
            }
            "modify_order" => {
                let mut body = json!({
                    "order_id": Self::get_str(&a, "order_id"),
                    "strategy": Self::get_str(&a, "strategy"),
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "action": Self::get_str(&a, "action").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "price_type": Self::get_str(&a, "price_type").to_uppercase(),
                    "product": Self::get_str(&a, "product").to_uppercase(),
                    "quantity": Self::get_i64(&a, "quantity").to_string(),
                });
                if let Some(v) = Self::get_f64_opt(&a, "price") { body["price"] = json!(v.to_string()); }
                self.call_api("/modifyorder", body).await
            }
            "cancel_order" => {
                self.call_api("/cancelorder", json!({"order_id": Self::get_str(&a, "order_id"), "strategy": Self::get_str(&a, "strategy")})).await
            }
            "cancel_all_orders" => {
                self.call_api("/cancelallorder", json!({"strategy": Self::get_str(&a, "strategy")})).await
            }
            "close_all_positions" => {
                self.call_api("/closeposition", json!({"strategy": Self::get_str(&a, "strategy")})).await
            }
            "get_open_position" => {
                self.call_api("/openposition", json!({
                    "strategy": Self::get_str(&a, "strategy"),
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "product": Self::get_str(&a, "product").to_uppercase(),
                })).await
            }
            "get_order_status" => {
                self.call_api("/orderstatus", json!({"order_id": Self::get_str(&a, "order_id"), "strategy": Self::get_str(&a, "strategy")})).await
            }
            "get_order_book" => self.call_api("/orderbook", json!({})).await,
            "get_trade_book" => self.call_api("/tradebook", json!({})).await,
            "get_position_book" => self.call_api("/positionbook", json!({})).await,
            "get_holdings" => self.call_api("/holdings", json!({})).await,
            "get_funds" => self.call_api("/funds", json!({})).await,
            "calculate_margin" => {
                self.call_api("/margin", json!({"positions": a.get("positions").cloned().unwrap_or(json!([]))})).await
            }
            "get_quote" => {
                self.call_api("/quotes", json!({"symbol": Self::get_str(&a, "symbol").to_uppercase(), "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase()})).await
            }
            "get_multi_quotes" => {
                self.call_api("/multiquotes", json!({"symbols": a.get("symbols").cloned().unwrap_or(json!([]))})).await
            }
            "get_option_chain" => {
                let mut body = json!({
                    "underlying": Self::get_str(&a, "underlying").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "expiry_date": Self::get_str(&a, "expiry_date").to_uppercase(),
                });
                if let Some(sc) = Self::get_i64_opt(&a, "strike_count") { body["strike_count"] = json!(sc); }
                self.call_api("/optionchain", body).await
            }
            "get_market_depth" => {
                self.call_api("/depth", json!({"symbol": Self::get_str(&a, "symbol").to_uppercase(), "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase()})).await
            }
            "get_historical_data" => {
                self.call_api("/history", json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "interval": Self::get_str(&a, "interval"),
                    "start_date": Self::get_str(&a, "start_date"),
                    "end_date": Self::get_str(&a, "end_date"),
                })).await
            }
            "search_instruments" => {
                self.call_api("/search", json!({"query": Self::get_str(&a, "query"), "exchange": Self::get_str_or(&a, "exchange", "NSE").to_uppercase()})).await
            }
            "get_symbol_info" => {
                let sym = Self::get_str(&a, "symbol").to_uppercase();
                let nse_idx = ["NIFTY","NIFTYNXT50","FINNIFTY","BANKNIFTY","MIDCPNIFTY","INDIAVIX"];
                let bse_idx = ["SENSEX","BANKEX","SENSEX50"];
                let raw_exch = Self::get_str_or(&a, "exchange", "NSE").to_uppercase();
                let exch = if nse_idx.contains(&sym.as_str()) && raw_exch == "NSE" { "NSE_INDEX".to_string() }
                    else if bse_idx.contains(&sym.as_str()) && raw_exch == "BSE" { "BSE_INDEX".to_string() }
                    else { raw_exch };
                self.call_api("/symbol", json!({"symbol": sym, "exchange": exch})).await
            }
            "get_expiry_dates" => {
                self.call_api("/expiry", json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "exchange": Self::get_str_or(&a, "exchange", "NFO").to_uppercase(),
                    "instrumenttype": Self::get_str_or(&a, "instrument_type", "options").to_lowercase(),
                })).await
            }
            "get_available_intervals" => self.call_api("/intervals", json!({})).await,
            "get_option_symbol" => {
                self.call_api("/optionsymbol", json!({
                    "underlying": Self::get_str(&a, "underlying").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "expiry_date": Self::get_str(&a, "expiry_date"),
                    "offset": Self::get_str(&a, "offset").to_uppercase(),
                    "option_type": Self::get_str(&a, "option_type").to_uppercase(),
                })).await
            }
            "get_synthetic_future" => {
                self.call_api("/syntheticfuture", json!({
                    "underlying": Self::get_str(&a, "underlying").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "expiry_date": Self::get_str(&a, "expiry_date"),
                })).await
            }
            "get_option_greeks" => {
                self.call_api("/optiongreeks", json!({
                    "symbol": Self::get_str(&a, "symbol").to_uppercase(),
                    "exchange": Self::get_str(&a, "exchange").to_uppercase(),
                    "interest_rate": Self::get_f64_opt(&a, "interest_rate").unwrap_or(0.0),
                    "underlying_symbol": Self::get_str(&a, "underlying_symbol").to_uppercase(),
                    "underlying_exchange": Self::get_str(&a, "underlying_exchange").to_uppercase(),
                })).await
            }
            "get_instruments" => {
                self.call_api("/instruments", json!({"exchange": Self::get_str(&a, "exchange").to_uppercase()})).await
            }
            "get_index_symbols" => {
                let exch = Self::get_str(&a, "exchange").to_uppercase();
                match exch.as_str() {
                    "NSE" => Self::ok(json!({"exchange":"NSE","exchange_code":"NSE_INDEX","indices":["NIFTY","NIFTYNXT50","FINNIFTY","BANKNIFTY","MIDCPNIFTY","INDIAVIX"]})),
                    "BSE" => Self::ok(json!({"exchange":"BSE","exchange_code":"BSE_INDEX","indices":["SENSEX","BANKEX","SENSEX50"]})),
                    _ => Self::ok(json!({"error": format!("Unknown exchange: {}. Use NSE or BSE.", exch)})),
                }
            }
            "validate_order_constants" => {
                Self::ok(json!({
                    "exchanges": {"NSE":"NSE Equity","NFO":"NSE F&O","CDS":"NSE Currency","BSE":"BSE Equity","BFO":"BSE F&O","BCD":"BSE Currency","MCX":"MCX Commodity","NCDEX":"NCDEX Commodity"},
                    "product_types": {"CNC":"Cash & Carry","NRML":"Normal F&O","MIS":"Intraday"},
                    "price_types": {"MARKET":"Market","LIMIT":"Limit","SL":"Stop Loss Limit","SL-M":"Stop Loss Market"},
                    "actions": {"BUY":"Buy","SELL":"Sell"},
                    "intervals": ["1m","3m","5m","10m","15m","30m","1h","D"]
                }))
            }
            "send_telegram_alert" => {
                self.call_api("/telegram", json!({"username": Self::get_str(&a, "username"), "message": Self::get_str(&a, "message")})).await
            }
            "get_holidays" => {
                self.call_api("/holidays", json!({"year": Self::get_i64(&a, "year")})).await
            }
            "get_timings" => {
                self.call_api("/timings", json!({"date": Self::get_str(&a, "date")})).await
            }
            "analyzer_status" => self.call_api("/analyzerstatus", json!({})).await,
            "analyzer_toggle" => {
                self.call_api("/analyzertoggle", json!({"mode": Self::get_bool(&a, "mode")})).await
            }
            _ => Self::err(format!("Unknown tool: {name}")),
        }
    }

    async fn call_api(&self, endpoint: &str, body: Value) -> Result<CallToolResult, ErrorData> {
        match self.client.post(endpoint, body).await {
            Ok(v) => Self::ok(v),
            Err(e) => Self::err(format!("API error: {e}")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  SERVER HANDLER — manual implementation (no macros)
// ═══════════════════════════════════════════════════════════════════════════

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

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async {
            Ok(ListToolsResult {
                tools: Self::tool_definitions(),
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            let name = request.name.as_str();
            let args = match request.arguments {
                Some(map) => serde_json::to_value(map).unwrap_or(json!({})),
                None => json!({}),
            };
            self.dispatch(name, args).await
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
