mod rule;

use rule::Rule;

use tokio::process::Command;
use clap::{ArgAction::Append, Parser};
use futures::stream::StreamExt;
use upower_dbus::DeviceProxy;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	verbose: bool,
	
	#[arg(short, long = "device", default_value = "/org/freedesktop/UPower/devices/battery_BAT0")]
	device_path: String,

	#[arg(short,
		  long,
		  action = Append,
		  num_args = 3,
		  value_names = ["STATE", "PERCENTAGE", "COMMAND"],
		  required = true
	)]
	when: Vec<String>,
}

macro_rules! verbose {
    ($enabled:expr, $($arg:tt)*) => {
        if $enabled {
            eprintln!($($arg)*);
        }
    };
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
	let args = Args::parse();

	let rules = Rule::collect_from(args.when);

	let connection = zbus::Connection::system().await?;
	let battery_path = args.device_path;
	let device = DeviceProxy::new(&connection, battery_path.clone()).await?;
	
	let percentage_stream = device.receive_percentage_changed().await;

	tokio::pin!(percentage_stream);

	'event_loop: while let Some(event) = percentage_stream.next().await {
		verbose!(args.verbose, "Event loop triggered!");
		for argument in &rules {
			// Don't execute if not the given battery state
			let current_battery_state = device.state().await;
			if current_battery_state.ok() != Some(argument.state) {
				if args.verbose { eprintln!("Current battery state not matched. Not executing..."); }
				continue 'event_loop;
			}

			// Execute if the given battery percentage
			let current_percentage = event.get().await.unwrap() as u8;
			if current_percentage == argument.percentage {
				verbose!(args.verbose, "On target percentage, executing: \"{}\"", argument.command);
				Command::new("sh")
					.arg("-c")
					.arg(&argument.command)
					.spawn()?
					.wait()
					.await?;
			} else {
				verbose!(args.verbose, "Current percentage not matched. Not executing...")
			}
		}
	}
    Ok(())
}
