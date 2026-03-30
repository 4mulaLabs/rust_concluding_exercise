# rust_concluding_exercise

Minimal TCP pub/sub broker: publishers send messages on one port; subscribers register topic filters on another. See `src/main.rs` for framing.

## Prerequisites

- [Rust](https://rustup.rs/) (Cargo)
- Python 3 (stdlib only for `scripts/`)

## Run the server

Default ports: publisher **8888**, subscriber **8889**.

```bash
cargo run -- --pub-port 8888 --sub-port 8889
```

Optional logging:

```bash
RUST_LOG=info cargo run -- --pub-port 8888 --sub-port 8889
```

## Simple example (three terminals)

Use the same host/ports the server prints.

**1 — Broker**

```bash
cargo run -- --pub-port 8888 --sub-port 8889
```

**2 — Subscriber** (receive messages for topic `events`)

```bash
python3 scripts/subscriber.py events
```

**3 — Publisher** (send lines interactively; topic `events`, any message text)

```bash
python3 scripts/publisher.py
```

At the prompts, enter `events` as the topic and a message (e.g. `hello`). The subscriber terminal should show something like `received topic='events' message='hello'`.

Try another topic from the publisher (e.g. `logs`); the subscriber above only listens for `events`, so that message should not appear unless you subscribe to `logs` too:

```bash
python3 scripts/subscriber.py events logs
```

## Python clients

| Script | Port | Role |
|--------|------|------|
| `scripts/subscriber.py` | 8889 (override with `--port`) | Subscribe to one or more topics, then print each incoming message. |
| `scripts/publisher.py` | 8888 (optional port as first arg) | Interactive: type topic and message per line until topic is empty. |

**Subscriber**

```bash
python3 scripts/subscriber.py <topic> [more topics ...] [--port 8889]
```

**Publisher**

```bash
python3 scripts/publisher.py          # port 8888
python3 scripts/publisher.py 8888      # explicit port
```

## Editor (VS Code / Cursor)

Run and Debug / **cargo: run** use `RUST_LOG=info`, ports **8888** / **8889**, and the integrated terminal.
