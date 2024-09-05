use tokio::net::TcpListener;
use tokio::io::{AsyncRead, AsyncWrite};
use async_std::io;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;



#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "127.0.0.1:2525";
    let listener = TcpListener::bind(addr).await?;

    println!("SMTP server listening on {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let mut state = "initial";
    let mut sender = String::new();
    let mut recipient = String::new();
    let mut message = String::new();

    // Initial greeting
    socket.write_all(b"220 Welcome to my SMTP server\r\n").await?;

    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
        println!("Received: {}", request);

        match state {
            "initial" => {
                if request.starts_with("HELO") || request.starts_with("EHLO") {
                    socket.write_all(b"250 Hello\r\n").await?;
                    state = "ready";
                } else {
                    socket.write_all(b"502 Command not implemented\r\n").await?;
                }
            }
            "ready" => {
                if request.starts_with("MAIL FROM:") {
                    sender = request.trim_start_matches("MAIL FROM:").trim().to_string();
                    socket.write_all(b"250 OK\r\n").await?;
                } else if request.starts_with("RCPT TO:") {
                    recipient = request.trim_start_matches("RCPT TO:").trim().to_string();
                    socket.write_all(b"250 OK\r\n").await?;
                } else if request == "DATA" {
                    socket.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;
                    state = "data";
                } else if request.starts_with("QUIT") {
                    socket.write_all(b"221 Bye\r\n").await?;
                    return Ok(());
                } else {
                    socket.write_all(b"502 Command not implemented\r\n").await?;
                }
            }
            "data" => {
                if request == "." {
                    socket.write_all(b"250 OK\r\n").await?;
                    println!("Received email:");
                    println!("From: {}", sender);
                    println!("To: {}", recipient);
                    println!("Message: {}", message);
                    // Process or store the email here
                    sender.clear();
                    recipient.clear();
                    message.clear();
                    state = "ready";
                } else {
                    message.push_str(&request);
                    message.push('\n');
                }
            }
            _ => {}
        }
    }
    Ok(())
}
