# MATCH CORE

This project implements a **centralized off-chain matching engine**, the fundamental component at the core of any trading platform or electronic exchange. A matching engine is responsible for receiving buy and sell orders from multiple clients, storing them in an order book, and continuously matching compatible orders to generate trades. In practice, this is the same mechanism that powers real-world exchanges such as Binance or Coinbase: every time a trader places a limit or market order, the matching engine determines if there is a counterpart order available and executes the transaction.

Our implementation is written in **Rust**, leveraging its safety guarantees and concurrency model to deliver **low latency, high throughput, and fault-tolerant order processing**. It is designed as a self-contained service that can run locally on your machine, providing developers and students with a realistic environment to study and experiment with how exchanges operate under the hood.

## Key Features

- **Order Handling:** supports limit and market orders.
- **Order Book:** efficient in-memory data structures for fast lookups and matching.
- **Matching Logic:** automatically executes trades when bid/ask prices overlap.
- **Persistence:** snapshots of the order book can be stored and restored from disk.
- **Resilience:** recovery after restarts or crashes.
- **Real-Time-Updates:** clients receive live notifications of book updates and trades via WebSocket.
- **Concurrency:** built with Rust’s async runtime to handle multiple clients safely and efficiently.
- **Metrics:** track trades, active orders, and basic performance stats.

## Architecture Overview

This matching engine is built with a strong focus on efficiency, parallelism, and safety. To achieve these goals, each engine instance is designed to handle a `single trading pair independently`. This isolation minimizes contention between markets and allows the system to scale horizontally.
If bidirectional or cross-pair trading is required (for example, `USD/EUR` and `EUR/USD`), separate instances can be deployed — for instance, as two distinct Docker containers — each dedicated to one direction of the market.

![Architecture Overview](./assets/images/architecture-overview.png)

To maximize parallelism and minimize thread blocking and bottlenecks, the engine follows a layered threads design. Each layer handles a specific stage of order processing, with minimal synchronization and communication between threads. This approach ensures that threads can operate concurrently with minimal interruptions, improving throughput and overall system efficiency.

- **Server Layer:** server for client connections. It does include a minimal API rest to manage orders and a socket channel to be updated with the order book changes.
- **Matching Engine Core:** the heart of the system, where the order book exists and the trades are created through matching orders.
- **Event System:** two **mspc** channels connect the execution threads and keep the paralellism efficient. The <span style="color:#f57751;">red channel</span> allows the connection to the engine while the <span style="color:#5951f5;">blue channel</span> helps the enginee to broadcast the changes.
The mpsc channels will act as FIFO queues. This way, when the matching engine is busy and cannot process an incoming order, that order will wait in the queue until it is executed.
- **Persistency Layer:** stores the orders and trades. It also in charge to create snapshots of the orderbook for recovery.

### Data Structure

The matching engine uses a **hybrid structure** that combines hash maps and binary heaps to efficiently manage and match orders.

````
pub type OrderQueue = LinkedHashmap<Order>;
pub type OrderMap = HashMap<Price, OrderQueue>;
pub type BidQueue = BTreeSet<Price>; // smallest first
pub type AskQueue = BTreeSet<Reverse<Price>>; // greatest first

pub struct OrderBook {
    pub bids: OrderMap,
    pub asks: OrderMap,
    pub bids_queue: BidQueue,
    pub asks_queue: AskQueue,
    pub last_price: Price
}
````

- `OrderQueue`:
Stores Order in a `LinkedHashmap` to ensure FIFO access from the linked list.
- `OrderMap` (`HashMap<Price, OrderQueue>`)
Each price level maps to a queue of orders (`OrderQueue`), stored as a stack (`VecDequeue<Order>`) sorted in a FIFO model (older orders have higher priority).
    - `bids`:
    contains buy orders grouped by price.
    - `asks`:
    contains sell orders grouped by price.
- `BidQueue` and `AskQueue`:
These heaps maintain the set of active price levels, enabling quick access to the best bid (highest price) and best ask (lowest price) in O(1).
- `BidQueue`:
Prices for placed bids are uniquely stored in a descending order in a `BTreeSet<Price>` structure.
- `AskQueue`:
Prices for placed asks are uniquely stored in an ascending order in a `BTreeSet<Reverse<Price>>` structure.
- `last_price`:
Stores the last traded price for market reference.


This structure optimizes for fast price-level access and priority matching:
Using HashMap allows constant-time lookup of existing price levels.
BinaryHeap ensures O(log n) insertion/removal while maintaining order priority (by price or time). Keeping separate global heaps for prices (BidQueue, AskQueue) avoids scanning all price levels to find the best price — crucial for real-time matching performance.

This design balances speed, simplicity, and memory efficiency, and scales well under high-frequency trading workloads.

### Matching flow

The following diagram shows the flow to match an entry order:

![Matching Overview](./assets/images/matching-overview.png)

#### Complexity

| Operation | Description | Complexity |
| --------- | ----------- | ---------- |
| **Add order** | Insert order into price-level map + update prices queue | O(log n) |
| **Find best bid/ask** | `.first()` of `BidQueue` or `AskQueue` | O(1) |
| **Find specific price level** | HashMap lookup `bids` or `asks` | O(1) |
| **Remove best bid/ask** | `pop_first()` of `BidQueue` or `AskQueue` | O(log n) |
| **Remove price level** | Remove queue from map `bids` or `asks` | O(1) |
| **Match order (partial/full)** | Pop from tree(s), update orders in maps and prices tree if needed | O(log n) |


## Development 

### Dependencies

- **axum:** Web framework for building HTTP servers and WebSocket endpoints; handles routing, extractors, and middleware.
- **tokio:** Asynchronous runtime for handling multiple concurrent tasks efficiently, including WebSocket connections.
- **serde:** Serialization/deserialization library; allows converting Rust structs to JSON and back.
- **serde_json:** JSON support for serde, enabling sending/receiving JSON messages over WebSocket.
- **dashmap:** Thread-safe concurrent hashmap, useful for managing connected clients without locking the entire structure.
- **tracing:** Structured logging library to track events, useful for debugging and monitoring.
- **tracing-subscriber:** Subscriber implementation for tracing to collect and format logs.
- **anyhow:** Simple error handling library for Rust; allows returning and propagating errors easily.
- **bincode:** Efficient binary serialization library, useful for saving/loading snapshots of the order book quickly.

### Commands

#### Install dependencies

````
cargo install cargo-watch
cargo install --path .
````

#### Start development mode

````
cargo watch -x run
````

#### Start testing mode

````
cargo watch -x test
````

