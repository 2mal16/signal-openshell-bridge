# Learning Journey: A Signal-to-OpenShell Bridge with Rust and Tokio

**Project:** Build a secure, asynchronous bridge between a dedicated `signal-cli` account in a Proxmox LXC and an agent controller running OpenShell. The agent controller uses NVIDIA PAIR (with your custom llama-swap support) to reach local inference backends.

**Teaching philosophy:** Learn by implementing progressively larger slices. Each module includes a concept, a small practical task, tests, questions to investigate, and a concrete completion criterion. Resist pasting a complete finished bot: write the code, observe failures, then improve it.

**Starting point (already working):** Dedicated Signal phone number; `signal-cli` 0.14.9 installed under service user `signalcli`; daemon bound to `127.0.0.1:8080`; `GET /api/v1/check` responds. The existing agent and inference architecture remains unchanged.

## 0. Target architecture and boundaries

```text
Your Signal app
      |
Signal network (E2EE ends at signal-cli)
      |
Signal LXC
  signal-cli daemon (localhost:8080)
      |   GET /api/v1/events (SSE)
      |   POST /api/v1/rpc (JSON-RPC)
  Rust/Tokio signal-bridge
      |   sender authorization, durable inbox/outbox,
      |   conversation mapping, bounded job queue
      |
      | HTTPS + service authentication on private network
      v
Trusted agent job controller (not inside sandbox)
      |
      v
OpenShell sandbox(s): agent code + approved tools
      |
      v
NVIDIA PAIR -> llama-swap -> llama.cpp / vLLM -> GPUs
```

**Core security rule:** The OpenShell agent never receives Signal's registration keys and cannot call the signal-cli control API. The Signal bridge is the only service that sends Signal replies. The agent controller submits jobs and returns results; it is not a Signal account holder.

**Initial scope:** One allowlisted sender, one-to-one text messages, no images or groups, one active agent session, one reply per job. Add features only after tests.

**Terminology:**
- **SSE:** Server-Sent Events, an HTTP stream of events.
- **JSON-RPC:** Request/response protocol used for signal-cli operations.
- **Tokio task:** A cooperatively scheduled asynchronous task, not necessarily an OS thread.
- **Backpressure:** Preventing producers from overwhelming consumers and memory.
- **At-least-once delivery:** An event may arrive multiple times; consumers must deduplicate.
- **Idempotency:** Repeating an operation does not create a second logical result.

## How to study

For every module:
1. Predict how the program should behave before executing it.
2. Implement the smallest possible working slice.
3. Write at least one automated test and one manual failure experiment.
4. Document one confusing compiler error or concurrency behavior in `notes/`.
5. Make a git commit when the completion criterion passes.

Suggested pace: **12 modules over roughly 3–6 weeks** at a few focused sessions per week. This is a flexible sequence, not a deadline. Work on your development machine and deploy release binaries to the Signal LXC. Keep real Signal credentials out of git and tests.

---

## Module 1 — Rust project fundamentals

**Learn:** Cargo workspaces and crates, `let` vs `let mut`, ownership, `String` vs `&str`, borrowing, `Option<T>`, `Result<T, E>`, `match`, `?`, error propagation.

**Build:** A command-line app that reads a fake message from stdin and prints `Agent received: ...`, rejecting empty messages.

**Exercises:**
- Implement `fn normalize_message(input: &str) -> Option<String>`.
- Experiment with moving a `String` into a function versus borrowing it.
- Replace `.unwrap()` in ordinary application code with meaningful error handling.
- Add unit tests for empty, whitespace-only, Unicode and long messages.

**Definition of done:** You can explain who owns each `String` and the difference between returning `Option` and `Result`.

## Module 2 — Typed data models and serde

**Learn:** `struct`, `enum`, `#[derive]`, `serde::{Serialize, Deserialize}`, optional JSON fields, unknown fields, newtypes, `serde_json::Value` versus strongly typed data.

**Build:** Parse saved, redacted Signal `receive` notifications from JSON files. Extract source identifier, timestamp, text and message type.

**Representative upstream shape (simplified):**

```json
{
  "jsonrpc": "2.0",
  "method": "receive",
  "params": {
    "envelope": {
      "sourceNumber": "+41000000000",
      "sourceUuid": "redacted-uuid",
      "sourceDevice": 1,
      "timestamp": 1710000000000,
      "dataMessage": {"message": "Hello"}
    },
    "account": "+41999999999"
  }
}
```

**Exercises:**
- Handle missing `sourceNumber` without panicking (number privacy may apply).
- Ignore receipts, typing events and `syncMessage` objects.
- Add a deterministic event key using account, sender identity, source device and message timestamp; consider collision and replay limitations.
- Design a `ParsedEvent` enum such as `IncomingText`, `Ignored`, `Malformed`.

