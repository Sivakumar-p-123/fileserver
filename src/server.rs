use tonic::{transport::Server, Request, Response, Status};
use std::fs;
use std::path::Path;

pub mod fileserver {
    tonic::include_proto!("fileserver");
}

use fileserver::file_service_server::{FileService, FileServiceServer};
use fileserver::{FileRequest, FileResponse};

#[derive(Default)]
pub struct MyFileServer;

#[tonic::async_trait]
impl FileService for MyFileServer {
    async fn upload_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        let req = request.into_inner();
        fs::write(&req.filename, req.data)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(FileResponse {
            message: "File uploaded successfully".into(),
            data: vec![],
        }))
    }

    async fn download_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        let filename = request.into_inner().filename;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let service = MyFileServer::default();

    println!("📡 FileServer running on {}", addr);

    Server::builder()
        .add_service(FileServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}




