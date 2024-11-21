use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use zenoh::{handlers::FifoChannelHandler, pubsub::{Publisher, Subscriber}, sample::Sample, Session};

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Message {
    name: String,
    message: String,
}

static APP: OnceLock<AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.app_handle();
            APP.set(app_handle.clone())
                .map_err(|_| "Failed to initialize application.")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![enter_room, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

static SESSION: OnceLock<Session> = OnceLock::new();
static PUBLISHER: OnceLock<Publisher> = OnceLock::new();

async fn prepare_session(server_ip: &str) -> Result<()> {
    let config = format!(r#"{{
        mode: "client",
        connect: {{
            endpoints: ["tcp/{server_ip}"]
        }}
    }}
    "#);
    let config =
        zenoh::Config::from_json5(&config).map_err(|_| "Cannot configure Zenoh.")?;
    let session = zenoh::open(config)
        .await
        .map_err(|_| "Failed to open zenoh session.")?;

    SESSION.set(session).unwrap();

    Ok(())
}

#[tauri::command]
async fn enter_room(server_ip: &str, room_id: &str, user_name: &str) -> Result<()> {
    if let Err(e) = prepare_session(server_ip).await {
        Notice::new(e).notify();
        return Err("Cannot connect to server.");
    }
    let session = SESSION.get().unwrap();

    let key = format!("bubble/message/{}", room_id);

    let mes_sub = session
        .declare_subscriber(key.clone())
        .await
        .map_err(|_| "Cannot declare subscriber.")?;
    tauri::async_runtime::spawn(listen_message(mes_sub));

    let mes_pub = session
        .declare_publisher(key.clone())
        .await
        .map_err(|_| "Cannot declare publisher.")?;
    PUBLISHER.set(mes_pub).expect("Failed to fix publisher.");

    println!("Enter: {}", room_id);

    Ok(())
}

async fn listen_message(subscriber: Subscriber<FifoChannelHandler<Sample>>) -> Result<()> {
    while let Ok(sample) = subscriber.recv() {
        let payload = sample.payload();
        let read = payload.reader();
        if let Ok(mes) = rmp_serde::from_read::<_, Message>(read) {
            if let Some(app) = APP.get() {
                app.emit("message", mes)
                    .expect("Failed to emit message.");
            }
        }
    }
    Ok(())
}

#[tauri::command]
async fn send_message(name: &str, message: &str) -> Result<()> {
    let publisher = match PUBLISHER.get() {
        Some(p) => p,
        None => {
            Notice::new("Cannot connect to server.").notify();
            return Err("Failed to get publisher.");
        }
    };

    let mes = Message {
        name: name.to_owned(),
        message: message.to_owned(),
    };
    let data = rmp_serde::to_vec(&mes).unwrap();
    match publisher.put(data).await {
        Ok(_) => Ok(()),
        Err(_) => {
            Notice::new("Cannot connect to server.").notify();
            Err("Failed to send message")
        }
    }
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

    fn notify(&self) {
        if let Some(app) = APP.get() {
            let _ = app.emit("notice", self).inspect_err(|e| eprintln!("{e}"));
        }
    }
}
