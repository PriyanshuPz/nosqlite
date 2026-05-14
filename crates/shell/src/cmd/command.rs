use anyhow::Ok;

use crate::errors::Error;

#[derive(Clone, Debug)]
pub enum Command {
    Exit,
    Clear,
    CreateCollection(String),
    Error(&'static str),
}

fn create_collection_subcommand(args: Vec<&str>) -> Result<Command, Error> {
    let vars: Vec<&str> = vec!["COLLECTION", "collection", "Collection"];

    if args.len() < 3 {
        return Ok(Command::Error("CREATE: provide collection name"));
    }

    if !vars.contains(&args[1]) {
        return Ok(Command::Error("CREATE: Invalid command"));
    }

    let collection_name = args[2].trim().to_lowercase();
    Ok(Command::CreateCollection(collection_name))
}

impl TryFrom<&str> for Command {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let split_value: Vec<&str> = value.split_whitespace().collect();
        match split_value[0] {
            "exit" => Ok(Command::Exit),
            "clear" => Ok(Command::Clear),
            "create" => create_collection_subcommand(split_value),
            "CREATE" => create_collection_subcommand(split_value),
            "Create" => create_collection_subcommand(split_value),
            _ => Ok(Command::Error("Unknown command")),
        }
    }
}
