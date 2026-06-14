use tokio::{net::UnixListener, process::Command};
use clap::{ArgAction::Append, Parser};
use futures::stream::StreamExt;

pub mod dbus;

use dbus::upower::{DeviceProxy, BatteryState};
use tracing::{Level, info, error};

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

pub struct Rule {
    pub state: BatteryState,
    pub percentage: u8,
    pub command: String
}

impl Rule {
    pub fn collect_from(raw: Vec<String>) -> Vec<Self> {
        let mut commands: Vec<Self> = Vec::new();

        for chunk in raw.chunks_exact(3) {
            // state = chunk[0]
            let state = match chunk[0].to_lowercase().as_str() {
                "charged" => Ok(BatteryState::Charging),
                "discharged" => Ok(BatteryState::Discharging),
                _ => Err(format!("'{}' is not an implemented state", chunk[0]))
            }.unwrap();
            let percentage: u8 = chunk[1].parse().unwrap();
            let command = String::from(&chunk[2]);
            
            commands.push(Rule {
                state,
                percentage,
                command,
            })
        }
        commands
    }
}

async fn battery_loop(
    device: DeviceProxy<'_>,
    rules: Vec<Rule>,
) -> zbus::Result<()> {
    let stream = device.receive_percentage_changed().await;

    tokio::pin!(stream);
 
    while let Some(event) = stream.next().await {
        info!("Event loop triggered!");
        for argument in &rules {
            // Don't execute if not the given battery state
            let current_battery_state = device.state().await;
            if current_battery_state.ok() != Some(argument.state) {
                info!("Current battery state not matched. Not executing...");
                continue;
            }

            // Execute if the given battery percentage
            let current_percentage = event.get().await.unwrap() as u8;
            if current_percentage == argument.percentage {
                info!("On target percentage, executing: \"{}\"", argument.command);
                Command::new("sh")
                    .arg("-c")
                    .arg(&argument.command)
                    .spawn()?
                    .wait()
                    .await?;
                continue;
            }
            info!("Current percentage not matched. Not executing...")
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let args = Args::parse();
    
    let level = if args.verbose {
        Level::INFO
    } else {
        Level::ERROR
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stdout)
        .init();

    let socket_name = "\0powerexecd_single_instance_socket";

    let listener = match UnixListener::bind(socket_name) {
        Ok(listener) => listener,
        Err(_) => {
            error!("powerexecd is already running! Exiting...");
            std::process::exit(0)
        }
    };

    info!("Unix listener: {:?}", listener.local_addr().unwrap());

    let connection = zbus::Connection::system().await?;
    let rules = Rule::collect_from(args.when);
    let device = DeviceProxy::new(&connection, args.device_path).await?;

    battery_loop(device, rules).await?;

    drop(listener);
    
    Ok(())
}
