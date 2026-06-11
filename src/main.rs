use tokio::process::Command;
use clap::{ArgAction::Append, Parser};
use futures::stream::StreamExt;
use upower_dbus::{BatteryState, DeviceProxy};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(long = "device", default_value = "/org/freedesktop/UPower/devices/battery_BAT0")]
	device_path: String,
	
	#[arg(long = "when", action = Append, required = true)]
    percentage: Vec<u8>,

	#[arg(required = true)]
	command: Vec<String>
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
	let args = Args::parse();

	let connection = zbus::Connection::system().await?;
	let battery_path = args.device_path;
	let device = DeviceProxy::new(&connection, battery_path.clone()).await?;
	
	let percentage_stream = device.receive_percentage_changed().await;

	tokio::pin!(percentage_stream);

	while let Some(event) = percentage_stream.next().await {
		// Skips the follow blocks if not discharging
		let battery_state = device.state().await;
		if battery_state.ok() != Some(BatteryState::Discharging) {
			continue;
		}

		let current_percentage = event.get().await.unwrap() as u8;
		
		for (arg_percentage, arg_command) in args.percentage.iter().zip(&args.command) {
			if &current_percentage == arg_percentage {
				// shell(arg_command.to_string());
				Command::new("sh")
					.arg("-c")
					.arg(arg_command)
					.spawn()?
					.wait()
					.await?;
			}
		}
	}
    Ok(())
}
