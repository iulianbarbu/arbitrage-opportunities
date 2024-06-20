# Arbitrage opportunity search

The data we use here consists from exchange rates for pairs of tokens/currencies. These pairs represent relationships between
the tokens and can be used as edges in a graph. Each pair rate represents a weight associated to a corresponding edge.

The example data we use comes from SwissBorg API (`https://api.swissborg.io/v1/challenge/rates`), and we assume it always returns
the rates with 8 decimal precision, so each rate is represented in the graph as weights of 10**8 denomination.

## The algorithm

The applied algorithm is Bellman-Ford who works well for determining minimum weighted paths in directed/undirected graphs
with negative weights, and can also spot negative cycles, which is exactly what we're interested in. To be able to apply
the algorithm on our graph of pairs rates we'll need to transform the raw data in the first place by applying
 `function(weight) = -1 * log2(weight)` on each edge weight. This is done because Bellman-Ford finds the minimum
weighted path by adding the weights associated to the path edges, but in our case we're interested in multiplying the
rates so that we simulate trade sequences that result in an arbitrage opportunity. The function previously mentioned
realizes the multiplication by the virtue of the logarithms additions, which in practice are equivalent to powers of
two multiplications, so the smaller the negative logarithm sums, the bigger the rates multiplications, and the bigger
the chance of observing an arbitrage opportunity.

**Time complexity** is [(V - 1) * V] * V, where V is the number of vertices. The theoretical time complexity of the algorithm is
O(V * E) where V is the number of vertices and E the number of edges, but since the pairs rates form a complete multidigraph
that includes self-vertex loops too (which are ignored in the algorithm), we get E = (V - 1) * V.

**Space complexity** is O(V) coming from the various structures we use to hold the minimum distance to each vertex, th


## A note about SwissBorg

SwissBorg is a crypto wealth management platform that utilizes blockchain technology to provide a secure and transparent place
for buying, selling, and managing digital assets. A few of the outstanding features are noted bellow:
- earn yield on their holdings through Smart Earn feature (for a limited set of tokens)
- use SwissBorg Earn feature to do the heavy lifting for staking, lending, and yield farming
- invest in crypto ETF like bundles through Thematics
- invest in BORG (SwissBorg governance token) to upgrade the plan, where each plan tier provides a set of benefits like smaller
  fees, higher yields and voting rights in governing within SwissBorg ecosystem.
- find the good liquidity and rates with SwissBorg Exchange Smart Engine (aka Meta-Exchange), which analyzes pairs on multiple
  CEXs, DEXs and foreign exchanges, finding the best route to execute customers orders in milliseconds.
- use SwissBorg Exchange AI Portfolio Analytics to gain better insights into their portfolios (total fees spent, overall performance,
  personal ROI for unrealized/realized gains) and the tokens they are interested in, through features like Cyborg Predictor, SwissBorg
  Indicator, Community Sentiment, support and resistance).
