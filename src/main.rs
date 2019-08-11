#![feature(async_await, async_closure)]

mod binance;

use facade::dsl::*;
use failure::Error;
use futures::join;

#[runtime::main(runtime_tokio::Tokio)]
pub async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    log::info!("Initialization...");
    let mut control = facade::main()?;

    let btc_panel = Panel(Row(many![Fixed("BTCUSDT:"), Dynamic("BTCUSDT")]));
    let eth_panel = Panel(Row(many![Fixed("ETHUSDT:"), Dynamic("ETHUSDT")]));
    let ltc_panel = Panel(Row(many![Fixed("LTCUSDT:"), Dynamic("LTCUSDT")]));
    let prices_page = Page(
        "Prices",
        "Detailed information about prices",
        Row(vec![btc_panel, eth_panel, ltc_panel]),
    );
    let perf_page = Page(
        "Perf",
        "Detailed performance information",
        Row(vec![
            TitledPanel("App Status", Dynamic("MMSTATE")),
            Panel(List(vec![
                ListItem("BTC/USD", "Description", Dynamic("BTCUSDT")),
                ListItem("ETH/USD", "Description", Dynamic("ETHUSDT")),
                ListItem("LTC/USD", "Description", Dynamic("LTCUSDT")),
            ])),
        ]),
    );
    let scene = Dashboard("LiveCrypto", vec![prices_page, perf_page]);
    //let scene = Spinner();
    control.scene(scene);

    let flow_1 = binance::flow(control.clone());
    let (res_1,) = join!(flow_1);
    res_1
}
