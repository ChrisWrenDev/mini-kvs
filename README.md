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
| Name                | 🔄 In Progress | description  |

---

### 📦 Project 3: Synchronous client-server networking

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Name               |  🔲 TODO     | description  |

---

### 🧵 Project 4: Concurrency and parallelism

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Name             |  🔲 TODO     | description  |

---

### 📊 Project 5: Asynchronous networking

| Task                                   | Status       | Notes                             |
|----------------------------------------|--------------|-----------------------------------|
| Name               | 🔲 TODO      | description  |

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

