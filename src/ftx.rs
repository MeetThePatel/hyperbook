use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use rust_decimal::Decimal;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub enum FTXOperation {
    Subscribe,
    Unsubscribe,
}
impl ToString for FTXOperation {
    fn to_string(&self) -> String {
        match self {
            FTXOperation::Subscribe => "subscribe".to_owned(),
            FTXOperation::Unsubscribe => "unsubscribe".to_owned(),
        }
    }
}

pub enum FTXChannel {
    Orderbook,
    Trades,
    Ticker,
}
impl ToString for FTXChannel {
    fn to_string(&self) -> String {
        match self {
            FTXChannel::Orderbook => "orderbook".to_owned(),
            FTXChannel::Ticker => "ticker".to_owned(),
            FTXChannel::Trades => "trades".to_owned(),
        }
    }
}

pub enum FTXMessage {
    FTXPing,
    FTXRequest {
        operation: FTXOperation,
        channel: FTXChannel,
        market: String,
    },
}
impl ToString for FTXMessage {
    fn to_string(&self) -> String {
        match self {
            FTXMessage::FTXPing => r#"{"op":"ping"}"#.to_owned(),
            FTXMessage::FTXRequest {
                ref operation,
                ref channel,
                ref market,
            } => {
                format!(
                    r#"{{"op":"{operation}","channel":"{channel}","market":"{market}"}}"#,
                    operation = operation.to_string(),
                    channel = channel.to_string(),
                    market = market
                )
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LOBLevel {
    pub price: Decimal,
    pub size: Decimal,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LOBSnapshot {
    action: String,
    pub bids: Vec<LOBLevel>,
    pub asks: Vec<LOBLevel>,
    checksum: u32,
    // #[serde(with = "ts_nanoseconds")]
    time: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FTXPartial {
    channel: String,
    pub data: LOBSnapshot,
    market: String,
    r#type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FTXUpdate {
    channel: String,
    pub data: LOBSnapshot,
    market: String,
    r#type: String,
}

pub async fn subscribe_channel(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    channel: FTXChannel,
    base: &str,
    quote: &str,
) {
    write
        .send(Message::Text(
            FTXMessage::FTXRequest {
                channel,
                market: base.to_owned() + "/" + quote,
                operation: FTXOperation::Subscribe,
            }
            .to_string(),
        ))
        .await
        .unwrap();
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FTXSubscribed {
    channel: String,
    market: String,
    r#type: String,
}