**Definition of done:** Fixtures cover supported and unsupported event variants; malformed data never crashes the process.

## Module 3 — Async foundations and the Tokio runtime

**Learn:** `async fn`, `Future`, `.await`, cooperative scheduling, `#[tokio::main]`, `tokio::spawn`, `JoinHandle`, `Send + 'static`, why blocking work harms an async runtime.

**Build:** Simulate ten incoming messages, with concurrent fake agent responses that take different durations.

**Exercises:**
- Compare sequential awaits with `tokio::spawn`.
- Limit concurrency with `tokio::sync::Semaphore`.
- Test what happens when you call blocking `std::thread::sleep` inside an async task; fix it using `tokio::time::sleep`.
- Investigate whether results preserve arrival order.

**Definition of done:** You can explain how many tasks and how many OS threads your program uses, and why their counts differ.

## Module 4 — Tokio channels and backpressure

**Learn:** `tokio::sync::mpsc`, `oneshot`, bounded channels, task ownership, fan-in, queues, producer/consumer lifecycle.

**Build:** Three-stage in-memory pipeline:

```text
Fake Signal producer -> bounded job channel -> worker -> response channel -> fake sender
```

**Exercises:**
- Configure queue capacity 5, then push 100 jobs; observe producer blocking/backpressure.
- Model `AgentJob` and `AgentResult` types.
- Use `oneshot` to return a result from a worker to its submitter.
- Simulate a dead worker and handle a closed channel.

**Definition of done:** Workload stays bounded; an unavailable worker causes an explicit error rather than unbounded memory growth.

## Module 5 — HTTP clients and JSON-RPC with reqwest

**Learn:** `reqwest::Client`, JSON request bodies, response deserialization, HTTP versus JSON-RPC errors, request IDs, timeouts, connection reuse.

**Build:** A `SignalClient` with `check()` and `send_message()` methods. Keep it pointed at loopback.

**JSON-RPC send example:**

```json
{
  "jsonrpc": "2.0",
  "method": "send",
  "params": {"recipient": ["+41000000000"], "message": "Test"},
  "id": "request-42"
}
```

In multi-account daemon mode, also supply `params.account` with the bot account identifier. Validate **both** the HTTP status and a JSON-RPC `error` field; HTTP 200 alone does not prove success.

**Exercises:**
- Use a unique request ID per operation.
- Test with a local fake HTTP server, not your actual Signal account.
- Distinguish temporary network failures from invalid RPC parameters.
- Keep messages and phone numbers out of routine logs.

**Definition of done:** Health and send operations are tested against a mock; production sends work only after an explicit manual test.

## Module 6 — Consume the Signal SSE stream

**Learn:** Streaming HTTP responses, `StreamExt`, SSE framing (`id:`, `data:`, blank lines), keepalive comments, reconnect logic, `Last-Event-ID`, heartbeats and stream termination.

**Build:** A read-only watcher that prints *redacted metadata* from `/api/v1/events` without sending replies.

**Exercises:**
- Use an SSE crate (such as `reqwest-sse`) or implement a small parser as a learning exercise.
- Handle multiple `data:` lines and blank-line event boundaries correctly.
- Add reconnect with capped exponential backoff and jitter.
- Remember the last processed SSE ID; send `Last-Event-ID` on reconnect.
- Simulate a server restarting halfway through a message.

**Important limitation:** signal-cli retains only the last 1,000 SSE events in memory, and replay availability changes when the daemon restarts. SSE IDs help reconnection but are **not** a substitute for a durable local inbox.

**Definition of done:** Disconnecting and reconnecting does not crash the watcher, and it parses mocked multiline SSE events.

## Module 7 — First complete echo bot

**Learn:** Composing async stages, dependency injection, testable business logic, distinguishing transport data from domain events.

**Build:** Authorize one personal Signal sender and reply with `Agent received: <message>`.

**Exercises:**
- Reject group messages, self-sent sync messages, attachments-only messages and unknown senders.
- Make the authorizer a pure function with unit tests.
- Reuse one HTTP client rather than constructing one per message.
- Add a maximum incoming text length and a bounded outbound message length.
- Test from your own Signal app to the dedicated bot number.

**Definition of done:** Only an authorized one-to-one text message triggers exactly one visible reply under normal operation.

### Milestone A: working Signal loop

At this point you can deploy a useful Rust service **without any agent or OpenShell connection**.

---

## Module 8 — Durable state and delivery guarantees

**Learn:** SQLite transactions, unique constraints, persistence boundaries, idempotency keys, state machines, the limits of exactly-once delivery.

**Build:** Durable inbox/job/outbox tables; start with one writer and a small database library (e.g. `sqlx` SQLite).

