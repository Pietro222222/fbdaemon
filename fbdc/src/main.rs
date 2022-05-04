use dmsg::Messages;
use tokio::net::UnixStream;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Write {
        #[clap(short, long)]
        index: usize,

        red: u8,
        green: u8,
        blue: u8,
    },
    Clear {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let stream = UnixStream::connect("/tmp/fbdaemon").await?;
    match args.command {
        Commands::Write {
            index,
            red,
            green,
            blue,
        } => {
            stream
                .try_write(
                    &Messages::Write((red, green, blue), index)
                        .to_bytes()
                        .unwrap()[..],
                )
                .ok();
        }
        Commands::Clear {} => {
            stream
                .try_write(&Messages::Clear.to_bytes().unwrap()[..])
                .ok();
        }
    }
    Ok(())
}
