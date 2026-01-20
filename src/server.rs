use tonic::{transport::Server, Request, Response, Status};
use std::fs;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::path::Path;

pub mod fileserver {
    tonic::include_proto!("fileserver");
}

use fileserver::file_service_server::{FileService, FileServiceServer};
use fileserver::{FileRequest, FileResponse};

// Store file ownership: filename -> (username, password)
static FILE_OWNERS: Lazy<Mutex<HashMap<String, (String, String)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Default)]
pub struct MyFileServer;

#[tonic::async_trait]
impl FileService for MyFileServer {
    async fn upload_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        // Extract username/password from metadata
        let username = request
            .metadata()
            .get("username")
            .and_then(|v| v.to_str().ok())
            .ok_or(Status::unauthenticated("Username missing"))?
            .to_string();

        let password = request
            .metadata()
            .get("password")
            .and_then(|v| v.to_str().ok())
            .ok_or(Status::unauthenticated("Password missing"))?
            .to_string();

        // Move the inner request out safely
        let req = request.into_inner();
        let filename = req.filename.clone();

        let mut owners = FILE_OWNERS.lock().unwrap();

        // Check if file exists and verify credentials
        if let Some((stored_user, stored_pass)) = owners.get(&filename) {
            if stored_user != &username || stored_pass != &password {
                return Err(Status::permission_denied(
                    "Invalid username or password for existing file",
                ));
            }
        }

        // Save the file
        fs::write(&filename, req.data)
            .map_err(|e| Status::internal(e.to_string()))?;

        // Register/update ownership
        owners.insert(filename.clone(), (username, password));

        Ok(Response::new(FileResponse {
            message: "File uploaded successfully".into(),
            data: vec![],
        }))
    }

    async fn download_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        // Extract username/password from metadata
        let username = request
            .metadata()
            .get("username")
            .and_then(|v| v.to_str().ok())
            .ok_or(Status::unauthenticated("Username missing"))?
            .to_string();

        let password = request
            .metadata()
            .get("password")
            .and_then(|v| v.to_str().ok())
            .ok_or(Status::unauthenticated("Password missing"))?
            .to_string();

        // Move inner request out safely
        let req = request.into_inner();
        let filename = req.filename;

        let owners = FILE_OWNERS.lock().unwrap();

        match owners.get(&filename) {
            Some((u, p)) if u == &username && p == &password => {
                if !Path::new(&filename).exists() {
                    return Err(Status::not_found("File not found"));
                }

                let data = fs::read(&filename)
                    .map_err(|e| Status::internal(e.to_string()))?;

                Ok(Response::new(FileResponse {
                    message: "File downloaded successfully".into(),
                    data,
                }))
            }
            _ => Err(Status::permission_denied("Invalid username or password")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    println!("📡 FileServer running on {}", addr);

    Server::builder()
        .add_service(FileServiceServer::new(MyFileServer::default()))
        .serve(addr)
        .await?;

    Ok(())
}
