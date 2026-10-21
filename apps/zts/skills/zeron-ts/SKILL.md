---
name: zeron-ts
description: Drive Zeron sessions and cloud machines from the zeron-remote CLI.
---

# zeron-remote

`zeron-remote` is a thin CLI over the local Zeron MCP tools. Prefer `--json`.
It talks to the running engine (`ZERON_IPC_PORT`, default 27654).

## Cloud sessions

Create a standalone session on a new Boat VM. Do not pass `--device`,
`--project`, `--parent`, or `--cwd`.

```
zeron-remote --json sessions create --kind cloud --title "Investigate" --prompt "…"
```

Harness defaults to `claude-code`. The VM may still be booting when this
returns; a send wakes it. To block on the first turn, add `--wait`.

```
zeron-remote --json cloud ls
zeron-remote --json cloud wake <device-or-chat> --wait
zeron-remote --json cloud stop <device-or-chat>
zeron-remote --json cloud discard <device-or-chat>
```

Targets are a device id/name or the bound chat (id, unique prefix, or exact
title). `discard` deletes the disk; the chat stays and the next send
recreates the machine.

## Ordinary sessions

```
zeron-remote --json sessions ls
zeron-remote --json sessions create --project <path-or-name> --prompt "…"
zeron-remote --json sessions send <chat> "…" --wait
zeron-remote --json sessions read <chat>
zeron-remote --json sessions wait <chat>
```

`devices ls` includes `cloud` on cloud machines (state, generation, labels).
`sessions ls` / `get` attach the same object so an asleep chat is visible.

## Asleep machines

Sending to a cloud chat wakes it. `harnesses ls` and `models ls` talk to the
local engine, not the VM. Catalogs and files on the cloud host fail until
the machine is running — after a send, `sessions wait`, then inspect.
