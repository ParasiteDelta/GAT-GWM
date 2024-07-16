#![windows_subsystem = "windows"]

use std::net::TcpStream;

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
    let mut tray: TrayItem;

    //TrayItem data init - add program name, icon, append to raw TrayItem object.
    match TrayItem::new(
        "GAT - GlazeWM Alternating Tiler",
        IconSource::Resource("main-icon"),
    ) {
        Ok(ti) => tray = ti,
        Err(tierr) => {
            eprintln!("\nERR: Could not init System Tray!\nRaw Error: {tierr}\n");
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
    let menu_item_function = || {
        std::process::exit(0);
    };
    match tray.add_menu_item("Quit GAT", menu_item_function) {
        Ok(()) => {}
        Err(e) => {
            eprintln!(
                "\nERR: Failed to add menu item! How did this even compile?\nRaw Error: {e}\n"
            );
            panic!("\nMOR: Cannot continue runtime, please double-check your computer configuration!\n");
        }
    }

    //Initial WS connection
    let (mut socket, response) = connect(
        match Url::parse("ws://localhost:6123") {
            Ok(u) => u,
            Err(e) => {
                eprintln!("\nERR: Could not parse registered string to URL, was this mistyped or something?\n{e}\n");
                panic!("\nMOR: Cannot continue runtime, please double-check your computer configuration!\n");
            }
        }
    ).expect("\nERR: Can't connect to GWM WS\n");

    //Successful connection, print debug info.
    println!("Connected to GWM\nResCode - {}", response.status());
    println!("Response Headers:\n");
    for (header, _value) in response.headers() {
        println!("* {}", &header);
    }

    //If we error out attempting to subscribe to GWM, kill the process.
    if let Err(e) = socket.send(Message::Text("subscribe -e window_managed".into())) {
        eprintln!("\nERR: Could not parse raw message data from initial GWM subscription! Raw error:\n{e}\n");
    } else {
        loop {
            let json_msg: Value = match serde_json::from_str(
                socket
                    .read()
                    .expect("ERR: Could not read message!\n")
                    .to_text()
                    .unwrap_or_default(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("\nERR: GWM WS MSG could not be parsed to Value! Raw error:\n{e}\n");
                    continue;
                }
            };

            if let Some((x, y)) = get_window_height_width(&json_msg["data"]["focusedContainer"])
                .or_else(|| get_window_height_width(&json_msg["data"]["managedWindow"]))
            {
                size_tile(&mut socket, x, y).unwrap();
            }
        }
    }
}

fn size_tile(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    x: f64,
    y: f64,
) -> Result<(), tungstenite::Error> {
    if x < y {
        socket.send(Message::Text(String::from(
            "command \"tiling direction vertical\"",
        )))
    } else {
        socket.send(Message::Text(String::from(
            "command \"tiling direction horizontal\"",
        )))
    }
}

fn get_window_height_width(v: &Value) -> Option<(f64, f64)> {
    v["width"]
        .as_f64()
        .and_then(|x| v["height"].as_f64().map(|y| (x, y)))
}
