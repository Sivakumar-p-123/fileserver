use clap::{Parser, Subcommand};
use tonic::Request;
use std::fs;

pub mod fileserver {
    tonic::include_proto!("fileserver");
}

use fileserver::file_service_client::FileServiceClient;
use fileserver::FileRequest;

#[derive(Parser)]
#[command(name = "fileserver-client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Upload {
        file: String,
        username: String,
        password: String,
    },
    Download {
        file: String,
        username: String,
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut client = FileServiceClient::connect("http://127.0.0.1:50051").await?;

    match cli.command {
        Commands::Upload { file, username, password } => {
            let data = fs::read(&file)?;
            let mut request = Request::new(FileRequest {
                filename: file.clone(),
                data,
            });
            request.metadata_mut().insert("username", username.parse()?);
            request.metadata_mut().insert("password", password.parse()?);

            let response = client.upload_file(request).await?;
            println!("{}", response.into_inner().message);
        }
        Commands::Download { file, username, password } => {
            let mut request = Request::new(FileRequest {
                filename: file.clone(),
                data: vec![],
            });
            request.metadata_mut().insert("username", username.parse()?);
            request.metadata_mut().insert("password", password.parse()?);

            let response = client.download_file(request).await?;
            let resp = response.into_inner();
            fs::write(&file, &resp.data)?;
            println!("{}", resp.message);
        }
    }

    Ok(())
}