Suggested state model:

```text
RECEIVED -> AUTHORIZED -> QUEUED -> RUNNING -> RESPONSE_READY -> SENT
                                      |              |
                                    FAILED ---------+
```

**Exercises:**
- Insert each event using a unique deduplication key in a transaction.
- Persist the incoming event *before* acknowledging its processing in your own pipeline.
- Persist the planned response before sending it.
- Deliberately crash after a successful Signal send but before marking `SENT`.
- Explain why a retry could send a duplicate; choose a documented recovery policy.
- Retain bounded metadata and avoid storing plaintext messages longer than necessary.

**Definition of done:** Restarting the bridge preserves accepted jobs and avoids most duplicates; you understand that ambiguous send results prevent a blanket exactly-once guarantee.

## Module 9 — Cancellation, timeouts and graceful shutdown

**Learn:** `tokio::select!`, cancellation safety, `CancellationToken`, `JoinSet`, `Semaphore`, `SIGTERM`, shutdown ordering.

**Build:** Stop accepting new events, finish or checkpoint active jobs, flush durable state, and exit cleanly.

**Exercises:**
- Race shutdown signals against a network request with `select!`.
- Document which operations are safe to cancel and which must first be committed to the database.
- Simulate a permanently hung agent call and enforce a timeout.
- Make `systemctl stop` cleanly terminate the bridge within a bounded period.
- Test repeated restarts during a response send.

**Definition of done:** No accepted job is silently discarded on ordinary shutdown, and retry behavior is explicit.

## Module 10 — Security and service operation

**Learn:** Least privilege, trusted boundaries, Linux service users, secrets, network ACLs, TLS, structured logging, health endpoints, deployment.

**Build:** A hardened systemd service for the bridge, under `signalcli` or preferably a separate `signalbridge` user with narrowly scoped access to the loopback RPC API.

**Exercises:**
- Keep signal-cli on `127.0.0.1:8080`, inaccessible from your LAN.
- Prefer an allowlisted Signal ACI/UUID when available, with careful enrollment/validation rather than blindly trusting display names.
- Do not log Signal message bodies, phone numbers or secrets by default.
- Add health diagnostics that do not disclose messages.
- Configure rate limits, message-size limits and a bounded job queue.
- Make the internal agent-controller connection authenticated, encrypted and restricted by firewall rules.
- Use `tracing` and `tracing-subscriber` with redaction and correlation IDs.

**Definition of done:** A compromised agent sandbox cannot read Signal keys, call the Signal RPC endpoint, or send Signal messages directly.

### Milestone B: production-shaped messaging bridge

You now have a durable asynchronous Signal service and can replace the echo handler with a remote-agent adapter.

---

## Module 11 — OpenShell job controller integration

**Learn:** HTTP service contracts, job creation, status polling or callbacks, task correlation, retries, access control, separation of trusted controller and untrusted sandbox.

**Build:** Define your own *authenticated* job API for the trusted controller, not a purported built-in OpenShell endpoint:

```http
POST /v1/jobs
Authorization: Bearer <service-token>
Content-Type: application/json
```

```json
{
  "job_id": "generated-idempotency-key",
  "conversation_id": "opaque-conversation-id",
  "text": "Summarize these notes"
}
```

Response on acceptance:

```json
{"job_id":"generated-idempotency-key","status":"queued"}
```

A separate authenticated `GET /v1/jobs/{job_id}` can return a final answer. The trusted controller translates requests into supported OpenShell sandbox-management calls. The agent itself never acquires permission to administer the Signal LXC.

**Exercises:**
- First use a fake agent server with predictable responses.
- Add per-conversation ordering and prevent two concurrent jobs from racing against the same conversation state.
- Handle agent failure, long context generation, timeouts and cancellation.
- Add correlation IDs from Signal event through agent job to outgoing reply.
- Write an integration test with a mock controller; use the real OpenShell runtime only after this passes.

**Definition of done:** Sending a Signal message creates an authenticated agent job and eventually returns its result through the trusted bridge.

## Module 12 — PAIR, llama-swap and the final reliability lab

**Learn:** Layered responsibility, operational boundaries, capacity limits, observability, chaos testing.

**Build:** Run the agent inside OpenShell with access only to the PAIR inference API. PAIR routes to llama-swap, which owns model lifecycle; llama.cpp or vLLM runs inference. Keep the Signal bridge unaware of GPU/model internals.

**Failure experiments:**
- Kill the Signal daemon during a live SSE stream.
- Restart the bridge while jobs are queued.
- Restart the trusted agent controller during a job.
- Make PAIR temporarily unavailable or force llama-swap to unload a model.
- Simulate slow inference with multiple Signal messages from the same conversation.
- Attempt a call from an OpenShell sandbox directly to signal-cli's RPC address: it must be denied.
- Flood the bridge with messages from an unauthorized sender: no jobs should be created.

