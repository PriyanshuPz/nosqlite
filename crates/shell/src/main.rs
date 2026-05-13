use nosqlite_shell::{cmd::command::Command, errors::CrateResult};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    task::JoinHandle,
};
pub fn pwd() -> CrateResult<String> {
    let current_dir = std::env::current_dir()?;
    Ok(current_dir.display().to_string())
}
fn spawn_user_input_handler() -> JoinHandle<CrateResult<()>> {
    tokio::spawn(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let mut reader = tokio::io::BufReader::new(stdin).lines();
        let mut stdout = tokio::io::BufWriter::new(stdout);
        stdout.write(b"Welcome to the nosqlite shell!\n").await?;

        stdout.write(b"\nnosqlite > ").await?;
        stdout.flush().await?;

        while let Ok(Some(line)) = reader.next_line().await {
            let command = handle_new_line(&line).await;
            if let Ok(command) = &command {
                match command {
                    _ => {}
                }
            } else {
                eprintln!("Error parsing command: {}", command.err().unwrap());
            }

            stdout.write(b"\nnosqlite > ").await?;
            stdout.flush().await?;
        }
        Ok(())
    })
}

async fn handle_new_line(line: &str) -> CrateResult<Command> {
    let command: Command = line.try_into()?;
    match command.clone() {
        Command::Echo(s) => {
            println!("{}", s);
        }
        _ => {}
    }
    Ok(command)
}

#[tokio::main]
async fn main() {
    let user_input_handler = spawn_user_input_handler().await;

    if let Ok(Err(e)) = user_input_handler {
        eprintln!("Error: {}", e);
    }
}
