//! Map CLI commands onto MCP tools and print the result.

use std::sync::Arc;

use anyhow::{Context, bail};
use serde_json::{Value, json};

use zeron_mcp::{Origin, Tools, Zeron};

use crate::cli::{
    Cli, CloudCmd, Command, DevicesCmd, HarnessesCmd, ModelsCmd, ProjectsCmd, SessionsCmd,
};

const DEFAULT_PORT: u16 = 27654;

/// Connect, dispatch, print. The theme of this crate.
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let tools = connect(cli.port)?;
    let value = dispatch(&tools, &cli.command).await?;
    print_result(cli.json, &value)
}

/// Build the in-process MCP tool surface against the local engine.
pub fn connect(port: Option<u16>) -> anyhow::Result<Tools> {
    let port = port
        .or_else(|| {
            std::env::var("ZERON_IPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
        })
        .unwrap_or(DEFAULT_PORT);
    let zeron = Zeron::new(format!("ws://127.0.0.1:{port}"), Origin::from_env());
    Ok(Tools::new(Arc::new(zeron)))
}

/// Run one CLI command against an already-built tool surface (tests).
pub async fn dispatch(tools: &Tools, command: &Command) -> anyhow::Result<Value> {
    match command {
        Command::Cloud(CloudCmd::Ls) => {
            let listed = tools
                .call("list_devices", json!({}))
                .await
                .map_err(anyhow::Error::msg)?;
            let machines = listed
                .get("devices")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|device| device.get("cloud").map(Value::is_object).unwrap_or(false))
                .cloned()
                .collect::<Vec<_>>();
            Ok(json!({ "machines": machines }))
        }
        other => {
            let (name, args) = tool_call(other)?;
            tools.call(&name, args).await.map_err(anyhow::Error::msg)
        }
    }
}

/// The MCP tool and arguments one command becomes. Cloud create refuses
/// host/project/parent/cwd here so the CLI fails before it talks to the engine.
pub fn tool_call(command: &Command) -> anyhow::Result<(String, Value)> {
    match command {
        Command::Whoami => Ok(("whoami".into(), json!({}))),
        Command::Sessions(SessionsCmd::Ls {
            project,
            device,
            parent,
            include_archived,
            limit,
        }) => {
            let mut args = json!({ "include_archived": include_archived });
            insert_opt(&mut args, "project", project.as_deref());
            insert_opt(&mut args, "device", device.as_deref());
            insert_opt(&mut args, "parent", parent.as_deref());
            if let Some(limit) = limit {
                args["limit"] = json!(limit);
            }
            Ok(("list_chats".into(), args))
        }
        Command::Sessions(SessionsCmd::Get { chat }) => {
            Ok(("get_chat".into(), json!({ "chat": chat })))
        }
        Command::Sessions(SessionsCmd::Create {
            kind,
            project,
            device,
            parent,
            harness,
            model,
            reasoning,
            sandbox,
            title,
            branch,
            cwd,
            prompt,
            wait,
            timeout_secs,
        }) => {
            let kind = kind.as_deref().map(str::trim).filter(|k| !k.is_empty());
            match kind {
                None | Some("chat") => {}
                Some("cloud") => {
                    refuse_cloud_extra("--device", device.as_deref())?;
                    refuse_cloud_extra("--project", project.as_deref())?;
                    refuse_cloud_extra("--parent", parent.as_deref())?;
                    refuse_cloud_extra("--cwd", cwd.as_deref())?;
                }
                Some(other) => bail!("unknown --kind {other:?}; expected chat or cloud"),
            }
            let mut args = json!({ "wait": wait });
            if let Some(kind) = kind {
                args["kind"] = json!(kind);
            }
            insert_opt(&mut args, "project", project.as_deref());
            insert_opt(&mut args, "device", device.as_deref());
            insert_opt(&mut args, "parent", parent.as_deref());
            insert_opt(&mut args, "harness", harness.as_deref());
            insert_opt(&mut args, "model", model.as_deref());
            insert_opt(&mut args, "reasoning", reasoning.as_deref());
            insert_opt(&mut args, "sandbox", sandbox.as_deref());
            insert_opt(&mut args, "title", title.as_deref());
            insert_opt(&mut args, "branch", branch.as_deref());
            insert_opt(&mut args, "cwd", cwd.as_deref());
            insert_opt(&mut args, "prompt", prompt.as_deref());
            if let Some(timeout_secs) = timeout_secs {
                args["timeout_secs"] = json!(timeout_secs);
            }
            Ok(("create_chat".into(), args))
        }
        Command::Sessions(SessionsCmd::Send {
            chat,
            text,
            mode,
            wait,
            timeout_secs,
        }) => {
            let text = text.join(" ");
            if text.trim().is_empty() {
                bail!("sessions send needs a message");
            }
            let mut args = json!({
                "chat": chat,
                "text": text,
                "mode": mode,
                "wait": wait,
            });
            if let Some(timeout_secs) = timeout_secs {
                args["timeout_secs"] = json!(timeout_secs);
            }
            Ok(("send_message".into(), args))
        }
        Command::Sessions(SessionsCmd::Wait { chat, timeout_secs }) => {
            let mut args = json!({ "chat": chat });
            if let Some(timeout_secs) = timeout_secs {
                args["timeout_secs"] = json!(timeout_secs);
            }
            Ok(("wait_for_turn".into(), args))
        }
        Command::Sessions(SessionsCmd::Read {
            chat,
            limit,
            offset,
            include_reasoning,
        }) => {
            let mut args = json!({
                "chat": chat,
                "include_reasoning": include_reasoning,
            });
            if let Some(limit) = limit {
                args["limit"] = json!(limit);
            }
            if let Some(offset) = offset {
                args["offset"] = json!(offset);
            }
            Ok(("read_chat".into(), args))
        }
        Command::Sessions(SessionsCmd::Archive { chat, restore }) => Ok((
            "archive_chat".into(),
            json!({ "chat": chat, "archived": !restore }),
        )),
        Command::Sessions(SessionsCmd::Interrupt { chat }) => {
            Ok(("interrupt_chat".into(), json!({ "chat": chat })))
        }
        Command::Devices(DevicesCmd::Ls) => Ok(("list_devices".into(), json!({}))),
        Command::Projects(ProjectsCmd::Ls) => Ok(("list_projects".into(), json!({}))),
        Command::Harnesses(HarnessesCmd::Ls) => Ok(("list_harnesses".into(), json!({}))),
        Command::Models(ModelsCmd::Ls { harness }) => {
            Ok(("list_models".into(), json!({ "harness": harness })))
        }
        Command::Cloud(CloudCmd::Ls) => {
            bail!("cloud ls is handled in dispatch")
        }
        Command::Cloud(CloudCmd::Wake {
            target,
            wait,
            timeout_secs,
        }) => {
            let mut args = json!({
                "action": "wake",
                "device": target,
                "wait": wait,
            });
            if let Some(timeout_secs) = timeout_secs {
                args["timeout_secs"] = json!(timeout_secs);
            }
            Ok(("cloud_machine".into(), args))
        }
        Command::Cloud(CloudCmd::Stop { target }) => Ok((
            "cloud_machine".into(),
            json!({ "action": "stop", "device": target }),
        )),
        Command::Cloud(CloudCmd::Discard { target }) => Ok((
            "cloud_machine".into(),
            json!({ "action": "discard", "device": target }),
        )),
    }
}

