use futures_util::StreamExt;
use hyperbook::ftx::{subscribe_channel, FTXChannel, FTXPartial, FTXSubscribed, FTXUpdate};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct LimitOrderBook {
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) =
        tokio_tungstenite::connect_async(url::Url::parse("wss://ftx.us/ws")?).await?;
    let (mut write, read) = ws_stream.split();

    let lob = Arc::new(Mutex::new(LimitOrderBook {
        bids: Default::default(),
        asks: Default::default(),
    }));

    println!("{:?}", lob.lock().unwrap());

    let _ingestion_engine_lob_arc_mutex = lob.clone();
    let _ingestion_engine_handler = tokio::spawn(async move {
        read.for_each(|message| async {
            let data = message.unwrap().into_text().unwrap();
            let json = serde_json::from_str::<serde_json::Value>(&data).unwrap();

            match json.get("type").unwrap().as_str().unwrap() {
                "subscribed" => {
                    let mes: FTXSubscribed = serde_json::from_value(json).unwrap();
                    println!("{:?}", mes);
                }
                "partial" => {
                    let mes: FTXPartial = serde_json::from_value(json).unwrap();
                    let mut bids = mes.data.bids;
                    bids.reverse();
                    let asks = mes.data.asks;

                    for bid in bids {
                        if bid.size == Decimal::from_f64(0.0).unwrap() {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .bids
                                .remove(&bid.price);
                        } else {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .bids
                                .insert(bid.price, bid.size);
                        }
                    }
                    for ask in asks {
                        if ask.size == Decimal::from_f64(0.0).unwrap() {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .asks
                                .remove(&ask.price);
                        } else {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .asks
                                .insert(ask.price, ask.size);
                        }
                    }
                }
                "update" => {
                    let mes: FTXUpdate = serde_json::from_value(json).unwrap();
                    let mut bids = mes.data.bids;
                    bids.reverse();
                    let asks = mes.data.asks;

                    for bid in bids {
                        if bid.size == Decimal::from_f64(0.0).unwrap() {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .bids
                                .remove(&bid.price);
                        } else {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .bids
                                .insert(bid.price, bid.size);
                        }
                    }
                    for ask in asks {
                        if ask.size == Decimal::from_f64(0.0).unwrap() {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .asks
                                .remove(&ask.price);
                        } else {
                            _ingestion_engine_lob_arc_mutex
                                .lock()
                                .unwrap()
                                .asks
                                .insert(ask.price, ask.size);
                        }
                    }
                }
                _ => panic!(
                    "Unknown message type. {}",
                    json.get("type").unwrap().as_str().unwrap()
                ),
            }
        })
        .await;
    });

    subscribe_channel(&mut write, FTXChannel::Orderbook, "BTC", "USD").await;

    loop {
        println!("{:#?}", lob.lock().unwrap());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
