#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]
#![allow(unused_labels)]

use serde_json::Value;
use tray_item::{
	IconSource,
	TrayItem,
};
use tungstenite::{
	connect,
	Message,
};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
	let mut tray: TrayItem = TrayItem::new(
		"GAT - GlazeWM Alternating Tiler",
		IconSource::Resource("main-icon"),
	)?;
	tray.add_label("GAT - GlazeWM Alternating Tiler")?;
	tray.add_menu_item("Quit GAT", || std::process::exit(0))?;

	let (mut socket, _) = connect(match Url::parse("ws://localhost:6123") {
		Ok(u) => u,
		Err(_) => {
			eprintln!("\nERR: Could not parse registered string to URL, was this mistyped?\n");
			panic!(
				"\nMOR: Cannot continue runtime, please double-check your computer configuration!\n"
			);
		}
	})
	.expect("\nERR: Cant' connect to GWM WS!\n");

	'sub_check: loop {
		if let Err(_) = socket.send(Message::Text("sub -e focus_changed".into())) {
			if let Err(_) = socket.send(Message::Text("subscribe -e \"focus_changed\"".into())) {
				if let Err(_) = socket.send(Message::Text("subscribe -e focus_changed".into())) {
					panic!(
						"\nMOR: No known method for subscribing to GWM focus_changed event worked, process suicide!\n"
					);
				} else {
					break 'sub_check;
				}
			} else {
				break 'sub_check;
			}
		} else {
			break 'sub_check;
		}
	}

	'focus_sub: loop {
		let json_msg: Value = match serde_json::from_str(
			socket
				.read()
				.expect("\nERR: Could not read message!\n")
				.to_text()
				.unwrap_or_default(),
		) {
			Ok(v) => v,
			Err(e) => {
				eprintln!("\nERR: GWM WS MSG could not be parsed to Value! Raw error:\n{e}\n");
				continue;
			}
		};

		let (x, y) = if let Some(x) = json_msg["data"]["focusedContainer"]["width"].as_f64() {
			if let Some(y) = json_msg["data"]["focusedContainer"]["height"].as_f64() {
				(x, y)
			} else {
				continue;
			}
		} else {
			continue;
		};

		let tiling_direction: Value = {
			socket.send(Message::Text("query tiling-direction".into()))?;
			serde_json::from_str(
				socket
					.read()
					.expect("\nERR: Could not read message!\n")
					.to_text()
					.unwrap_or_default(),
			)?
		};

		if x < y {
			if let Err(_) = socket.send(Message::Text(
				"command set-tiling-direction vertical".into(),
			)) {
				if let Err(_) = socket.send(Message::Text(
					"command \"tiling direction vertical\"".into(),
				)) {
					if tiling_direction["data"]["tilingDirection"] == "\"horizontal\"" {
						let _ =
							socket.send(Message::Text("command toggle-tiling-direction".into()));
					} else {
						continue;
					}
				} else {
					continue;
				}
			} else {
				continue;
			}
		} else if x > y {
			if let Err(_) = socket.send(Message::Text(
				"command set-tiling-direction horizontal".into(),
			)) {
				if let Err(_) = socket.send(Message::Text(
					"command \"tiling direction horizontal\"".into(),
				)) {
					if tiling_direction == "\"vertical\"" {
						let _ =
							socket.send(Message::Text("command toggle-tiling-direction".into()));
					} else {
						continue;
					}
				} else {
					continue;
				}
			} else {
				continue;
			}
		}
	}
}
