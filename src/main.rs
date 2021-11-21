#![warn(rust_2018_idioms)]

use bytes::{Buf, BytesMut};
use futures::io::Cursor;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter, Error};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn: Client = Client::new(String::from("172.0.0.1:6379")).await.unwrap();
    conn.auth().await?;
    conn.keys(" *").await?;
    conn.set("test", "test").await?;
    Ok(())
}

#[derive(Debug)]
struct Client{
    stream: BufWriter<TcpStream>,

    buffer: BytesMut,
}

impl Client {
    pub async fn new(addr: String) -> Result<Client, String> {
        let mut conn = Client {
            stream: BufWriter::new(TcpStream::connect(addr).await.unwrap()),
            // Default to a 4KB read buffer. For the use case of mini redis,
            // this is fine. However, real applications will want to tune this
            // value to their specific use case. There is a high likelihood that
            // a larger read buffer will work better.
            buffer: BytesMut::with_capacity(4 * 1024),
        };
        Ok(conn)
    }

    async fn auth(&mut self) -> Result<bool, String> {
        match self.write_value("AUTH dyn8DUbpzRQvrCxn").await {
            Ok(_) => {Ok(true)}
            Err(msg) => {
                println!("auth failed {}", msg);
                Err(msg)
            }
        }
    }

    pub async fn keys(&mut self, key_pattern: &str) -> Result<String, String> {
        return self.write_value(&*("KEYS ".to_owned() + key_pattern)).await
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<String, String> {
        return self.write_value(&*("SET ".to_owned() + key + " " + value)).await
    }

    async fn write_value(&mut self, command: &str) -> Result<String, String> {
        // self.stream.write_u8(b'*').await.unwrap();
        self.stream.write_all(command.as_bytes()).await.unwrap();
        self.stream.write_all(b"\r\n").await.unwrap();
        self.stream.flush().await;
        match self.read_value().await {
            Ok(resp) => { println!("resp is {}", resp) }
            Err(err_msg) => { println!("error msg is {}", err_msg) }
        };
        Ok("success".to_string())
    }

    pub async fn read_value(&mut self) -> Result<String, String> {
        loop {
            println!("ready read buffer");
            match self.stream.read_buf(&mut self.buffer).await {
                Ok(size) => {
                    if 0 == size {
                        if self.buffer.is_empty() {

                        }
                        return Err(String::from("error"))
                    }else {
                        let mut buf = Cursor::new(&self.buffer[..]);
                        let len = buf.position() as usize;
                        println!("buf len is {}", len);
                        buf.set_position(0);
                        let line = Client::get_line(&mut buf).unwrap().to_vec();
                        println!("line {}", String::from_utf8(line).unwrap());
                        self.buffer.advance(len);
                        return Ok(String::from("ok"));
                    }
                }
                Err(_) => {println!("read error")}
            }
        }
    }

    fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], String> {
        // Scan the bytes directly
        let start = src.position() as usize;
        // Scan to the second to last byte
        let end = src.get_ref().len() - 1;

        for i in start..end {
            if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
                // We found a line, update the position to be *after* the \n
                src.set_position((i + 2) as u64);

                // Return the line
                return Ok(&src.get_ref()[start..i]);
            }
        }

        Err(String::from("sad"))
    }
}
