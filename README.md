# Home Assistant Matter Hub (Fork)

![Home-Assistant-Matter-Hub](./docs/assets/hamh-logo-small.png)

---

## Fork notice

This repository is a fork of `t0bst4r/home-assistant-matter-hub`.
Maintained by **parnunu**. Development work is performed by AI only.

---

## About

Home Assistant Matter Hub (HAMH) simulates Matter bridges that expose your Home Assistant entities to Matter controllers
like Alexa, Apple Home, and Google Home. It uses local communication and does not require cloud or port forwarding.

---

## Project status

Upstream announced end of maintenance in January 2026. This fork continues independently for experimentation and
maintenance as needed.

---

## Installation (Home Assistant Add-on)

The supported production install method is the Home Assistant add-on.

1. In Home Assistant, go to Settings -> Add-ons -> Add-on Store.
2. Add this add-ons repository URL:
   - https://github.com/parnunu/home-assistant-addons
3. Refresh the Add-on Store, then install Home Assistant Matter Hub.
4. Configure as needed and click Start.

For full details, see:
- docs/Getting Started/Installation.md

---

## Documentation

Start here for guides, limitations, and troubleshooting:

- docs/Getting Started/Installation.md
- docs/Getting Started/Bridge Configuration.md
- docs/Developer Documentation/README.md
- docs/Developer Documentation/local-dev.md

---

## Home Assistant Add-on

Production deployment remains the Home Assistant add-on.
The add-on build files live in:

- apps/home-assistant-matter-hub/addon.Dockerfile
- apps/home-assistant-matter-hub/addon.docker-entrypoint.sh
- apps/home-assistant-matter-hub/build.js

---

## Local development (PC)

Requirements:
- Node.js 22
- pnpm 10.28.1

Run backend:

pnpm --filter @home-assistant-matter-hub/backend run serve

Run frontend:

pnpm --filter @home-assistant-matter-hub/frontend run dev

---

## Contributing

This fork follows an evidence-first workflow and uses child issues + PRs per work item.
See AGENTS.md for the exact rules.

---

## License

See LICENSE.
