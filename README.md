# PingCAP Talent Plan - Key-Value Store Project

## 📘 Overview

This repository contains my implementation and progress for the Key-Value Store (Project 1) from the [PingCAP Talent Plan](https://github.com/pingcap/talent-plan) course. The course is designed to help developers understand how to build distributed systems by implementing components similar to those found in TiKV, PingCAP’s distributed key-value database.

The KV project focuses on building a simple standalone storage engine that mimics basic functionalities of a key-value store. It’s implemented in Rust and serves as a foundation for more advanced projects in the Talent Plan series.

---

## 🎯 Goals

- Learn Rust through systems-level programming.
- Understand how storage engines work, including the log-structured merge (LSM) concept.
- Implement core components of a KV store:
  - Log persistence
  - Command serialization and deserialization
  - Engine abstraction
  - Error handling
  - Concurrency and testing

---

## 🧱 Project Structure

- `src/`: Contains the source code for the KV engine.
- `tests/`: Contains integration tests to validate correctness.
- `benches/`: Benchmarking utilities to test performance.
- `Cargo.toml`: Project configuration and dependencies.

---

## ✅ Progress Tracker

This project is divided into **five stages**, each introducing new concepts and functionalities. Here's a breakdown of each stage and the specific tasks involved.

---

🔲 TODO
🔄 In Progress 
✅ Done 

### 🧩 Project 1: Basic Storage Engine

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Design `KvStore` struct                | ✅ Done      |                                   |
| Part 1: Make the tests compile         | ✅ Done      |                                   |
| Part 2: Accept command line arguments  | ✅ Done      |                                   |
| Part 3: Cargo environment variables    | ✅ Done      |                                   |
| Part 4: Store values in memory         | ✅ Done      |                                   |
| Part 5: Documentation                  | ✅ Done      |                                   |
| Part 6: Ensure good style with clippy and rustfmt| ✅ Done |                              |

---

### ⚙️ Project 2: Log-structured file I/O

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Part 1: Error handling                 | ✅ Done      | description  |
| Part 2: How the log behaves            | ✅ Done      | description  |
| Part 3: Writing to the log             | ✅ Done      | description  |
| Part 4: Reading from the log           | ✅ Done      | description  |
| Part 5: Storing log pointers in index  | ✅ Done      | description  |
| Part 6: Stateless vs Stateful          | ✅ Done      | description  |
| Part 7: Compacting the log             | ✅ Done      | description  |

---

### 📦 Project 3: Synchronous client-server networking

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Part 1: Command line parsing           |  ✅ Done      | description  |
| Part 2: Logging                        |  ✅ Done      | description  |
| Part 3: Client-server networking setup |  ✅ Done      | description  |
| Part 4: Commands across the network    |  ✅ Done      | description  |
| Part 5: Pluggable storage engines      |  ✅ Done      | description  |
| Part 6: Benchmarking                   |  ✅ Done      | description  |

---

### 🧵 Project 4: Concurrency and parallelism

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Part 1: Multithreading                 |  ✅ Done     | description  |
| Part 2: Creating a shared KvsEngine    |  ✅ Done     | description  |
| Part 3: Add multithreading to KvServer |  ✅ Done     | description  |
| Part 4: Creating a thread pool         |  ✅ Done     | description  |
| Part 5: Abstracted thread pools        |  ✅ Done     | description  |
| Part 6: Evaluating thread pool         |  ✅ Done     | description  |
| Part 7: Evaluating other thread pools and engines |  ✅ Done     | description  |
| Part 8: Lock-free readers              |  ✅ Done     | description  |

---

### 📊 Project 5: Asynchronous networking

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Part 1: Introducing Tokio to the client | 🔲 TODO      | description  |
| Part 2: Convert KvsClient to boxed futures |  🔲 TODO     | description  |
| Part 3: KvsClient with explicit future types |  🔲 TODO     | description  |
| Part 4: KvsClient with anonymous future types |  🔲 TODO     | description  |
| Part 5: Making ThreadPool sharable |  🔲 TODO     | description  |
| Part 6: Converting KvsEngine to futures |  🔲 TODO     | description  |
| Part 7: Driving KvsEngine with tokio |  🔲 TODO     | description  |

---

## 🚀 Getting Started

### Requirements

- Rust (latest stable version)
- Cargo (comes with Rust)

### Build

```bash
cargo build
```
### Test
```bash
cargo test
```
### Run Benchmark (optional)
```bash
cargo bench
```

---

### 📚 Resources

- [PingCAP Talent Plan GitHub](https://github.com/pingcap/talent-plan)
- [TiKV Source Code](https://github.com/tikv/tikv)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Serde](https://serde.rs/)
- [Criterion.rs](https://github.com/bheisler/criterion.rs)

---

### 📝 Notes
This repository is a personal learning exercise. All work is based on the Talent Plan curriculum and expanded with my own understanding and experimentation.

Feel free to fork, explore, and contribute ideas!

