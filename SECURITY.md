# Security

Nebulax has no public supported release yet. The current program replays embedded synthetic terminal bytes without a PTY, clipboard, GUI or agent-control service. This is not evidence that a future terminal application is secure.

Do not put credentials, real terminal contents or a sensitive proof of concept in a public issue. Use GitHub's private vulnerability reporting for this repository if it is available; its enablement has not been verified. If unavailable, open a minimal request for a confidential contact channel without vulnerability details. The maintainer provides best-effort handling without a response-time guarantee.

Future control policy follows [ADR 0001](docs/adr/0001-approved-direction.md): PTY output cannot mutate configuration or authority; protected actions require application-enforced approval. Owner-only local IPC does not isolate unrestricted same-user malware. Dependencies and generated artifacts require provenance checks before distribution.
