use life::Life;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let life = Life::new(PathBuf::from(".gravither-data").join("users.txt"));

    println!("Life — Entity Recorder");
    print!("Observer user name: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;

    match life.record_observer_user(&name) {
        Ok(()) => println!("Observer user recorded."),
        Err(error) => println!("Could not record observer user: {error}"),
    }

    Ok(())
}
