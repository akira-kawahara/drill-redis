//! Redis client.
//!
use crate::protocol;
use crate::protocol::resp::{Data, Decoder, Encoder};
use async_std::{
    io::{stdin, stdout, BufReader, BufWriter},
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use std::net::Shutdown;

///
pub async fn run(addr: impl ToSocketAddrs) -> crate::Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let mut lines_from_stdin = BufReader::new(stdin()).lines().fuse();
    let mut writer = BufWriter::new(&stream);
    let mut reader = BufReader::new(&stream);
    let mut decoder = Decoder::new();

    command_pronpt().await?;
    loop {
        match lines_from_stdin.next().await {
            Some(line) => {
                let line = line?;
                let iter = line.split_whitespace();
                let mut array = Vec::new();

                for param in iter {
                    let bulk = Data::Bulk(Vec::from(param.as_bytes()));
                    array.push(bulk);
                }
                let cmd = Data::Array(array);
                let mut encoder = Encoder::new(cmd);
                encoder.encode(&mut writer).await?;
            }
            None => break,
        };
        match decoder.decode(&mut reader).await {
            Ok(data) => {
                display_data(&data)?;
                command_pronpt().await?;
            }
            Err(e) => {
                println!("{:}", e);
                match e.downcast_ref::<protocol::Error>() {
                    Some(decode_err) => match decode_err {
                        protocol::Error::ProtcolError => {
                            break;
                        }
                        protocol::Error::ConnectionClosed => {
                            return Ok(());
                        }
                    },
                    _ => {
                        break;
                    }
                }
            }
        }
    }
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

/// show command prompt
async fn command_pronpt() -> crate::Result<()> {
    print!("> ");
    stdout().flush().await?;
    Ok(())
}

///
fn display_data(data: &Data) -> crate::Result<()> {
    match data {
        Data::SimpleString(simple_string) => {
            println!("{}", std::str::from_utf8(&simple_string[..])?);
        }
        Data::Error(error) => {
            println!("{}", std::str::from_utf8(&error[..])?);
        }
        Data::Integer(integer) => {
            println!("{}", integer);
        }
        Data::Bulk(bulk) => {
            println!("{}", std::str::from_utf8(&bulk[..])?);
        }
        Data::NullBulk | Data::NullArray => {
            println!("(nil)");
        }
        Data::Array(array) => {
            for item in array {
                display_data(item)?;
            }
        }
    }
    Ok(())
}
