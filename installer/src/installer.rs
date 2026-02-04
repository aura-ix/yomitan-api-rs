use std::env;
use std::io;
use std::io::Write;

use yomitan_api_installer::Browser;

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// TODO: on windows, make sure window stays open after error
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("yomitan-api-rs installer {}", env!("CARGO_PKG_VERSION"));

    println!("Select browser:");
    for (i, browser) in Browser::VALUES.iter().enumerate() {
        println!("\t{}: {}", i + 1, browser.name());
    }
    println!("");
    println!("Don't see your browser or the installation doesn't appear to work? Open an issue at https://github.com/aura-ix/yomitan-api-rs");
    println!("");
    
    let browser = loop {
        if let Ok(choice) = prompt("Choice (enter corresponding number): ")?.parse::<usize>() && choice >= 1 && choice <= Browser::VALUES.len() {
            break Browser::VALUES[choice - 1];
        } else {
            println!("Invalid input, must be a number 1 - {}.", Browser::VALUES.len());
            continue
        }
    };

    println!("");
    println!("If you are using a development version of yomitan, enter your extension IDs now (space-separated). Otherwise, hit enter to skip this step.");
    println!("");

    let choice = prompt("Additional extension IDs (enter to skip): ")?;
    let extension_ids: Vec<&str> = choice.split_whitespace()
        .chain(std::iter::once(browser.extension_id()))
        .collect();

    browser.install_api(&extension_ids)?;

    let _ = prompt("Installation complete. Hit enter to exit.");
    Ok(())
}