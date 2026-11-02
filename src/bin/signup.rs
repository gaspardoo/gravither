use gravither::{Life, Signup};
use std::{env, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::var_os("GRAVITHER_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".gravither-data"));

    println!("Signup");
    let mut username = String::new();
    print!("Username: ");
    use std::io::Write;
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;

    let password = rpassword::prompt_password("Password: ")?;
    let confirmation = rpassword::prompt_password("Confirm password: ")?;

    match Signup::new(Life::new(root)).register(username.trim(), &password, &confirmation) {
        Ok(user_id) => println!("Signup completed for {user_id}."),
        Err(error) => {
            eprintln!("Signup failed: {error}");
            std::process::exit(1);
        }
    }

    // Getin, 2FA, metrics and observation are intentionally not started here.
    Ok(())
}
