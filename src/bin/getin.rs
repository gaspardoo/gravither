use gravither::{Getin, GetinOutcome, Life};
use std::{env, io::{self, Write}, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::var_os("GRAVITHER_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".gravither-data"));
    let getin = Getin::new(Life::new(root));

    println!("Getin — enter Gravither");
    let mut username = String::new();
    print!("Username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    let password = rpassword::prompt_password("Password: ")?;

    match getin.enter(username, &password, None) {
        Ok(GetinOutcome::Authenticated(user)) => {
            println!("Credentials accepted for {user}. Post-entry processes are not implemented yet.");
        }
        Ok(GetinOutcome::SignupRequired) => {
            println!("No account found. Continuing to Signup.");
            let confirmation = rpassword::prompt_password("Confirm password: ")?;
            match getin.enter(username, &password, Some(&confirmation)) {
                Ok(GetinOutcome::Registered(user)) => {
                    println!("Signup completed for {user}. Post-entry processes are not implemented yet.");
                }
                Ok(GetinOutcome::Authenticated(_)) | Ok(GetinOutcome::SignupRequired) => {
                    eprintln!("Unexpected Getin state.");
                    std::process::exit(1);
                }
                Err(error) => {
                    eprintln!("Entry failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        Ok(GetinOutcome::Registered(_)) => {
            eprintln!("Unexpected Getin state.");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("Entry failed: {error}");
            std::process::exit(1);
        }
    }

    // Getin does not start sessions, 2FA, metrics, or observation yet.
    Ok(())
}
