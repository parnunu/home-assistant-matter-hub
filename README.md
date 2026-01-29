# Home Assistant Matter Hub vNext (Rework)

This is a from-scratch Rust rewrite of Home Assistant Matter Hub.

## Status
- Active development happens on the `Rework` branch.
- `main` remains stable and may lag behind.

## Installation (Home Assistant Add-on)
Production deployment is via the Home Assistant add-on:

1. Settings -> Add-ons -> Add-on Store
2. Add this repository:
   - https://github.com/parnunu/home-assistant-addons
3. Install **Home Assistant Matter Hub** and start it

## Local development
Requirements:
- Rust (stable toolchain)
- Node.js 22 only if you still use the legacy frontend (not used for Rework)

Create `.env` at repo root (see `.env.sample`) and set your Home Assistant URL and token.

Run API server:
```
cargo run -p hamh-app
```

API base: `http://localhost:8482/api/matter`

## Design spec
See `docs/Design-Spec-vNext.md`.

## License
Apache-2.0. See `LICENSE`.
