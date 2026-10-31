use gravither::Portal;
use std::{env, io::{self, Write}, path::PathBuf};

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::var_os("GRAVITHER_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".gravither-data"));
    let portal = Portal::new(root);

    println!("GRAVITHER // START PORTAL");
    println!("1. Sign in");
    println!("2. Create identity");
    let choice = prompt("> ")?;
    let username = prompt("Username: ")?;
    let password = prompt("Password (minimum 8 characters): ")?;

    let user_id = match choice.as_str() {
        "1" => portal.authenticate(&username, &password)?,
        "2" => portal.register(&username, &password)?,
        _ => { eprintln!("Unknown portal option."); return Ok(()); }
    };

    println!("Identity accepted: {}", user_id.as_str());
    let device_id = prompt("Device ID [device-1]: ")?;
    let device_id = if device_id.is_empty() { "device-1".to_string() } else { device_id };
    let mut session = portal.launch_session(user_id, device_id)?;

    println!("Common space: {}", session.common_space.id);
    println!("Observation service: {:?}", session.observation.state);
    println!("Observer socket: {:?}", session.observation.observer_socket);
    println!("User process remains active for management and activity logging.");
    println!("Commands: status | stop-navigation | logout");

    loop {
        let command = prompt("gravither> ")?;
        match command.as_str() {
            "status" => {
                println!("user process: {:?}", session.user_process.state);
                println!("common space: {:?}", session.common_space.state);
                println!("observation: {:?}", session.observation.state);
                println!("observer socket: {:?}", session.observation.observer_socket);
            }
            "stop-navigation" => {
                session.stop_navigation()?;
                println!("Navigation stopped; user process remains active.");
            }
            "logout" => {
                session.logout()?;
                println!("Session closed. Identity and files remain on disk.");
                break;
            }
            "" => {}
            _ => println!("Unknown command. Use status, stop-navigation or logout."),
        }
    }
    Ok(())
}
