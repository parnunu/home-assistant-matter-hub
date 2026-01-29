# Home Assistant Matter Hub vNext (Rework)

![Home-Assistant-Matter-Hub](./docs/assets/hamh-logo-small.png)

## What this is
Home Assistant Matter Hub (HAMH) is a Matter bridge for Home Assistant. It exposes selected Home Assistant entities to
Matter controllers like Apple Home, Google Home, and Alexa using local networking (no cloud, no port forwarding).

## Project status
- Active development is on the `Rework` branch.
- `main` is considered stable and may lag behind `Rework`.

## Installation (Home Assistant Add-on)
Production deployment is the Home Assistant add-on.

1. In Home Assistant: Settings -> Add-ons -> Add-on Store.
2. Add this add-ons repository URL:
   - https://github.com/parnunu/home-assistant-addons
3. Refresh the Add-on Store and install **Home Assistant Matter Hub**.
4. Configure as needed and click Start.

## Local development (PC)
Requirements:
- Node.js 22
- pnpm 10.28.1

Create a `.env` at repo root (see `.env.sample`) and set your Home Assistant URL and token.

Run backend:
```
pnpm --filter @home-assistant-matter-hub/backend run serve
```

Run frontend:
```
pnpm --filter @home-assistant-matter-hub/frontend run dev
```

Run both:
```
pnpm run dev:pc
```

Backend API: http://localhost:8482
Frontend UI: http://localhost:5173

## API
API base: `/api/matter`

## Logs
Persistent logs are stored under `${HAMH_STORAGE_LOCATION}/logs`:
- `backend.log`
- `bridge-delete.log`
- `backend-crash.log`

## Design spec
The vNext design spec for the Rework branch is here:
- `docs/Design-Spec-vNext.md`

## Documentation
- `docs/Getting Started/Installation.md`
- `docs/Getting Started/Bridge Configuration.md`
- `docs/Developer Documentation/README.md`
- `docs/Developer Documentation/local-dev.md`

## License
See `LICENSE`.
