use clap::Parser;
use reqwest::Url;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub url: Url,
    #[arg(short, long)]
    pub trade_amount: u64,
}
