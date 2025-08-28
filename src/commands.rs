use serde::Serialize;
use std::sync::mpsc::Sender;
use std::{env, io};

#[cfg(feature = "logging")]
use tracing::{error, info};

use crate::protocol::Protocol;

#[derive(Serialize)]
pub struct Action {
    pub kind: ActionKind,
    pub argv: Vec<String>,
    pub argc: usize,
}

#[derive(Serialize)]
pub enum ActionKind {
    HELP,
    BROADCAST,
    MESSAGE,
    NUKE,
    OTHER,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Action".to_string())
        )
    }
}

pub fn input(sender: Sender<Protocol>) -> ! {
    let prefix = env::var("CMD_PREFIX").expect("[INPUT] CMD_PREFIX must be set");

    #[cfg(feature = "logging")]
    info!("[INPUT] Listening for commands with prefix: '{}'", prefix);

    loop {
        // Take input from the console.
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                #[cfg(feature = "logging")]
                error!("Could not read stdin: {e}");
                continue;
            }
        }

        if !input.starts_with(prefix.as_str()) {
            continue;
        }

        #[cfg(feature = "logging")]
        info!("[INPUT] Parsing command.");

        // Sanitize and Tokenize
        let input = input[prefix.len()..].trim().to_string();
        let argv: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let argc = argv.len();

        // TODO: Add a revive command that brings all dead monsters back to life

        let kind = match argv[0].to_ascii_lowercase().as_str() {
            "broadcast" => ActionKind::BROADCAST,
            "help" => ActionKind::HELP,
            "message" => ActionKind::MESSAGE,
            "nuke" => ActionKind::NUKE,
            _ => ActionKind::OTHER,
        };

        sender
            .send(Protocol::Command(Action { kind, argv, argc }))
            .unwrap_or_else(|_| {
                #[cfg(feature = "logging")]
                error!("[INPUT] Failed to send INPUT packet");
            })
    }
}
