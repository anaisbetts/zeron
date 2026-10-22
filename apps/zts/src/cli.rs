//! Clap surface for `zeron-remote`.

use clap::{Parser, Subcommand};

/// Drive the local Zeron engine: sessions, devices, and cloud machines.
#[derive(Debug, Parser)]
#[command(name = "zeron-remote", version, about)]
pub struct Cli {
    /// Print one JSON object (the default is pretty-printed JSON).
    #[arg(long, global = true)]
    pub json: bool,
    /// Engine IPC port (`ZERON_IPC_PORT`, default 27654).
    #[arg(long, global = true)]
    pub port: Option<u16>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Who this CLI is talking to.
    Whoami,
    /// Chats: list, create, send, read, wait.
    #[command(subcommand)]
    Sessions(SessionsCmd),
    /// Host devices, including cloud machines.
    #[command(subcommand)]
    Devices(DevicesCmd),
    /// Projects (folders on a device).
    #[command(subcommand)]
    Projects(ProjectsCmd),
    /// Agent harnesses on the local engine.
    #[command(subcommand)]
    Harnesses(HarnessesCmd),
    /// Models a harness offers on this device.
    #[command(subcommand)]
    Models(ModelsCmd),
    /// Edge-managed cloud machines (one chat per machine).
    #[command(subcommand)]
    Cloud(CloudCmd),
}

#[derive(Debug, Clone, Subcommand)]
pub enum SessionsCmd {
    /// List chats (newest activity first).
    Ls {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        include_archived: bool,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// One chat's metadata and live status.
    Get { chat: String },
    /// Create a chat, or a standalone cloud session with `--kind cloud`.
    Create {
        /// `chat` (default) or `cloud`. Cloud refuses `--device`, `--project`,
        /// `--parent`, and `--cwd`.
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        device: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        harness: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        reasoning: Option<String>,
        #[arg(long)]
        sandbox: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        wait: bool,
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Send a message. `auto` runs, steers, or queues from the chat's status.
    Send {
        chat: String,
        text: Vec<String>,
        #[arg(long, default_value = "auto")]
        mode: String,
        #[arg(long)]
        wait: bool,
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Block until the chat is no longer working.
    Wait {
        chat: String,
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Read the transcript.
    Read {
        chat: String,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        offset: Option<usize>,
        #[arg(long)]
        include_reasoning: bool,
    },
    /// Archive (or restore) a chat.
    Archive {
        chat: String,
        #[arg(long)]
        restore: bool,
    },
    /// Stop the running turn.
    Interrupt { chat: String },
}

#[derive(Debug, Clone, Subcommand)]
pub enum DevicesCmd {
    Ls,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProjectsCmd {
    Ls,
}

#[derive(Debug, Clone, Subcommand)]
pub enum HarnessesCmd {
    Ls,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModelsCmd {
    Ls {
        #[arg(long)]
        harness: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum CloudCmd {
    /// Cloud machines (devices that carry `cloud`).
    Ls,
    /// Wake a machine. Target is a device id/name or its chat id/prefix/title.
    Wake {
        target: String,
        #[arg(long)]
        wait: bool,
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Put a machine to sleep. Files survive.
    Stop { target: String },
    /// Delete the VM disk. The chat stays; the next send recreates the machine.
    Discard { target: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("zeron-remote").chain(args.iter().copied()))
            .unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn sessions_create_kind_cloud_parses() {
        let cli = parse(&[
            "--json", "sessions", "create", "--kind", "cloud", "--title", "Boat", "--prompt", "hi",
        ]);
        assert!(cli.json);
        match cli.command {
            Command::Sessions(SessionsCmd::Create {
                kind,
                title,
                prompt,
                device,
                parent,
                project,
                cwd,
                ..
            }) => {
                assert_eq!(kind.as_deref(), Some("cloud"));
                assert_eq!(title.as_deref(), Some("Boat"));
                assert_eq!(prompt.as_deref(), Some("hi"));
                assert!(device.is_none());
                assert!(parent.is_none());
                assert!(project.is_none());
                assert!(cwd.is_none());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn cloud_lifecycle_parses() {
        match parse(&["cloud", "ls"]).command {
            Command::Cloud(CloudCmd::Ls) => {}
            other => panic!("{other:?}"),
        }
        match parse(&["cloud", "wake", "Boat", "--wait"]).command {
            Command::Cloud(CloudCmd::Wake { target, wait, .. }) => {
                assert_eq!(target, "Boat");
                assert!(wait);
            }
            other => panic!("{other:?}"),
        }
        match parse(&["cloud", "stop", "Cloud Alpha"]).command {
            Command::Cloud(CloudCmd::Stop { target }) => {
                assert_eq!(target, "Cloud Alpha")
            }
            other => panic!("{other:?}"),
        }
        match parse(&["cloud", "discard", "chat-cloud-1"]).command {
            Command::Cloud(CloudCmd::Discard { target }) => {
                assert_eq!(target, "chat-cloud-1")
            }
            other => panic!("{other:?}"),
        }
    }
}
