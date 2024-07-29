#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

use std::net::TcpStream;

use semver::Version;
use serde_json::Value;
use tray_item::{IconSource, TrayItem};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

// Retaining the unsafe function in case it proves to be necessary later, will
// remove in a later PR if it proves unnecessary for certain.
//---
// fn hide_console_window() {
//     use std::ptr;
//     use winapi::um::{
//         wincon::GetConsoleWindow,
//         winuser::{ ShowWindow, SW_HIDE }
//     };

//     let window = unsafe { GetConsoleWindow() };

//     if window != ptr::null_mut() {
//         unsafe { ShowWindow(window, SW_HIDE); }
//     }
// }

#[tokio::main]
async fn main() {
    //Create tray icon using `tray-item-rs`
    let mut tray: TrayItem = {
        if let Ok(ti) = TrayItem::new(
        "GAT - GlazeWM Alternating Tiler",
        IconSource::Resource("main-icon")
        ) { ti } else {
            eprintln!("\nERR: Could not init System Tray!\n");
            panic!("\nMOR: Cannot continue runtime, please double-check your computer configuration!\n");
        }
    };

    //At this point, the tray itself was successfully created, so we use Windows' shit to hide the console.
    //Could be considered preemptive, given how the rest of the menu wasn't constructed yet, but eh.
    //hide_console_window();

    //Create menu label to show what it is.
    match tray.add_label("GAT - GlazeWM Alternating Tiler") {
        Ok(()) => {}
        Err(e) => eprintln!("\nERR: Could not add label to System Tray!\nRaw Error: {e}\n"),
    }

    //Create menu item for exiting program.
    let menu_item_function = || { std::process::exit(0) };

    if let Err(e) = tray.add_menu_item("Quit GAT", menu_item_function) {
        eprintln!("\nERR: Failed to add menu item! How did this even compile?\nRaw Error: {e}\n");
        panic!("\nMOR: Cannot continue runtime, please double-check your computer configuration!\n");
    }

    //Initial WS connection
    let (mut socket, _response) = connect(
        if let Ok(url) = Url::parse("ws://localhost:6123") {
            url
        } else {
            eprintln!("\nERR: Could not parse internal string to URL, was this mistyped or something?\n");
            panic!("\nMOR: Cannot continue runtime, please double-check your computer configuration!\n");
        }

    ).expect("\nERR: Can't connect to GWM WS\n");

    let version = get_version(&mut socket);
    let subscription: String = match version.major {
        3u64 => String::from("sub -e focus_changed"),
        _ => String::from("subscribe -e focus_changed"),
    };

    //If we error out attempting to subscribe to GWM, kill the process.
    if let Err(e) = socket.send(Message::Text(subscription)) {
        eprintln!("\nERR: Could not parse raw message data from initial GWM subscription! Raw error:\n{e}\n");
    } else {
        loop {
            let focus_msg = {
                let test = Value::default();
                let result = get_value(&mut socket);
                if result == test { continue } else { result }
            };

            if version.major == 3u64 {
                let tiling_direction = {
                    let _ = socket.send(Message::Text("query tiling-direction".into()));
                    get_value(&mut socket)
                };

                if let Some((x, y)) = get_window_height_width(&focus_msg["data"]["focusedContainer"]) {
                    size_tile_v3(
                        &mut socket, x, y,
                        tiling_direction["data"]["directionContainer"]["tilingDirection"].to_string()
                    ).unwrap();
                }
            } else if let Some((x, y)) = get_window_height_width(&focus_msg["data"]["focusedContainer"]) {
                size_tile(&mut socket, x, y).unwrap();
            }
        }
    }
}

fn get_value(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Value {
    match serde_json::from_str(
        socket
            .read()
            .expect("ERR: Could not read message!\n")
            .to_text()
            .unwrap_or_default()
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("\nERR: GWM WS MSG could not be parsed to Value!\nRaw error:\n{e}\n");
            Value::default()
        }
    }
}

fn get_version(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Version {
    let _query = socket.send(Message::Text("query app-metadata".into()));
    let reading = {
        let test = Value::default();
        let result = get_value(socket);

        if result == test { panic!("ERR: Could not retrieve version info! Aborting..."); } else { result }
    };

    if let Ok(v) = Version::parse(reading["data"]["version"].as_str().unwrap_or_default()) { v }
    else { Version::new(0, 0, 0) }
}

fn size_tile(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    x: f64,
    y: f64,
) -> Result<(), tungstenite::Error> {
    if x < y {
        socket.send(Message::Text(String::from(
            "command \"tiling direction vertical\""
        )))
    } else {
        socket.send(Message::Text(String::from(
            "command \"tiling direction horizontal\""
        )))
    }
}

fn size_tile_v3(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    x: f64,
    y: f64,
    tiling_direction: String
) -> Result<(), tungstenite::Error> {
    if (x < y && tiling_direction != "\"vertical\"")
    || (x > y && tiling_direction != "\"horizontal\"") {
        socket.send(Message::Text(String::from(
            "command toggle-tiling-direction"
        )))
    } else { Ok(()) }
}

fn get_window_height_width(v: &Value) -> Option<(f64, f64)> {
    v["width"]
        .as_f64()
        .and_then(|x| v["height"].as_f64().map(|y| (x, y)))
}
