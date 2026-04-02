---
author: johndoe
description: Docker Compose shortcuts for daily container management
category: docker
tags: [docker, compose, containers]
profiles: [docker]
---

# Docker Compose Shortcuts

Aliases I use every day for managing local dev containers.

## Aliases

| Alias | Expands to | When I use it |
|-------|-----------|---------------|
| `dcu` | `docker compose up -d` | Start the stack in the background |
| `dcd` | `docker compose down` | Tear it all down |
| `dcl` | `docker compose logs -f` | Tail logs — `dcl` for all, `dcl api` for one service |
| `dcp` | `docker compose pull` | Pull latest images before restarting |
| `dcr` | `docker compose restart` | Quick restart without rebuilding |
| `dps` | `docker ps --format 'table ...'` | Clean status table with names, status, ports |

## Typical workflow

```bash
dcp                  # pull latest images
dcu                  # start everything
dcl api              # watch the api logs
# ... work work work ...
dcr api              # restart after code change
dcd                  # done for the day
```

## Notes

- Requires **Docker** and **Docker Compose v2** (the `docker compose` subcommand, not the old `docker-compose` binary)
- `dps` uses a custom format string — shows only name, status, and ports in a clean table
