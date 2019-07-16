use bigdecimal::BigDecimal;
use facade::Control;
use failure::Error;
use futures::Stream;
use futures3::compat::{Future01CompatExt, Sink01CompatExt, Stream01CompatExt};
use futures3::StreamExt;
use serde_derive::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;


pub type StreamId = String;
pub type Timestamp = u64;
pub type Symbol = String;
pub type Id = u64;
pub type Price = BigDecimal;
pub type Quantity = BigDecimal;
pub type OrderId = u64;

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamWrapper {
    stream: StreamId,
    data: Envelope,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Envelope {
    #[serde(flatten)]
    event: Event,
    #[serde(rename = "E")]
    time: Timestamp,
    #[serde(rename = "s")]
    symbol: Symbol,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "e", rename_all = "lowercase")]
pub enum Event {
    Trade {
        #[serde(rename = "t")]
        id: Id,
        #[serde(rename = "p")]
        price: Price,
        #[serde(rename = "q")]
        quantity: Quantity,
        #[serde(rename = "b")]
        buyer_order: OrderId,
        #[serde(rename = "a")]
        seller_order: OrderId,
        #[serde(rename = "T")]
        time: Timestamp,
        #[serde(rename = "m")]
        buyer_is_maker: bool,
        #[serde(rename = "M")]
        ignore: bool,
    }
}

pub async fn flow(mut control: Control) -> Result<(), Error> {
    let url = Url::parse("wss://stream.binance.com:9443/stream?streams=btcusdt@trade/ethusdt@trade/ltcusdt@trade")?;
    let (ws_stream, _) = connect_async(url).compat().await?;
    let (sink, stream) = ws_stream.split();
    let (_sink, mut stream) = (sink.sink_compat(), stream.compat());
    loop {
        let item = stream.next().await;
        match item {
            Some(msg) => {
                match msg? {
                    Message::Text(txt) => {
                        let wrapper: StreamWrapper = serde_json::from_str(&txt)?;
                        log::info!("Event: {:?}", wrapper);
                        let envelope = wrapper.data;
                        match envelope.event {
                            Event::Trade { price, .. } => {
                                control.assign(envelope.symbol, price);
                            }
                        }
                    }
                    msg => {
                        log::trace!("Message skipped: {:?}", msg);
                    }
                }
            }
            None => {
                log::error!("Flow ended");
                break;
            }
        }
    }
    Ok(())
}
