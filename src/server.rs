pub mod streaming {
    tonic::include_proto!("streaming");
}

use tokio::io::{self, AsyncBufReadExt};
use tonic::{Request, Response, Status, Streaming};
use tonic::transport::Server;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use crate::streaming::streaming_service_server::{StreamingService, StreamingServiceServer};
use crate::streaming::{StreamRequest, StreamResponse};


#[derive(Debug, Default)]
struct StreamingHandler;

#[tonic::async_trait]
impl StreamingService for StreamingHandler {
    type StartStreamStream = ReceiverStream<Result<StreamResponse, Status>>;

    async fn start_stream(&self, request: Request<Streaming<StreamRequest>>) -> Result<Response<Self::StartStreamStream>, Status> {
        let mut incoming_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(4096);

        tokio::spawn(async move {
            while let Ok(Some(message)) = incoming_stream.message().await {
                println!("Received: {:?}", message);
            }
        });

        tokio::spawn(async move {
            let mut reader = io::BufReader::new(io::stdin());
            let mut buffer = String::new();
            loop {
                match reader.read_line(&mut buffer).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let response = StreamResponse {
                            message: buffer.clone(),
                        };
                        println!("Sending: {:?}", response);
                        if let Err(e) = tx.send(Ok(response)).await {
                            eprintln!("Error sending response to client: {:?}", e);
                            break;
                        }
                        buffer.clear();
                    }
                    Err(e) => {
                        eprint!("Error reading from stdin: {:?}", e);
                        break;
                    }
                }
               
                
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = StreamingHandler::default();

    Server::builder()
        .add_service(StreamingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}