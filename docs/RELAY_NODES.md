# Relay Nodes (Offline Mailboxes)

PAGI-Core supports **store-and-forward relays** for DIDComm-style peer messaging.

Today this is implemented as an **HTTP mailbox** in [`plugins/pagi-didcomm-plugin/src/main.rs`](plugins/pagi-didcomm-plugin/src/main.rs:1):

- Sender attempts direct delivery to the recipient’s `/receive`.
- If the recipient is offline/unreachable, the sender POSTs the same message to a **relay** node’s `/receive`.
- The relay persists it in a mailbox keyed by `to_did`.
- The recipient later fetches messages from the relay via `/poll_relay` (or by calling the relay’s `/inbox` directly).

This gives:

- Offline resilience
- NAT/firewall friendliness (only the relay needs inbound access)
- No SPOF (you can run many relays)

## Running a relay node

Run the DIDComm plugin on an always-on machine/container and enable persistence:

```bash
export DIDCOMM_MAILBOX_DIR=/data/didcomm-mailbox
export DIDCOMM_MAILBOX_MAX_PER_DID=10000
```

The relay is just the same service; persistence makes it survive restarts.

## Sending via relay

Use the ExternalGateway tool `didcomm_send_message_with_relay` (HTTP transport).

The sender will:

1. Try `{to_url}/receive`.
2. On failure, send to `{relay_url}/receive`.

## Polling a relay

Use the ExternalGateway tool `didcomm_poll_relay_inbox` with:

- `did`: your DID
- `relay_url`: relay base URL

This tool calls `{relay_url}/inbox` and returns the relay-cleared batch.

## Notes on IPFS Circuit Relay (Phase 6)

The workspace already includes an embedded IPFS node option with **circuit relay v2** toggles in [`plugins/pagi-ipfs-plugin/src/main.rs`](plugins/pagi-ipfs-plugin/src/main.rs:116):

- `IPFS_RELAY=true` enables relay client behavior.
- `IPFS_RELAY_SERVER=true` turns the node into a relay server.

That layer improves transport reachability. The DIDComm relay/mailbox above provides **asynchronous delivery**.

