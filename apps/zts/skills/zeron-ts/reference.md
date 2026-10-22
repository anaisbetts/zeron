# zeron-remote reference

Binary: `zeron-remote`. Global flags: `--json` (one-line JSON), `--port`
(`ZERON_IPC_PORT`, default 27654).

## sessions

| Command | MCP tool | Notes |
| ------- | -------- | ----- |
| `sessions ls [--project] [--device] [--parent] [--include-archived] [--limit]` | `list_chats` | Newest activity first. Cloud chats include `cloud`. |
| `sessions get <chat>` | `get_chat` | Chat id, unique prefix, or exact title. |
| `sessions create [--kind chat\|cloud] …` | `create_chat` | See create flags below. |
| `sessions send <chat> <text…> [--mode auto\|run\|steer\|queue] [--wait]` | `send_message` | A send to a cloud chat wakes its machine. |
| `sessions wait <chat> [--timeout-secs]` | `wait_for_turn` | |
| `sessions read <chat> [--limit] [--offset] [--include-reasoning]` | `read_chat` | Reads the local replica (safe while the VM sleeps). |
| `sessions archive <chat> [--restore]` | `archive_chat` | |
| `sessions interrupt <chat>` | `interrupt_chat` | |

Create flags: `--kind`, `--project`, `--device`, `--parent`, `--harness`,
`--model`, `--reasoning`, `--sandbox`, `--title`, `--branch`, `--cwd`,
`--prompt`, `--wait`, `--timeout-secs`.

`--kind cloud` is standalone: refuse `--device`, `--project`, `--parent`,
`--cwd`. Always a scratch cwd (`~/.zeron/scratch/<date>-<id8>`). Harness
defaults to `claude-code` without `ListHarnesses`. Optional `--prompt`
queues a Run (the engine nudges DeviceRoom; the VM boots in the
background). Without `--wait`, return after provision + createChat.

## cloud

| Command | MCP tool | Notes |
| ------- | -------- | ----- |
| `cloud ls` | `list_devices` (filtered) | Devices whose `cloud` object is set. |
| `cloud wake <target> [--wait]` | `cloud_machine` action `wake` | `--wait` blocks on `WatchDevices` until `running` or `error`. |
| `cloud stop <target>` | `cloud_machine` action `stop` | Sleep; files survive. |
| `cloud discard <target>` | `cloud_machine` action `discard` | Deletes the VM disk. The chat stays. |

`<target>` is a device id or exact name, or the bound chat's id / unique
prefix / exact title. One cloud device hosts one chat.

`discard` is not `devices remove`: the device row and chat remain; the next
send provisions a fresh machine.

## devices / projects / catalogs

| Command | MCP tool |
| ------- | -------- |
| `devices ls` | `list_devices` |
| `projects ls` | `list_projects` |
| `harnesses ls` | `list_harnesses` |
| `models ls --harness <id>` | `list_models` |
| `whoami` | `whoami` |

`list_devices` includes `cloud` when present: `provider`, `state`, `chatId`,
`generation`, `error`, plus `statusLabel` / `shortLabel` / `recreated`.

`harnesses ls` and `models ls` query the **local** engine. Do not use them
to pick a harness for `--kind cloud` — the VM may not exist yet.

## Engine RPCs

Cloud create: `CloudProvision { chatId }` then `Mutate createChat` with the
same id, no space, scratch cwd. On createChat failure the CLI/tools issue
`CloudMachine { action: discard }`.

Lifecycle: `CloudMachine { deviceId, action: wake\|stop\|discard }`.
