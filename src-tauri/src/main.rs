// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use flume::Receiver;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use zenoh::{prelude::sync::*, publication::Publisher, subscriber::Subscriber};

#[tauri::command]
fn enter_room(room: &str) {
    let session = match SESSION.get() {
        Some(s) => s,
        None => {
            println!("Session is illegal state.");
            return;
        }
    };

    let key = format!("bubble/message/{}", room);

    match session.declare_subscriber(key.clone()).res() {
        Ok(s) => SUBSCRIBER.set(s),
        Err(_) => {
            println!("Failed to declare subscriber: {}", key);
            return;
        }
    };

    match session.declare_publisher(key.clone()).res() {
        Ok(p) => PUBLISHER.set(p),
        Err(_) => {
            println!("Failed to declare publisher: {}", key);
            return;
        }
    };

    println!("Enter: {}", room);
}

#[tauri::command]
fn send_message(room: &str, name: &str, message: &str) {
    println!("{}/{} > {}", room, name, message);
}

static SESSION: OnceLock<Session> = OnceLock::new();
static SUBSCRIBER: OnceLock<Subscriber<'_, Receiver<Sample>>> = OnceLock::new();
static PUBLISHER: OnceLock<Publisher> = OnceLock::new();

#[derive(Clone, Serialize)]
struct Message {
    room: String,
    name: String,
    message: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.config.exists() {
        bail!("Failed to read config.")
    }

    let config = read_config(&args.config)?;
    prepare_session(&config.server);

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![enter_room, send_message])
        .run(tauri::generate_context!())
        .map_err(|_| anyhow!("error while running tauri application"))
}

fn read_config(path: &Path) -> Result<Config> {
    let config = fs::read_to_string(path)?;
    toml::from_str::<Config>(&config).map_err(|_| anyhow!("Failed to parse config."))
}

fn prepare_session(server: &str) -> Result<()> {
    let endpoint = EndPoint::new("tcp", server, "", "")
        .map_err(|_| anyhow!("Server address cannot be parsed."))?;
    let config = zenoh::config::client(vec![endpoint]);
    let session = zenoh::open(config)
        .res()
        .map_err(|_| anyhow!("Failed to open zenoh session."))?;

    SESSION
        .set(session)
        .map_err(|_| anyhow!("Failed to fix zenoh session."))?;

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[clap(default_value = "./bubble.toml")]
    config: PathBuf,
}

#[derive(Deserialize)]
struct Config {
    server: String,
}
