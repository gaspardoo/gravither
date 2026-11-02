use gravither::Life;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var_os("GRAVITHER_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".gravither-data"));
    let life = Life::new(root);

    println!("Life — Entity Recorder");
    print!("Observer user name: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;

    match life.record_observer_user(&name) {
        Ok(()) => println!("Observer user recorded."),
        Err(error) => {
            eprintln!("Could not record observer user: {error}");
            std::process::exit(1);
        }
    }

    Ok(())
}
