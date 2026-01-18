
use clap::{Parser, Subcommand};
use tonic::Request;
use std::fs;

pub mod fileserver {
    tonic::include_proto!("fileserver");
}

use fileserver::file_service_client::FileServiceClient;
use fileserver::FileRequest;

#[derive(Parser)]
#[command(name = "fileserver")]
#[command(about = "CLI for gRPC File Server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Upload {
        file: String,
    },
    Download {
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut client = FileServiceClient::connect("http://127.0.0.1:50051").await?;

    match cli.command {
        Commands::Upload { file } => {
            let data = fs::read(&file)?;
            let request = Request::new(FileRequest {
                filename: file,
                data,
            });

            let response = client.upload_file(request).await?;
            println!("{}", response.into_inner().message);
        }

        Commands::Download { file } => {
            let request = Request::new(FileRequest {
                filename: file.clone(),
                data: vec![],
            });

            let response = client.download_file(request).await?;
            fs::write(&file, response.into_inner().data)?;
            println!("File downloaded: {}", file);
        }
    }

    Ok(())
}
	
