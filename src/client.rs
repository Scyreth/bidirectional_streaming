use std::io;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;

pub mod streaming {
    tonic::include_proto!("streaming");
}

use crate::streaming::streaming_service_client::StreamingServiceClient;
use crate::streaming::StreamRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = StreamingServiceClient::connect("http://[::1]:50051").await?;

    let (tx, rx) = mpsc::channel(4096);

    let mut incoming_stream = client.start_stream(ReceiverStream::new(rx)).await?.into_inner();
    tokio::spawn(async move {
        while let Ok(message) = incoming_stream.message().await {
            println!("Received: {:?}", message);
        }
    });

    let mut buffer = String::new();

    while let Ok(_) = io::stdin().read_line(&mut buffer) {
        let request = StreamRequest { message: buffer.clone() };
        println!("Sending: {:?}", request);
        tx.send(request).await.unwrap();
        buffer.clear();
    }

    Ok(())
}