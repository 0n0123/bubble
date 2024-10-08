// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use flume::Receiver;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use zenoh::{prelude::sync::*, publication::Publisher, subscriber::Subscriber};


static SESSION: OnceLock<Session> = OnceLock::new();
static PUBLISHER: OnceLock<Publisher> = OnceLock::new();
static APP: OnceLock<AppHandle> = OnceLock::new();

#[tauri::command]
fn enter_room(server: &str, room: &str) {
    if let Err(_) = prepare_session(server) {
        eprintln!("Cannot connect to server {server}");
        return;
    }

    let session = match SESSION.get() {
        Some(s) => s,
        None => {
            notify(Notice::new("Cannot connect to server."));
            eprintln!("Session is illegal state.");
            return;
        }
    };

    let key = format!("bubble/message/{}", room);

    match session.declare_subscriber(key.clone()).res() {
        Ok(s) => tauri::async_runtime::spawn(listen_message(s)),
        Err(_) => {
            notify(Notice::new("Cannot connect to server."));
            eprintln!("Failed to declare subscriber {key}");
            return;
        }
    };

    match session.declare_publisher(key.clone()).res() {
        Ok(p) => PUBLISHER.set(p).expect("Failed to fix publisher."),
        Err(_) => {
            notify(Notice::new("Cannot connect to server."));
            eprintln!("Failed to declare publisher {key}");
            return;
        }
    };

    eprintln!("Enter: {room}");
}

#[tauri::command]
fn send_message(name: &str, message: &str) {
    let publisher = match PUBLISHER.get() {
        Some(p) => p,
        None => {
            notify(Notice::new("Cannot connect to server."));
            eprintln!("Failed to get publisher.");
            return;
        }
    };

    let mes = Message {
        name: name.to_owned(),
        message: message.to_owned(),
    };
    let data = rmp_serde::to_vec(&mes).unwrap();
    if let Err(_) = publisher.put(data).res() {
        notify(Notice::new("Cannot connect to server."));
        eprintln!("Failed to send message");
    }
}

async fn listen_message(subscriber: Subscriber<'_, Receiver<Sample>>) {
    while let Ok(sample) = subscriber.recv() {
        let read = sample.payload.reader();
        if let Ok(mes) = rmp_serde::from_read::<_, Message>(read) {
            if let Some(app) = APP.get() {
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

fn prepare_session(server: &str) -> Result<()> {
    let endpoint = EndPoint::new("tcp", server, "", "").map_err(|_| anyhow!("Failed to setup server config."))?;
    let config = zenoh::config::client(vec![endpoint]);
    let session = zenoh::open(config)
        .res()
        .map_err(|_| anyhow!("Failed to open zenoh session."))?;

    SESSION
        .set(session)
        .map_err(|_| anyhow!("Failed to fix zenoh session."))?;

    Ok(())
}

#[derive(Serialize, Clone)]
struct Notice {
    message: String,
}

impl Notice {
    fn new(message: &str) -> Self {
        Notice {
            message: message.to_owned(),
        }
    }
}

fn notify(notice: Notice) {
    if let Some(app) = APP.get() {
        let _ = app.emit_all("notice", notice)
            .inspect_err(|e| eprintln!("{e}"));
    }
}
