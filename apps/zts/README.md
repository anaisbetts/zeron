# zeron-remote

A thin CLI over the local [Zeron MCP tools](../../docs/mcp.md). It talks to the
running engine on `ws://127.0.0.1:$ZERON_IPC_PORT` (default 27654) — the same
surface `zeron mcp` serves.

```
zeron-remote --json cloud ls
zeron-remote --json cloud wake <device-or-chat> [--wait]
zeron-remote --json cloud stop <device-or-chat>
zeron-remote --json cloud discard <device-or-chat>
zeron-remote --json sessions create --kind cloud [--harness] [--model] [--title] [--prompt] [--wait]
zeron-remote --json sessions ls
zeron-remote --json devices ls
```

`--kind cloud` is a standalone session on a new cloud machine. It refuses
`--device`, `--project`, `--parent`, and `--cwd`. The harness defaults to
`claude-code` without querying the VM (it may not exist yet). Without `--wait`,
the command returns after provision + createChat; the VM boots in the
background and a queued prompt wakes it.

`cloud` targets resolve like devices (id or exact name) **or** the bound chat
(id / unique prefix / exact title). One cloud device hosts one chat.

`discard` deletes the VM disk; the chat stays. The next send recreates the
machine. That is distinct from deleting the device row.

Harness catalogs on an asleep machine fail until it is up. After a send, use
`sessions wait` before inspecting the host.
