//! This is an algorithm that traverses a complete graph with weighted edge
//! trying to find all cycles where the associated token conversions result
//! in an arbitrage opportunity.
//!
//! The approach is based on Bellman-Ford minimum cost path search.
//! We can apply it because the initial rates we get from the API are normalized
//! in base 10**8 (based on the assumption that the rates have 8 decimals always),
//! and then transformed into negative logharitms so that adding them (for min cost
//! path finding) is equivalent to multiplying the underlying rates for multiple
//! pairs, which is relevant for finding arbitrage opportunities.

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Trades {
    pub arbitrage: Vec<String>,
}

pub struct ArbitrageIteration<'a> {
    pub min_dist: Vec<f64>,
    pub pre: Vec<i32>,
    indices_map: Vec<&'a String>,
}

impl<'a> ArbitrageIteration<'a> {
    pub fn new(tokens_count: usize) -> Self {
        Self {
            min_dist: vec![f64::MAX; tokens_count],
            pre: vec![-1i32; tokens_count],
            indices_map: Vec::new(),
        }
    }

    /// Computed for all starting points.
    pub fn compute_arbitrage_opportunities(
        &mut self,
        log_negated_graph: &'a HashMap<String, HashMap<String, f64>>,
    ) {
        // Setup.
        let n = log_negated_graph.keys().len();
        self.indices_map = log_negated_graph.keys().collect();
        // We sort the indices map because otherwise will return different results for
        // different runs, which are hard to test.
        self.indices_map.sort();

        // We consider the source being the token associated to index 0. This will be used
        // when we map the tokens (string representation) to indices so that we can use
        // vectors instead of hashmaps. This simplifies a bit the mental model of applying
        // the algorithm. Also, which source we pick shouldn't matter for finding the arbitrage
        // opportunities, since we have an undirected graph which is complete (considering the
        // data we get for the pairs);
        self.min_dist[0] = 0f64;

        // The algorithm needs to iterate for n - 1 times so that paths up to n - 1 edges are
        // investigated. This is because the longest path in a graph with n vertices has n - 1 edges.
        for _ in 0..n - 1 {
            for source_curr in 0..n {
                for dest_curr in 0..n {
                    // We want to skip the iteration if source equals with the destination since it
                    // doesn't make sense to convert a token to itself, or if we discover that a pair has
                    // a rate which is unrealistically high, which indicates there is something off with the
                    // data.
                    if source_curr == dest_curr
                        || log_negated_graph[self.indices_map[source_curr]]
                            [self.indices_map[dest_curr]]
                            == f64::MAX
                    {
                        continue;
                    }

                    // We try to relax the distance to current destination through current source and
                    // the pair rate between source and destination.
                    if self.min_dist[dest_curr]
                        > self.min_dist[source_curr]
                            + log_negated_graph[self.indices_map[source_curr]]
                                [self.indices_map[dest_curr]]
                    {
                        self.min_dist[dest_curr] = self.min_dist[source_curr]
                            + log_negated_graph[self.indices_map[source_curr]]
                                [self.indices_map[dest_curr]];
                        self.pre[dest_curr] = source_curr as i32;
                    }
                }
            }
        }
    }

