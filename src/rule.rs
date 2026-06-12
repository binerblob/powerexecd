use upower_dbus::BatteryState;

pub fn parse_state(s: &str) -> Result<BatteryState, String> {
	match s.to_lowercase().as_str() {
		"charged" => Ok(BatteryState::Charging),
		"discharged" => Ok(BatteryState::Discharging),
		_ => Err(format!("'{}' is not a handled battery state. \"charged\" and \"discharged\" Are the currently handled values", s)),
	}
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
			let state = parse_state(&chunk[0]).expect("Failed to parse battery state");
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
