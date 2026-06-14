use tokio::process::Command;
use clap::{ArgAction::Append, Parser};
use futures::stream::StreamExt;

pub mod dbus;

use dbus::{
    upower::{DeviceProxy, BatteryState},
    logind::ManagerProxy
};
use tracing::{Level, info};

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

async fn session_loop(manager: ManagerProxy<'_>) -> zbus::Result<()> {
    let session_id =
        std::env::var("XDG_SESSION_ID")
            .expect("No session. Perhaps you might want to run --no-session?");

    let attached_session =
        manager.get_session(session_id).await?;
    
    let stream = manager.receive_session_removed().await?;
 
    tokio::pin!(stream);
    
    while let Some(event) = stream.next().await {
        info!("Session loop triggered!");
        
        let received_session = event.args();

        let received_session_str =
            received_session.unwrap().object_path().clone().to_string();

        let attached_session_str =
            attached_session.to_string();

        info!("Session received: {}", received_session_str);
        info!("Session attached: {}", attached_session_str);

        let received_id: u32 = received_session_str
            [received_session_str.find('_').unwrap() + 1..]
            .parse()
            .expect("Failed to parse received ID");
        
        let attached_id: u32 = attached_session_str
            [received_session_str.find('_').unwrap() + 1..]
            .parse()
            .expect("Failed to parse attached ID");
        
        info!("ID received: {}", received_id);
        info!("ID attached: {}", attached_id);
        
        // Fuck it
        if (received_id + 1) == attached_id {
            info!("Nuking the process...");
            break;
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

    let connection = zbus::Connection::system().await?;

    // Battery loop
    let rules = Rule::collect_from(args.when);
    let device = DeviceProxy::new(&connection, args.device_path).await?;

    // Session loop
    let manager = ManagerProxy::new(&connection).await?;
    
    tokio::select! {
        result = battery_loop(device, rules) => result?,
        result = session_loop(manager) => result?
    }
    
    Ok(())
}
