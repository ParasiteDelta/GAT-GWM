#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]

use anyhow::anyhow;
use gat_gwm::{
	error_prompt,
	get_ws_socket,
	toggle_tiling_dir,
	value_from_ws,
	version_from_ws,
	window_dimensions_from_value,
	ws_send,
	GlazeContainerData,
	GlazeCurrentTilingData,
	GlazeEventType,
	GlazeMajorVersion,
	GlazeTilingDirection,
};
use tray_item::{
	IconSource,
	TrayItem,
};

//Left in main due to specificity; non-generic.
fn build_tray(tray: &mut TrayItem) -> anyhow::Result<()>
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
		let mut tray_object = match TrayItem::new(
			"GAT - GlazeWM Alternating Tiler",
			IconSource::Resource("main-icon"),
		) {
			Ok(ti) => ti,
			Err(e) => {
				error_prompt(
					"Could Not Init Tray!",
					"Could not initialize the tray! Aborting...",
				);
				return Err(anyhow!("Could not initialize tray!\nRaw error: {e}"));
			}
		};

		match build_tray(&mut tray_object) {
			Ok(()) => tray_object,
			Err(tie) => {
				error_prompt(
					"Could not build tray!",
					format!("For some reason, could not build the tray.\nRaw error: {tie}")
						.as_str(),
				);
				return Err(tie);
			}
		}
	};

	let mut socket = get_ws_socket();
	let mut current_data = GlazeCurrentTilingData {
		rt_version: { GlazeMajorVersion::from(version_from_ws(&mut socket).major) },
		..GlazeCurrentTilingData::default()
	};

	//Explicit match, to further restrict runtime validity to quantified values.
	let focus_runtime_container: GlazeEventType = match current_data.rt_version {
		GlazeMajorVersion::V3 => GlazeEventType::FocusChanged(GlazeContainerData::new()),
		GlazeMajorVersion::V2 => GlazeEventType::FocusChanged(GlazeContainerData::new()),
	};

	//See, this is a good example of why I'm probably just going to stick to the
	// fundamentals for comparators. Using If Let here causes a false warning from
	// the compiler, stating that the code after the exit-on-error call is
	// unreachable when it's not. While I know that this is actually the fault of
	// the design team, since they've yet to even create a stabilized format, much
	// less fully incorporate now-standard features, it doesn't make it any less
	// irritating to deal with.
	match focus_runtime_container.subscribe_from(&mut socket) {
		Err(e) => {
			error_prompt(
				"Could not subscribe via GWM IPC!",
				"For some reason, could not subscribe to Event via GWM IPC",
			);
			return Err(anyhow!(
				"Could not parse GWM Subscription data!\nRaw error: {e}"
			));
		}
		_ => loop {
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

					Some(
						match GlazeTilingDirection::from_str(
							tiling_value["data"]["directionContainer"]["tilingDirection"]
								.to_string(),
						) {
							Ok(td) => td,
							Err(_) => continue,
						},
					)
				}
				GlazeMajorVersion::V2 => None,
			};

			match window_dimensions_from_value(&focus_msg["data"]["focusedContainer"]) {
				Some((x, y)) => toggle_tiling_dir(&mut socket, x, y, &current_data)?,
				None => continue,
			}
		},
	}
}
