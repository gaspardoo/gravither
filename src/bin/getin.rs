use gravither::{Getin, GetinOutcome, Life};
use std::{env, io, path::PathBuf};
use std::io::Write;

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

    let password = rpassword::prompt_password("Password: ")?;

    match getin.enter(username.trim(), &password, None) {
        Ok(GetinOutcome::Authenticated(user)) => {
            println!("Credentials accepted for {user}. 2FA and post-entry processes are not implemented yet.");
        }
        Ok(GetinOutcome::SignupRequired) => {
            println!("No account found. Continuing to Signup while retaining the entered values for this flow.");
            let confirmation = rpassword::prompt_password("Confirm password: ")?;
            match getin.enter(username.trim(), &password, Some(&confirmation)) {
                Ok(GetinOutcome::Registered(user)) => {
                    println!("Signup completed for {user}. 2FA and post-entry processes are not implemented yet.");
                }
                Ok(_) => {
                    eprintln!("Unexpected Getin state.");
                    std::process::exit(1);
                }
                Err(error) => {
                    eprintln!("Entry failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        Ok(GetinOutcome::Registered(_)) => unreachable!("confirmation is not passed on initial entry"),
        Err(error) => {
            eprintln!("Entry failed: {error}");
            std::process::exit(1);
        }
    }

    // No 2FA, metrics, observation, or long-running user session is started here.
    Ok(())
}