fn refuse_cloud_extra(flag: &str, value: Option<&str>) -> anyhow::Result<()> {
    if value.map(str::trim).is_some_and(|v| !v.is_empty()) {
        bail!("--kind cloud cannot have {flag}");
    }
    Ok(())
}

fn insert_opt(args: &mut Value, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) {
        args[key] = json!(value);
    }
}

fn print_result(one_line: bool, value: &Value) -> anyhow::Result<()> {
    let text = if one_line {
        serde_json::to_string(value)
    } else {
        serde_json::to_string_pretty(value)
    }
    .context("serialize result")?;
    println!("{text}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command, SessionsCmd};
    use clap::Parser;

    fn parse(args: &[&str]) -> Command {
        Cli::try_parse_from(std::iter::once("zeron-remote").chain(args.iter().copied()))
            .unwrap()
            .command
    }

    #[test]
    fn sessions_create_kind_cloud_dispatches_create_chat() {
        let (name, args) = tool_call(&parse(&[
            "sessions",
            "create",
            "--kind",
            "cloud",
            "--title",
            "Boat",
            "--harness",
            "claude-code",
        ]))
        .unwrap();
        assert_eq!(name, "create_chat");
        assert_eq!(args["kind"], "cloud");
        assert_eq!(args["title"], "Boat");
        assert_eq!(args["harness"], "claude-code");
        assert!(args.get("device").is_none());
        assert!(args.get("parent").is_none());
        assert!(args.get("project").is_none());
        assert!(args.get("cwd").is_none());
    }

    #[test]
    fn sessions_create_kind_cloud_refuses_device_and_parent() {
        let err = tool_call(&parse(&[
            "sessions", "create", "--kind", "cloud", "--device", "Laptop",
        ]))
        .unwrap_err()
        .to_string();
        assert!(err.contains("--kind cloud cannot have --device"), "{err}");

        let err = tool_call(&parse(&[
            "sessions", "create", "--kind", "cloud", "--parent", "Alpha",
        ]))
        .unwrap_err()
        .to_string();
        assert!(err.contains("--kind cloud cannot have --parent"), "{err}");
    }

    #[test]
    fn cloud_commands_dispatch_to_cloud_machine() {
        let (name, args) = tool_call(&parse(&["cloud", "wake", "Boat", "--wait"])).unwrap();
        assert_eq!(name, "cloud_machine");
        assert_eq!(args["action"], "wake");
        assert_eq!(args["device"], "Boat");
        assert_eq!(args["wait"], true);

        let (name, args) = tool_call(&parse(&["cloud", "stop", "Cloud Alpha"])).unwrap();
        assert_eq!(name, "cloud_machine");
        assert_eq!(args["action"], "stop");
        assert_eq!(args["device"], "Cloud Alpha");

        let (name, args) = tool_call(&parse(&["cloud", "discard", "chat-cloud-1"])).unwrap();
        assert_eq!(name, "cloud_machine");
        assert_eq!(args["action"], "discard");
        assert_eq!(args["device"], "chat-cloud-1");
    }

    #[test]
    fn cloud_ls_is_not_a_direct_tool_call() {
        let err = tool_call(&parse(&["cloud", "ls"])).unwrap_err().to_string();
        assert!(err.contains("cloud ls is handled in dispatch"), "{err}");
        match parse(&["sessions", "get", "alpha"]) {
            Command::Sessions(SessionsCmd::Get { chat }) => assert_eq!(chat, "alpha"),
            other => panic!("{other:?}"),
        }
        match parse(&["whoami"]) {
            Command::Whoami => {}
            other => panic!("{other:?}"),
        }
    }
}
