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
use tauri::{AppHandle, Manager};
use zenoh::{prelude::sync::*, publication::Publisher, subscriber::Subscriber};

static SESSION: OnceLock<Session> = OnceLock::new();
static PUBLISHER: OnceLock<Publisher> = OnceLock::new();
static APP: OnceLock<AppHandle> = OnceLock::new();

#[tauri::command]
fn enter_room(room: &str) {
    let session = match SESSION.get() {
        Some(s) => s,
        None => {
            dbg!("Session is illegal state.");
            return;
        }
    };

    let key = format!("bubble/message/{}", room);

    match session.declare_subscriber(key.clone()).res() {
        Ok(s) => tauri::async_runtime::spawn(listen_message(s)),
        Err(_) => {
            dbg!("Failed to declare subscriber", key);
            return;
        }
    };

    match session.declare_publisher(key.clone()).res() {
        Ok(p) => PUBLISHER.set(p).expect("Failed to fix publisher."),
        Err(_) => {
            dbg!("Failed to declare publisher", key);
            return;
        }
    };

    dbg!("Enter:", room);
}

#[tauri::command]
fn send_message(name: &str, message: &str) {
    let publisher = match PUBLISHER.get() {
        Some(p) => p,
        None => {
            dbg!("Failed to get publisher.");
            return;
        }
    };

    let mes = Message {
        name: name.to_owned(),
        message: message.to_owned(),
    };
    let data = rmp_serde::to_vec(&mes).unwrap();
    if let Err(_) = publisher.put(data).res() {
        dbg!("Failed to send message", mes);
    }
}

async fn listen_message(subscriber: Subscriber<'_, Receiver<Sample>>) {
    while let Ok(sample) = subscriber.recv() {
        let read = sample.payload.reader();
        if let Ok(mes) = rmp_serde::from_read::<_, Message>(read) {
            if let Some(app) = APP.get() {
                dbg!("send message", &mes);
                app.emit_all("message", mes)
                    .expect("Failed to emit message.");
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Message {
    name: String,
    message: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.config.exists() {
        bail!("Failed to read config.")
    }

    let config = read_config(&args.config)?;
    prepare_session(config.server)?;

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            APP.set(app_handle)
                .map_err(|_| anyhow!("Failed to initialize application."))?;
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

fn prepare_session(servers: Vec<String>) -> Result<()> {
    let endpoints = servers
        .iter()
        .map(|server| EndPoint::new("tcp", server, "", ""))
        .filter_map(|r| r.ok());
    let config = zenoh::config::client(endpoints);
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
    server: Vec<String>,
}