    /// Check for all negative weight cycles, meaning all circular trades
    /// which have the potential of growing the profit continously.
    pub fn trades(
        &self,
        log_negated_graph: &HashMap<String, HashMap<String, f64>>,
        trade_amount: u64,
        graph: &HashMap<String, HashMap<String, u64>>,
    ) -> Trades {
        let n = self.min_dist.len();
        let mut paths = HashSet::new();
        let mut arbitrage_paths = HashSet::new();
        for mut source_curr in 0..n {
            for dest_curr in 0..n {
                // This check confirms this vertices are part of a negative weight cycle.
                if self.min_dist[dest_curr]
                    > self.min_dist[source_curr]
                        + log_negated_graph[self.indices_map[source_curr]]
                            [self.indices_map[dest_curr]]
                {
                    // Construct the cycle in reverse order.
                    let mut print_cycle = vec![dest_curr];
                    while !print_cycle.contains(&(self.pre[source_curr] as usize)) {
                        source_curr = self.pre[source_curr] as usize;
                        print_cycle.push(source_curr);
                    }
                    print_cycle.push(dest_curr);

                    let path = print_cycle
                        .iter()
                        .map(|idx| self.indices_map[*idx].to_owned())
                        .collect::<Vec<String>>()
                        .join(" <--> ");

                    // Given the rates data is a complete graph, we can end up finding the same minimum path
                    // (aka the maximum multiplication of rates) for multiple times, so we want to print it
                    // once.
                    if !paths.contains(&path) {
                        let mut new_amount = trade_amount as f64;
                        for idx in 0..print_cycle.len() - 1 {
                            let first_token = self.indices_map[print_cycle[idx]];
                            let second_token = self.indices_map[print_cycle[idx + 1]];
                            let rate = graph[first_token][second_token];
                            new_amount *= rate as f64 / 100000000f64;
                        }

                        if new_amount > trade_amount as f64 {
                            arbitrage_paths.insert(format!(
                                "Arbitrage opportunity: {}, new trade amount is {:.8}",
                                path, new_amount
                            ));
                        }

                        paths.insert(path);
                    }
                }
            }
        }

        let mut arbitrage: Vec<String> = arbitrage_paths.into_iter().collect();
        arbitrage.sort();
        Trades { arbitrage }
    }
}

#[cfg(test)]
mod tests {
    use super::ArbitrageIteration;
    use crate::pairs::PairMap;
    use maplit::hashmap;

    #[test]
    fn challenge_example() {
        let pair_map = PairMap::from(hashmap! {
            "BTC-BTC".to_owned() => "1.00000000".to_owned(),
            "BTC-BORG".to_owned() => "116352.26544401".to_owned(),
            "BTC-DAI".to_owned() => "23524.13915530".to_owned(),
            "BTC-EUR".to_owned() => "23258.88655838".to_owned(),
            "BORG-BTC".to_owned() => "0.00000868".to_owned(),
            "BORG-BORG".to_owned() => "1.00000000".to_owned(),
            "BORG-DAI".to_owned() => "0.20539905".to_owned(),
            "BORG-EUR".to_owned() => "0.20175399".to_owned(),
            "DAI-BTC".to_owned() => "0.00004290".to_owned(),
            "DAI-BORG".to_owned() => "4.93204333".to_owned(),
            "DAI-DAI".to_owned() => "1.00000000".to_owned(),
            "DAI-EUR".to_owned() => "0.99076521".to_owned(),
            "EUR-BTC".to_owned() => "0.00004355".to_owned(),
            "EUR-BORG".to_owned() => "5.04275777".to_owned(),
            "EUR-DAI".to_owned() => "1.02113789".to_owned(),
            "EUR-EUR".to_owned() => "1.00000000".to_owned()
        });

        let graph = pair_map.to_graph().unwrap();
        let log_negated_graph = graph.log_negate();

        let tokens_count = graph.as_ref().keys().len();
        let mut arb_iter = ArbitrageIteration::new(tokens_count);
        arb_iter.compute_arbitrage_opportunities(&log_negated_graph);
        let trades = arb_iter.trades(&log_negated_graph, 100, graph.as_ref());

        assert_eq!(vec![
            "Arbitrage opportunity: BORG <--> EUR <--> DAI <--> BORG, new trade amount is 101.60928773",
            "Arbitrage opportunity: BTC <--> EUR <--> DAI <--> BTC, new trade amount is 101.88977518",
            "Arbitrage opportunity: DAI <--> EUR <--> DAI, new trade amount is 101.17078960",
            "Arbitrage opportunity: EUR <--> DAI <--> EUR, new trade amount is 101.17078960",
        ], trades.arbitrage);
    }
}
