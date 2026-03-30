# rust_concluding_exercise

TCP pub/sub broker (publisher port + subscriber port). Needs Rust; Python scripts are optional for testing.

## How to run

```bash
cargo run -- --pub-port 8888 --sub-port 8889
```

That starts the broker on **8888** (publishers) and **8889** (subscribers). Nothing else is required.

**Optional — try publish/subscribe** (two more terminals):

```bash
python3 scripts/subscriber.py events
python3 scripts/publisher.py
```

In the publisher prompts, use topic `events` and any message; the subscriber prints it.

---

## Prerequisites

- [Rust](https://rustup.rs/) (Cargo)
- Python 3 for `scripts/` (stdlib only)

## Python clients

| Script | Port | Role |
|--------|------|------|
| `scripts/subscriber.py` | 8889 (`--port` to change) | Subscribe to topics, print messages. |
| `scripts/publisher.py` | 8888 (optional port arg) | Interactive topic + message until empty topic. |

```bash
python3 scripts/subscriber.py <topic> [more ...] [--port 8889]
python3 scripts/publisher.py [port]
```

## Editor (VS Code / Cursor)

**cargo: run** / Run and Debug use the same ports and `RUST_LOG=info` where configured.
