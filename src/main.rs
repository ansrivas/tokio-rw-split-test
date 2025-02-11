use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    // Create a TCP listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("Server listening on 127.0.0.1:8080");

    // // Create a broadcast channel to send health messages
    let (tx, _rx) = tokio::sync::broadcast::channel(10);

    // Spawn a task to send health messages at regular intervals
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            let health_message = format!("Health check");
            let _ = tx_clone.send(health_message);
        }
    });

    loop {
        // Accept an incoming connection
        let (socket, _) = listener.accept().await.unwrap();
        let tx_clone = tx.clone();
        let mut rx = tx_clone.subscribe();

        // Spawn a task to handle the connection
        tokio::spawn(async move {
            let mut server_timeout = tokio::time::interval(tokio::time::Duration::from_secs(15));
            let (mut reader, mut writer) = socket.into_split();
            let mut buffer = [0; 1024];

            let mut is_communication_complete = false;
            loop {
                tokio::select! {
                    result = reader.read(&mut buffer) => {
                        let payload = result.as_ref().unwrap();
                        if payload == &0 {
                            // Connection was closed
                            break;
                        }

                        let marker = String::from_utf8_lossy(&buffer[..*payload]);
                        if marker == "start" {
                            is_communication_complete = true;
                        }
                        println!("Received: {}", marker);
                        let server_response = "We will start sending health messages".as_bytes();
                         writer.write_all(&server_response).await.unwrap();

                    }
                    // result = server_timeout.tick() => { # Could use the tick directly here
                        result = rx.recv()=> {
                            if is_communication_complete || result.is_ok(){
                                // Start publishing health messages
                                let _ = writer.write_all("healthy".as_bytes()).await;
                            }else{
                                println!("Communication not completed, no health messages will be sent");
                            }

                    }
                }
            }
        });
    }
}