**Definition of done:** An authorized Signal request reaches a sandboxed agent, uses PAIR/llama-swap inference, and receives a reply; common failures are observable and recoverable without uncontrolled duplicates or privilege escalation.

---

## Proposed Rust dependencies (add them only when needed)

| Crate | Why | Introduce |
|---|---|---|
| `serde`, `serde_json` | Strongly typed JSON | Module 2 |
| `tokio` | Async runtime, tasks, channels, signals | Module 3 |
| `reqwest` | HTTP/JSON-RPC client | Module 5 |
| `reqwest-sse` or another maintained SSE parser | SSE streaming | Module 6 |
| `futures-util` | Stream combinators, if required | Module 6 |
| `uuid` | Correlation/request identifiers | Module 5–7 |
| `sqlx` with SQLite | Durable inbox/outbox and migrations | Module 8 |
| `tokio-util` | Cancellation token | Module 9 |
| `tracing`, `tracing-subscriber` | Structured observability | Module 10 |
| `thiserror` or `anyhow` | Domain errors / application errors | Module 1–5 |

Prefer understanding the role of each crate over adding everything on day one. Pin tested dependency versions using `Cargo.lock` in the deployed application.

## Suggested repository layout

```text
signal-agent-bridge/
  Cargo.toml
  Cargo.lock
  src/
    main.rs              # runtime wiring and shutdown
    config.rs            # validated environment configuration
    signal/
      mod.rs
      client.rs          # JSON-RPC HTTP client
      events.rs          # typed Signal events
      sse.rs             # SSE connection/reconnect
    domain/
      mod.rs
      authorization.rs   # pure permission checks
      message.rs         # normalized messages and jobs
    queue.rs             # bounded Tokio channels
    persistence.rs       # durable inbox/outbox
    agent.rs             # trusted controller client
  tests/
    fixtures/
      receive_text.json
      receive_receipt.json
      receive_group.json
    integration.rs
  migrations/
  deploy/
    signal-bridge.service
    signal-bridge.env.example
  notes/
    rust-learnings.md
    failure-experiments.md
  README.md
```

Start with only `main.rs`; refactor into this structure when modules warrant it. Premature abstractions can obscure the lessons.

## Practical checklist

- [ ] 1. Parse and validate CLI input with Rust
- [ ] 2. Deserialize realistic Signal JSON fixtures
- [ ] 3. Understand tasks and `await`
- [ ] 4. Implement bounded Tokio channels
- [ ] 5. Create a typed JSON-RPC client
- [ ] 6. Consume and reconnect to SSE
- [ ] 7. Reply to an authorized Signal message
- [ ] 8. Persist job and response state with SQLite
- [ ] 9. Handle cancellation and shutdown safely
- [ ] 10. Run as a secure systemd service
- [ ] 11. Submit jobs to an authenticated OpenShell controller
- [ ] 12. Complete the PAIR/llama-swap reliability lab

## Learning questions to answer before considering the project complete

1. Why can an async Rust function be cancelled by dropping its future, and why does that matter for message delivery?
2. What does `tokio::spawn` require from captured values, and why?
3. When does an `mpsc` channel provide useful backpressure?
4. Why is `HTTP 200` insufficient evidence that a JSON-RPC call succeeded?
5. Why can SSE replay not guarantee durable, exactly-once processing?
6. What happens if sending a reply succeeds but recording its success in SQLite fails?
7. Why must Signal credentials and outbound Signal messaging remain outside an untrusted OpenShell sandbox?
8. Why should PAIR handle model routing while the bridge handles conversation identity and delivery state?

## Official reference material

- [Rust Book](https://doc.rust-lang.org/book/) — language, ownership, traits and error handling
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — small runnable examples
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) — tasks, channels, shared state, `select!`
- [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) — cancellation and service shutdown
- [signal-cli repository](https://github.com/AsamK/signal-cli) — project and release information
- [signal-cli JSON-RPC manual](https://github.com/AsamK/signal-cli/blob/master/man/signal-cli-jsonrpc.5.adoc) — exact daemon endpoints and event envelopes
- [reqwest documentation](https://docs.rs/reqwest/) — HTTP client
- [reqwest-sse documentation](https://docs.rs/reqwest-sse/) — one SSE streaming option
- [OpenShell documentation](https://docs.nvidia.com/openshell/) — sandbox runtime and security configuration

**Next action:** Complete Module 1 locally, then use your Signal LXC only for the first live JSON-RPC and SSE exercises. The agent controller and OpenShell integration come *after* the reliable echo bot.
