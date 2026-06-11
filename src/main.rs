use std::process::Command;
use clap::Parser;
use futures::stream::StreamExt;
use upower_dbus::DeviceProxy;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(long = "wait", num_args = 2, action = clap::ArgAction::Append)]
    raw_battery_commands: Vec<String>,
}

fn shell(cmd_str: String) {
	Command::new("sh")
		.arg("-c")
		.arg(cmd_str)
		.output()
		.unwrap();
}

struct BatteryInstance {
	connection: zbus::Connection,
	battery_path: String,
	device: DeviceProxy<'static>
}

impl BatteryInstance {
	async fn new() -> Result<Self, zbus::Error> {
		let connection = zbus::Connection::system().await?;
		let battery_path = String::from("/org/freedesktop/UPower/devices/battery_BAT0");
		let device = DeviceProxy::new(&connection, battery_path.clone()).await?;

		Ok(Self {
			connection,
			battery_path,
			device
		})
	}
}

struct BatteryCommand {
	percentage: u8,
	command: String,
}

struct BatteryCommands {
	list: Vec<BatteryCommand>
}

impl BatteryCommands {
	fn from_args(raw_commands: Vec<String>) -> Self {
		let mut list = Vec::new();
		for chunk in raw_commands.chunks_exact(2) {
			let current_percentage = chunk[0]
				.parse::<u8>()
				.expect("count must be a number");
			let trigger_command = chunk[1].clone();

			list.push(BatteryCommand {
				percentage: current_percentage,
				command: trigger_command
			});
		}
		Self { list }
	}
}

fn main() -> zbus::Result<()> {
    futures::executor::block_on(async move {
		let args = Args::parse();

		let battery_instance = BatteryInstance::new().await?;
		
		let mut percentage_stream =
			battery_instance.device.receive_percentage_changed().await;

		let battery_commands =
			BatteryCommands::from_args(args.raw_battery_commands);

		while let Some(event) = percentage_stream.next().await {
			println!("battery percentage event received:\n");
			let current_percentage = event.get().await.unwrap() as u8;

			for entry in &battery_commands.list {
				println!("parsed percentage: {}\ncurrent percentage: {}\n",
						 entry.percentage,
						 current_percentage
				);
				if current_percentage == entry.percentage {
					shell(String::from(&entry.command));
					println!("running command: \"{}\"\n", entry.command);
				} else {
					println!("not running command: \"{}\"\n", entry.command);
				};
			}
		}
        Ok(())
    })
}
