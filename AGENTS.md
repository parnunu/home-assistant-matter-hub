# AGENTS.md

## Evidence-First Workflow
- Do not change behavior without evidence.
- Reproduce once, capture logs/stack traces, and summarize findings.
- Propose the smallest fix that matches the evidence.

## Issue Workflow
- Keep the main issue intact.
- Create a child issue for each work item.
- Close the child issue after the fix is pushed to `main`.

## GitHub Admin Workflow
- Keep the fork synced with upstream before starting work.
- Do work on a branch per child issue.
- Open a PR for each branch and link it to the child issue.
- Close the child issue only after the PR is opened and pushed.

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
