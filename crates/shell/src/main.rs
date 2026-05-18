use std::{env, fs, path::PathBuf};

use comfy_table::Table;

use nosqlite::{
    Database,
    document::document::{DocId, Document},
};

use nosqlite_shell::{cmd::command::Command, errors::CrateResult};

use rustyline::{DefaultEditor, error::ReadlineError};

fn execute_command(db: &mut Database, command: Command) -> CrateResult<bool> {
    match command {
        // Exit shell
        Command::Exit => {
            println!("Exiting nosqlite...");
            return Ok(false);
        }

        // Clear screen
        Command::Clear => {
            print!("\x1B[2J\x1B[1;1H");
        }

        // COLLECTIONS
        Command::ListCollections => {
            let collections = db.list_collections()?;

            if collections.is_empty() {
                println!("No collections found.");
            } else {
                let mut table = Table::new();

                table.set_header(vec!["Collection", "Documents"]);

                for collection in collections {
                    table.add_row(vec![collection.name, collection.document_count.to_string()]);
                }

                println!("{table}");
            }
        }

        Command::CreateCollection(name) => {
            db.create_collection(&name)?;

            println!("Collection '{}' created.", name);
        }

        Command::DeleteCollection(name) => {
            db.delete_collection(&name)?;
            println!("Collection '{}' deleted.", name);
        }

        // DOCUMENTS
        Command::InsertDocument { collection, json } => {
            let document: Document = serde_json::from_str(&json)?;

            db.insert_one(&collection, document)?;

            println!("Document inserted into '{}'.", collection);
        }

        Command::FindDocuments { collection } => {
            let documents = db.find_all(&collection)?;

            if documents.is_empty() {
                println!("No documents found.");
            } else {
                for document in documents {
                    println!("{}", serde_json::to_string_pretty(&document)?);
                }
            }
        }

        Command::DeleteDocuments { collection, id } => {
            let obj_id = DocId::parse_str(&id)?;
            let _ = db.delete_by_id(&collection[..], obj_id)?;
            println!("document '{}' deleted successfully for '{}'.",id, collection);
        }

        Command::Error(error) => {
            eprintln!("nosqlite: {}", error);
        }
    }

    Ok(true)
}

fn main() -> CrateResult<()> {
    let args: Vec<String> = env::args().collect();

    let mut db_path = PathBuf::from("nosqlite.db");

    // Optional: nosqlite --db custom.db
    if let Some(index) = args.iter().position(|a| a == "--db") {
        if let Some(path) = args.get(index + 1) {
            db_path = PathBuf::from(path);
        }
    }

    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut db = Database::open(&db_path)?;

    // Non-interactive mode: nosqlite --cmd "collections list"
    if let Some(index) = args.iter().position(|a| a == "--cmd") {
        let command_input = args
            .get(index + 1)
            .ok_or_else(|| anyhow::anyhow!("--cmd requires input"))?;

        let command: Command = command_input.as_str().try_into()?;

        execute_command(&mut db, command)?;

        return Ok(());
    }

    println!(
        r"
▄▄  ▄▄  ▄▄▄   ▄▄▄▄  ▄▄▄  ▄▄    ▄▄ ▄▄▄▄▄▄ ▄▄▄▄▄
███▄██ ██▀██ ███▄▄ ██▀██ ██    ██   ██   ██▄▄
██ ▀██ ▀███▀ ▄▄██▀ ▀███▀ ██▄▄▄ ██   ██   ██▄▄▄
▀▀
"
    );

    println!("Welcome to nosqlite!\n");

    let mut rl = DefaultEditor::new()?;

    let history_path = env::temp_dir().join(".nosqlite_history");

    match rl.load_history(&history_path) {
        Ok(_) => {}

        Err(ReadlineError::Io(_)) => {
            fs::File::create(&history_path)?;
        }

        Err(error) => {
            eprintln!("history error: {}", error);
        }
    }

    loop {
        let line = rl.readline("nosqlite > ");

        match line {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                rl.add_history_entry(input)?;

                let command_result: Result<Command, _> = input.try_into();

                match command_result {
                    Ok(command) => match execute_command(&mut db, command) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }

                        Err(error) => {
                            eprintln!("nosqlite error: {:?}", error);
                        }
                    },

                    Err(error) => {
                        eprintln!("parse error: {:?}", error);
                    }
                }
            }

            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("\nExiting nosqlite...");

                break;
            }

            Err(error) => {
                eprintln!("shell error: {:?}", error);
            }
        }
    }

    rl.save_history(&history_path)?;

    Ok(())
}
