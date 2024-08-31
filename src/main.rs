#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

use gat_gwm::{
	error_prompt,
	get_ws_socket,
	toggle_tiling_dir,
	value_from_ws,
	version_from_ws,
	window_dimensions_from_value,
	ws_send,
	GlazeCurrentTilingData,
	GlazeMajorVersion,
	GlazeTilingDirection,
};
use tray_item::{
	IconSource,
	TIError,
	TrayItem,
};

//Left in main due to specificity; non-generic.
fn build_tray(tray: &mut TrayItem) -> std::result::Result<(), TIError>
{
	tray.add_label("GAT - GlazeWM Alternating Tiler")?;
	let quit_menu_function = || std::process::exit(0);
	tray.add_menu_item("Quit GAT", quit_menu_function)?;
	Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
	let _tray: TrayItem = {
		let mut tray_object = if let Ok(ti) = TrayItem::new(
			"GAT - GlazeWM Alternating Tiler",
			IconSource::Resource("main-icon"),
		) {
			ti
		} else {
			error_prompt(
				"Could not init tray!",
				"ERR: Could not initialize the tray! Aborting...",
			);
			std::process::exit(333);
		};

		match build_tray(&mut tray_object) {
			Ok(()) => tray_object,
			Err(tie) => {
				error_prompt(
					"Could not build tray!",
					format!("For some reason, could not build the tray.\nRaw error: {tie}")
						.as_str(),
				);
				std::process::exit(333);
			}
		}
	};

	let mut socket = get_ws_socket();
	let mut current_data = GlazeCurrentTilingData {
		rt_version: {
			let vfws = version_from_ws(&mut socket);
			GlazeMajorVersion::from(vfws.major)
		},
		..GlazeCurrentTilingData::default()
	};
	let focus_sub_str: String = match current_data.rt_version {
		GlazeMajorVersion::V3 => String::from("sub -e focus_changed"),
		GlazeMajorVersion::V2 => String::from("subscribe -e focus_changed"),
	};

	if let Err(e) = ws_send(&mut socket, focus_sub_str) {
		eprintln!(
			"\nERR: Could not parse raw message data from initial GWM subscription!
			\nRaw error:\n{e}\n"
		);
	} else {
		loop {
			let focus_msg = {
				let buf = value_from_ws(&mut socket);
				match buf {
					_ if buf == serde_json::Value::default() => continue,
					_ => buf,
				}
			};

			current_data.tiling_dir = match &current_data.rt_version {
				GlazeMajorVersion::V3 => {
					let _ = ws_send(&mut socket, "query tiling-direction".into());
					let tiling_value = value_from_ws(&mut socket);

					Some(GlazeTilingDirection::from_string(
						tiling_value["data"]["directionContainer"]["tilingDirection"].to_string(),
					))
				}
				GlazeMajorVersion::V2 => None,
			};

			if let Some((x, y)) =
				window_dimensions_from_value(&focus_msg["data"]["focusedContainer"])
			{
				toggle_tiling_dir(&mut socket, x, y, &current_data)?;
			} else {
				println!("WRN: Failed to retrieve window dimensions");
			}
		}
	}

	Ok(())
}
