use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[tokio::main]
async fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:9080").await.unwrap();
    let (txc, _rxc): (Sender<(String, SocketAddr)>, Receiver<(String, SocketAddr)>) = broadcast::channel(10);
    loop {
        let (mut socket, address): (TcpStream, SocketAddr) = listener.accept().await.unwrap();
        let tx: Sender<(String, SocketAddr)> = txc.clone();
        let mut rx: Receiver<(String, SocketAddr)> = tx.subscribe();
        tokio::spawn(async move {
            let (reader, mut writer): (ReadHalf, WriteHalf) = socket.split();
            let mut reader: BufReader<ReadHalf> = BufReader::new(reader);
            let mut line: String = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 || line == "\r\n" {
                            break;
                        }
                        tx.send((line.clone(), address)).unwrap();
                        line.clear();
                    },
                    result = rx.recv() => {
                        let (msg, sender_address): (String, SocketAddr) = result.unwrap();
                        if address != sender_address {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
