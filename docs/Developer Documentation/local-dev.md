# Local PC Development (Remote Home Assistant)

This guide runs the app locally on your PC while connecting to a remote Home Assistant instance.
Production deployment remains the Home Assistant add-on.

## Prerequisites

- Node.js 22
- pnpm 10.x
- A Home Assistant Long-Lived Access Token

## Configure environment

1. Copy the sample file to `.env` at the repo root.
2. Fill in your Home Assistant URL and access token.

Example:
```
HAMH_HOME_ASSISTANT_URL="http://YOUR_HA_HOST:8123/"
HAMH_HOME_ASSISTANT_ACCESS_TOKEN="YOUR_LONG_LIVED_TOKEN"
HAMH_STORAGE_LOCATION=~/.hamh-development
HAMH_LOG_LEVEL=debug
```

## Install dependencies

```
pnpm install
```

## Run locally (backend + frontend)

```
pnpm run dev:pc
```

- Backend API runs on `http://localhost:8482`
- Frontend dev server runs on `http://localhost:5173` and proxies `/api` to the backend

## Backend only

```
pnpm run dev:pc:backend
```

## Frontend only

```
pnpm run dev:pc:frontend
```

## Debugging (VS Code)

Use the launch configs:

- `Backend: Serve (inspect)`
- `Frontend: Vite`
- `App: Backend + Frontend`
- `Tests: Vitest (backend)`
