use std::{env, fs};

use nosqlite::Database;
use nosqlite_shell::{cmd::command::Command, errors::CrateResult};
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> CrateResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut db_path = &"nosqlite.db".to_string();
    if args.len() > 2 {
        db_path = &args[1];
    }

    let mut rl = DefaultEditor::new()?;
    let temp_dir = env::temp_dir();
    let history_path = &temp_dir.join(".nosqlite_history").to_path_buf();

    let _ = Database::open(db_path);

    println!(
        r"
▄▄  ▄▄  ▄▄▄   ▄▄▄▄  ▄▄▄  ▄▄    ▄▄ ▄▄▄▄▄▄ ▄▄▄▄▄
███▄██ ██▀██ ███▄▄ ██▀██ ██    ██   ██   ██▄▄
██ ▀██ ▀███▀ ▄▄██▀ ▀███▀ ██▄▄▄ ██   ██   ██▄▄▄
▀▀
"
    );
    println!(" Welcome to nosqlite! Type 'exit' to quit.\n");
    match rl.load_history(history_path) {
        Ok(_) => {}
        Err(ReadlineError::Io(_)) => {
            fs::File::create(history_path)?;
        }
        Err(err) => {
            eprintln!("nosqlite: Error loading history: {}", err);
        }
    }

    loop {
        let line = rl.readline("nosqlite > ");
        match line {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                rl.add_history_entry(input)?;

                let command: Command = input
                    .try_into()
                    .unwrap_or(Command::Error("Invalid Command"));

                match command {
                    Command::Clear => {
                        let _ = rl.clear_screen();
                    }
                    Command::Exit => {
                        eprintln!("Exiting nosqlite...");
                        rl.save_history(history_path)?;
                        break;
                    }
                    Command::CreateCollection(name) => {
                        println!("Collection {name} created sucessfully.")
                    }
                    Command::Error(e) => {
                        eprintln!("nosqlite error: {:?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                // Handle Ctrl-C or Ctrl-D gracefully
                println!("\nExiting nosqlite...");
                break;
            }
            Err(e) => {
                eprintln!("nosqlite error: {:?}", e);
            }
        }
    }

    rl.save_history(history_path)?;

    Ok(())
}
