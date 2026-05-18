use crate::errors::Error;

#[derive(Clone, Debug)]
pub enum Command {
    Exit,
    Clear,

    ListCollections,
    CreateCollection(String),
    DeleteCollection(String),

    InsertDocument { collection: String, json: String },

    FindDocuments { collection: String },

    DeleteDocuments { collection: String },

    Error(String),
}

fn collections_subcommand(args: &[&str]) -> Result<Command, Error> {
    if args.len() < 2 {
        return Ok(Command::Error("collections: missing subcommand".into()));
    }

    match args[1] {
        "list" => Ok(Command::ListCollections),

        "create" => {
            if args.len() < 3 {
                return Ok(Command::Error("collections create <name>".into()));
            }

            Ok(Command::CreateCollection(args[2].to_string()))
        }

        "delete" => {
            if args.len() < 3 {
                return Ok(Command::Error("collections delete <name>".into()));
            }

            Ok(Command::DeleteCollection(args[2].to_string()))
        }

        _ => Ok(Command::Error("unknown collections subcommand".into())),
    }
}

fn documents_subcommand(input: &str, args: &[&str]) -> Result<Command, Error> {
    if args.len() < 2 {
        return Ok(Command::Error("documents: missing subcommand".into()));
    }

    match args[1] {
        // documents insert users {...}
        "insert" => {
            if args.len() < 4 {
                return Ok(Command::Error(
                    "documents insert <collection> <json>".into(),
                ));
            }

            let collection = args[2].to_string();
            let json_start = input.find(args[3]).unwrap();

            let json = input[json_start..].to_string();

            Ok(Command::InsertDocument { collection, json })
        }

        // documents find users
        "find" => {
            if args.len() < 3 {
                return Ok(Command::Error("documents find <collection>".into()));
            }

            Ok(Command::FindDocuments {
                collection: args[2].to_string(),
            })
        }

        // documents delete users
        "delete" => {
            if args.len() < 3 {
                return Ok(Command::Error("documents delete <collection>".into()));
            }

            Ok(Command::DeleteDocuments {
                collection: args[2].to_string(),
            })
        }

        _ => Ok(Command::Error("unknown documents subcommand".into())),
    }
}

impl TryFrom<&str> for Command {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Ok(Command::Error("empty command".into()));
        }

        let args: Vec<&str> = trimmed.split_whitespace().collect();

        match args[0] {
            "exit" => Ok(Command::Exit),
            "clear" => Ok(Command::Clear),
            "collections" => collections_subcommand(&args),
            "documents" => documents_subcommand(trimmed, &args),
            _ => Ok(Command::Error("unknown command".into())),
        }
    }
}
