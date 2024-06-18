//! Provides means for fetching the pairs and associated weights, and
//! transform them to a graph with edges between tokens, where an edge
//! between the tokens represent a pair for them, and the edge weight
//! represents the pair conversion ratio.

use anyhow::Context;
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest client error: {0}")]
    Client(#[from] reqwest::Error),
    #[error("Deserialize error: {0}")]
    Deser(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// Used to deserialize the challenge API response body.
#[derive(Deserialize, Debug)]
pub struct PairMap {
    rates: HashMap<String, String>,
}

impl PairMap {
    #[cfg(test)]
    pub fn from(rates: HashMap<String, String>) -> Self {
        Self { rates }
    }
}

#[derive(Debug)]
pub struct Graph(HashMap<String, HashMap<String, u64>>);

impl AsRef<HashMap<String, HashMap<String, u64>>> for Graph {
    fn as_ref(&self) -> &HashMap<String, HashMap<String, u64>> {
        &self.0
    }
}

impl AsMut<HashMap<String, HashMap<String, u64>>> for Graph {
    fn as_mut(&mut self) -> &mut HashMap<String, HashMap<String, u64>> {
        &mut self.0
    }
}

impl Graph {
    pub fn log_negate(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut new_graph = HashMap::new();
        for edges in self.as_ref() {
            let mut detailed_edges = HashMap::new();
            for weight in edges.1 {
                detailed_edges.insert(weight.0.to_owned(), -1f64 * (*weight.1 as f64).log2());
            }
            new_graph.insert(edges.0.to_owned(), detailed_edges);
        }

        new_graph
    }
}

impl PairMap {
    /// Transform the map into a graph of relationships between tokens,
    /// where edges represent the pair weight normalized to 10^8 (this is
    /// based on the observation that the API returns all pairs weights with
    /// 8 decimal places).
    pub fn to_graph(&self) -> anyhow::Result<Graph> {
        let mut graph_inner: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for (k, v) in &self.rates {
            let tokens: Vec<&str> = k.split('-').collect();
            let first_token = tokens[0];
            let second_token = tokens[1];
            // let v_as_f64 = v
            //     .parse::<f64>()
            //     .context(format!("decimals part {err_msg}"))?;

            let v_parts: Vec<&str> = v.split('.').collect();
            let v_as_u64 = v_parts[0]
                .parse::<u64>()
                .context("can not convert rate whole part to u64")?
                * 100000000u64
                + v_parts[1]
                    .parse::<u64>()
                    .context("can not convert rate fractional part to u64")?;

            match graph_inner.get_mut(first_token) {
                Some(edges) => {
                    edges.insert(second_token.to_owned(), v_as_u64);
                }
                None => {
                    let mut new_map = HashMap::new();
                    new_map.insert(second_token.to_owned(), v_as_u64);
                    graph_inner.insert(first_token.to_owned(), new_map);
                }
            };
        }

        Ok(Graph(graph_inner))
    }
}

/// Configures a client that calls the challenge API to fetch the token pairs.
pub struct PairReader {
    client: Client,
    pair_api: Url,
}

impl PairReader {
    pub fn new(url: Url) -> Self {
        PairReader {
            client: Client::new(),
            pair_api: url,
        }
    }

    pub async fn fetch_pairs_map(self) -> Result<PairMap> {
        let resp = self.client.get(self.pair_api).send().await?;
        let value: Value = serde_json::from_slice(resp.bytes().await?.as_ref())?;
        tracing::debug!("value= {value}");
        let pairs = serde_json::from_value::<PairMap>(value)?;
        Ok(pairs)
    }
}
