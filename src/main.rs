#![warn(rust_2018_idioms)]

use std::str;

use bytes::{BufMut, BytesMut};
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use protocol::{gen_req, parse_resp};
use redis_client::{protocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn: Client = Client::new(String::from("127.0.0.1:6379")).await.unwrap();
//    conn.execute(&["AUTH",""]).await?;
    conn.execute(&["KEYS","*"]).await?;
    conn.execute(&["PING"]).await?;
    conn.execute(&["SET","mykey", "myvalue"]).await?;
    conn.execute(&["SET","inc", "1"]).await?;
    conn.execute(&["GET","mykey"]).await?;
    conn.execute(&["GET","inc"]).await?;
    conn.execute(&["HKETS","hr:ess:ess-model"]).await?;
    Ok(())
}

#[derive(Debug)]
struct Client {
    stream: TcpStream
}

impl Client {
    pub async fn new(addr: String) -> Result<Client, String> {
        let conn = Client {
            stream: TcpStream::connect(addr).await.unwrap()
        };
        Ok(conn)
    }

    pub async fn execute(&mut self, args: &[&str]) -> Result<&str, &str> {
        let command = gen_req(args).to_owned();
        let mut buf = [0u8; 1024];
        let mut resp = BytesMut::with_capacity(1024);

        let (mut reader, mut writer) = self.stream.split();
        writer.write(command.as_bytes()).await.unwrap();
        let n = reader.read(&mut buf).await.unwrap();
        resp.put(&buf[0..n]);
        let result = parse_resp(resp);
        println!("type : {:?}", result.replay_type);
        println!("result : {:?}", result.replay_data);
        Ok("ok")
    }
}
