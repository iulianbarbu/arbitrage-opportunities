use algorithm::ArbitrageIteration;
use anyhow::Context;
use args::Args;
use clap::Parser;
use pairs::PairReader;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod algorithm;
mod args;
mod pairs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let pair_reader = PairReader::new(args.url);
    let pairs_map = pair_reader
        .fetch_pairs_map()
        .await
        .context("failed fetching pairs map")?;

    let graph = pairs_map.to_graph()?;
    let log_negated_graph = graph.log_negate();
    let tokens_count = graph.as_ref().keys().len();
    let mut arbitrage_iter = ArbitrageIteration::new(tokens_count);
    arbitrage_iter.compute_arbitrage_opportunities(&log_negated_graph);
    let trades = arbitrage_iter.trades(&log_negated_graph, args.trade_amount, graph.as_ref());

    println!("{:#?}", trades.arbitrage);

    Ok(())
}
