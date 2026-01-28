# AGENTS.md

## Evidence-First Workflow
- Do not change behavior without evidence.
- Reproduce once, capture logs/stack traces, and summarize findings.
- Propose the smallest fix that matches the evidence.

## Issue Workflow
- Keep the main issue intact.
- Create a child issue for each work item.
- Close the child issue after the fix is pushed to `main`.

## Logging
- Use persistent logs for debugging (do not rely on console only).
- Keep logs under `${HAMH_STORAGE_LOCATION}/logs`.
- Preferred log files:
  - `bridge-delete.log` for delete flow
  - `backend-crash.log` for uncaught errors

## Local Dev
- Node.js 22 is required.
- Use `pnpm@10.28.1`.
- Run backend: `pnpm --filter @home-assistant-matter-hub/backend run serve`
- Run frontend: `pnpm --filter @home-assistant-matter-hub/frontend run dev`
