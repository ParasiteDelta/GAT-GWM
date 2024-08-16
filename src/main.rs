#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

use std::net::TcpStream;

use semver::Version;
use serde_json::Value;
use tray_item::{IconSource, TIError, TrayItem};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

enum GlazeMajor
{
	V2,
	V3(String),
}

fn build_tray(tray: &mut TrayItem) -> Result<(), TIError>
{
	tray.add_label("GAT - GlazeWM Alternating Tiler")?;
	let quit_menu_function = || std::process::exit(0);
	tray.add_menu_item("Quit GAT", quit_menu_function)?;
	Ok(())
}

fn get_value(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Value
{
	match serde_json::from_str(
		socket
			.read()
			.expect("ERR: Could not read message!\n")
			.to_text()
			.unwrap_or_default(),
	) {
		Ok(v) => v,
		Err(e) => {
			eprintln!(
				"\nERR: GWM WS MSG could not be parsed to Value!\nRaw \
				 error:\n{e}\n"
			);
			Value::default()
		}
	}
}

fn get_version(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Version
{
	let _query = socket.send(Message::Text("query app-metadata".into()));
	let reading = {
		let result = get_value(socket);

		if result == Value::default() {
			panic!("ERR: Could not retrieve version info! Aborting...");
		} else {
			result
		}
	};

	if let Ok(v) = Version::parse(reading["data"]["version"].as_str().unwrap_or_default()) {
		v
	} else {
		Version::new(0, 0, 0)
	}
}

fn manage_tiling_dir(
	socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
	x: f64,
	y: f64,
	gwm_version: GlazeMajor,
) -> Result<(), tungstenite::Error>
{
	match gwm_version {
		GlazeMajor::V2 => {
			if x < y {
				socket.send(Message::Text(
					"command \"tiling direction vertical\"".into(),
				))
			} else {
				socket.send(Message::Text(
					"command \"tiling direction horizontal\"".into(),
				))
			}
		}
		GlazeMajor::V3(m) => {
			if (x < y && m != "\"vertical\"") || (x > y && m != "\"horizontal\"") {
				socket.send(Message::Text("command toggle-tiling-direction".into()))
			} else {
				Ok(())
			}
		}
	}
}

fn get_window_dimensions(v: &Value) -> Option<(f64, f64)>
{
	v["width"]
		.as_f64()
		.and_then(|x| v["height"].as_f64().map(|y| (x, y)))
}

#[tokio::main]
async fn main()
{
	let _tray: TrayItem = {
		let mut tray_object = if let Ok(ti) = TrayItem::new(
			"GAT - GlazeWM Alternating Tiler",
			IconSource::Resource("main-icon"),
		) {
			ti
		} else {
			panic!("\nERR: Could not initialize tray!\nAborting...\n");
		};

		match build_tray(&mut tray_object) {
			Ok(()) => tray_object,
			Err(tie) => panic!("\nERR: Could not build tray!\nRaw Error: {tie}\nAborting...\n"),
		}
	};

	let (mut socket, _) = connect(if let Ok(url) = Url::parse("ws://localhost:6123") {
		url
	} else {
		eprintln!(
			"\nERR: Could not parse internal string to URL, was this \
				 mistyped or something?\n"
		);
		panic!(
			"\nMOR: Cannot continue runtime, please double-check your \
				 computer configuration!\n"
		);
	})
	.expect("\nERR: Can't connect to GWM WS\n");

	let version: Version = get_version(&mut socket);
	let subscription: String = match version.major {
		3u64 => String::from("sub -e focus_changed"),
		_ => String::from("subscribe -e focus_changed"),
	};

	if let Err(e) = socket.send(Message::Text(subscription)) {
		eprintln!(
			"\nERR: Could not parse raw message data from initial GWM \
			 subscription! Raw error:\n{e}\n"
		);
	} else {
		loop {
			let focus_msg = {
				let result = get_value(&mut socket);
				if result == Value::default() {
					continue;
				} else {
					result
				}
			};

			let version_data: GlazeMajor = {
				if version.major == 3u64 {
					let _ = socket.send(Message::Text("query tiling-direction".into()));
					let tv = get_value(&mut socket);
					GlazeMajor::V3(tv["data"]["directionContainer"]["tilingDirection"].to_string())
				} else {
					GlazeMajor::V2
				}
			};

			if let Some((x, y)) = get_window_dimensions(&focus_msg["data"]["focusedContainer"]) {
				manage_tiling_dir(&mut socket, x, y, version_data).unwrap();
			}
		}
	}
}
