use anyhow::Ok;

use crate::errors::Error;

#[derive(Clone, Debug)]
pub enum Command {
    Exit,
    Clear,
    CreateCollection(String),
    ListCollections,
    DeleteCollection(String),
    Error(&'static str),
}

fn collections_subcommand(args: Vec<&str>) -> Result<Command, Error> {
    if args.len() < 2 {
        return Ok(Command::Error("Invalid subcommand call"));
    }

    let cmd = &args[1].to_ascii_lowercase()[..];

    match cmd {
        "list" => {
            return Ok(Command::ListCollections);
        }
        "create" => {
            if args.len() < 3 {
                return Ok(Command::Error("collections: provide collection name"));
            }
            let collection_name = args[2].trim().to_lowercase();
            return Ok(Command::CreateCollection(collection_name));
        }
        "delete" => {
            if args.len() < 3 {
                return Ok(Command::Error("collections: provide collection name"));
            }
            let collection_name = args[2].trim().to_lowercase();
            return Ok(Command::DeleteCollection(collection_name));
        }
        _ => {
            return Ok(Command::Error("collections: subcommand not found."));
        }
    }
}

impl TryFrom<&str> for Command {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let split_value: Vec<&str> = value.split_whitespace().collect();
        let cmd = &split_value[0].to_lowercase()[..];
        match cmd {
            "exit" => Ok(Command::Exit),
            "clear" => Ok(Command::Clear),
            "collections" => collections_subcommand(split_value),
            _ => Ok(Command::Error("Unknown command")),
        }
    }
}
