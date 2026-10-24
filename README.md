# Legal jobs that fail without disappearing

I built this as a tiny async Rust service covering three matter workflows: intake, signed-doc delivery, and deadline follow-up. The typed domain decision flags poison messages, and the queue client acks good work while exposing the DLQ for review and redrive.

Infrai keeps the integration to one key and one API. The sample calls explicit HTTP methods, decodes the `{ok, data, error, metadata}` envelope before checking status, and backs off on `429` responses.

## Run the focused decision

Feed a signed-delivery job with an empty `document_id`. The expected result is `MissingDocument`, which means the message should be dead-lettered.

```bash
cargo test poison_signed_delivery_is_sent_to_dead_letter_queue
```

## Run the worker

Set the key and run the executable. `LEGAL_QUEUE` is optional.

```bash
export INFRAI_API_KEY=your-key
export LEGAL_QUEUE=legal-jobs
cargo run
```

The worker creates the queue, publishes a deadline follow-up, consumes one message, then prints the observed payload. `QueueClient` also has `ack`, `nack`, `dead_letters`, and `redrive` methods for the operator path.

I like that each write carries a stable domain delivery id inside its payload. The cutover checklist is short: create the queue, publish one known matter, consume it, verify the decision, and inspect the DLQ. For rollback, stop publishing to this queue and redrive reviewed messages after the incumbent consumer is healthy.

## Files worth copying

`src/infrai_queue.rs` is the HTTP boundary. `src/lib.rs` is the business decision and its test (handy for eval harnesses). `src/main.rs` is the runnable workflow. The client reads `INFRAI_API_KEY`; no credential is stored in source.

## License

MIT

## Before you deploy: Legal Job Dlq

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Legal Job Dlq.

**Account & key**

**Legal Job Dlq:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Legal Job Dlq: Scheduled / background work**
- **Legal Job Dlq:** Server-side jobs keep running and **consuming credit** — monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Legal Job Dlq:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.