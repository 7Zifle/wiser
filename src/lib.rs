use serde::Deserialize;
use std::fs;
use std::{process::Command, thread, time};

pub fn run_cmd(cmd: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("error processing command");
    let output = String::from_utf8_lossy(&output.stdout).to_string();
    output.trim_end_matches("\n").to_owned()
}

pub fn get_connection(interface: &str) -> String {
    let cmdoutput = run_cmd("nmcli -t -f name,device connection show --active");
    for line in cmdoutput.lines() {
	let split: Vec<&str> = line.split(":").collect();
	if split.len() == 2 {
	    if split[1] == interface {
		return String::from(split[0]);
	    }
	}
    }
    return String::from("unknown");
}

#[derive(Deserialize)]
pub struct Network {
    connection: String,
    commands: Vec<String>,
}

#[derive(Deserialize)]
pub struct NetworkConfig {
    interface: String,
    sleep: u64,
    networks: Vec<Network>,
}

pub fn run() {
    let dir = match dirs::home_dir() {
	Some(path) => format!("{}/.config/wiser/config.ron", path.display()),
	None => String::from("config.ron"),
    };
    let config_file = fs::read_to_string(dir).expect("error finding config");
    let config: NetworkConfig = ron::from_str(&config_file).expect("error parsing config");

    let mut already_ran = Vec::new();

    loop {
        for network in &config.networks {
            if !already_ran.contains(&network.connection)
                && network.connection == get_connection(&config.interface)
            {
                already_ran.push(network.connection.to_owned());
                println!("In new network: {} running commands", &network.connection);
                for cmd in &network.commands {
                    run_cmd(cmd);
                }
            }
        }
        thread::sleep(time::Duration::from_secs(config.sleep));
    }
}

#[cfg(test)]
#[test]
fn test_network() {
    let int = "wlp0s20f3";
    let ssid = get_connection(int);
    assert_ne!(String::from("unknown"), ssid);
}

#[test]
fn load_network_config() {
    let dir = match dirs::home_dir() {
	Some(path) => format!("{}/.config/wiser/config.ron", path.display()),
	None => String::from("config.ron"),
    };
    let config_file = fs::read_to_string(dir).expect("error finding config");
    let config: NetworkConfig = ron::from_str(&config_file).expect("error parsing config");
    

    let network: &Network = &config.networks[0];
    assert_eq!(network.connection, String::from("R3Net-5G"));
    assert_eq!(network.commands[0], "alacritty");
    assert_eq!(config.sleep, 5);
}
