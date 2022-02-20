//! RESP (REdis Serialization Protocol). 
//! 
//! <https://redis.io/topics/protocol>
//! 
use async_std::io::{prelude::*, Read, Write};
use std::{str, str::FromStr, vec};

///　RESP Data
/// 
///  Strings are treated as bytes with utf-8 encoding.
/// 
#[derive(Debug)]
pub(crate) enum Data {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    Bulk(Vec<u8>),
    NullBulk,
    Array(Vec<Data>),
    NullArray,
}

impl Data {
    /// helper function. return OK RESP string.
    pub(crate) fn ok() -> Data {
        Data::SimpleString(b"OK".to_vec())
    }
    /// helper function. return PONG RESP string.
    pub(crate) fn pong() -> Data {
        Data::SimpleString(b"PONG".to_vec())
    }
    /// helper function. return error RESP string.    
    pub(crate) fn error(msg: &str) -> Data {
        Data::Error(msg.as_bytes().to_vec())
    }
}
/// Encode Decode byte data into Data struct. into byte data.
pub(crate) struct Encoder {
    /// Data to be decoded.
    data: Data,
}

impl Encoder {
    /// Create Encoder instance.
    pub(crate) fn new(data: Data) -> Self {
        Encoder { data }
    }
    /// Encode Data struct into byte data.
    /// 
    /// Array do not contain arrays.
    /// 
    pub(crate) async fn encode<T>(&mut self, stream: &mut T) -> crate::Result<()>
    where
        T: Write + std::marker::Unpin + std::marker::Send,
    {
        match &mut self.data {
            Data::Array(array) => {
                stream.write_all(b"*").await?;
                stream.write_all(&array.len().to_string().as_bytes()).await?;
                stream.write_all(b"\r\n").await?;
                for data in array {
                    Encoder::_encode(stream, data).await?;
                }
            }
            _ => {
                Encoder::_encode(stream, &mut self.data).await?;
            }
        }
        stream.flush().await?;
        Ok(())
    }
    /// Encode Data struct into byte data.
    /// 
    /// internal function.
    async fn _encode<T>(stream: &mut T, data: &mut Data) -> crate::Result<()>
    where
        T: Write + Unpin + std::marker::Send,
    {
        match data {
            Data::SimpleString(simple_string) => {
                stream.write_all(b"+").await?;
                stream.write_all(&simple_string[..]).await?;
                stream.write_all(b"\r\n").await?;
            }
            Data::Error(error) => {
                stream.write_all(b"-ERR ").await?;
                stream.write_all(&error[..]).await?;
                stream.write_all(b"\r\n").await?;
            }
            Data::Integer(integer) => {
                stream.write_all(b":").await?;
                stream.write_all(integer.to_string().as_bytes()).await?;
                stream.write_all(b"\r\n").await?;
            }
            Data::Bulk(bulk) => {
                stream.write_all(b"$").await?;
                stream.write_all(bulk.len().to_string().as_bytes()).await?;
                stream.write_all(b"\r\n").await?;
                stream.write_all(&bulk[..]).await?;
                stream.write_all(b"\r\n").await?;
            }
            Data::NullBulk => {
                stream.write_all(b"$-1\r\n").await?;
            }
            Data::NullArray => {
                stream.write_all(b"*-1\r\n").await?;
            }
            _ => { //ignore.
            }
        }
        return Ok(());
    }
}

/// Decode byte data into Data struct.
pub(crate) struct Decoder {
    /// Read buffer.
    buffer: Vec<u8>,
}

impl Decoder {
    /// Bulk bytes max size.
    const MAX_BULK_BYTE: i64 = 512 * 1000 * 1000;
    /// Array max size.
    const MAX_ARRAY_SIZE: i64 = 1000;
    /// Create Decoder instance.
    pub(crate) fn new() -> Self {
        Decoder {
            buffer: Vec::with_capacity(4 * 1024),
        }
    }
    /// Decode byte data into Data struct.
    pub(crate) async fn decode<T>(&mut self, stream: &mut T) -> crate::Result<Data>
    where
        T: BufReadExt + Unpin + std::marker::Send + std::marker::Sync,
    {
        self.read(stream).await?;

        match self.peek_byte() {
            //Arrays
            b'*' => match self.get_integer() {
                Some(size) => {
                    if Self::MAX_ARRAY_SIZE < size {
                        return Err("Array length is too long".into());
                    } else if size < 1 {
                        return Ok(Data::NullArray);
                    } else {
                        let mut array = Vec::with_capacity(size as usize);
                        for _ in 0..size {
                            self.read(stream).await?;
                            let data = self._decode(stream).await?;
                            array.push(data);
                        }
                        return Ok(Data::Array(array));
                    }
                }
                None => return Err("protocol error 1".into()),
            },
            _ => return self._decode(stream).await,
        }
    }
    /// Decode byte data into Data struct.
    /// intelnal
    async fn _decode<T>(&mut self, stream: &mut T) -> crate::Result<Data>
    where
        T: BufReadExt + Unpin + std::marker::Send + std::marker::Sync,
    {
        match self.peek_byte() {
            //Bulk Strings
            b'$' => match self.get_integer() {
                Some(len) => {
                    if Self::MAX_BULK_BYTE < len {
                        return Err("Bulk length is too long".into());
                    } else if len < 1 {
                        return Ok(Data::NullBulk);
                    } else {
                        let bulk = self.read_bulk(stream, (len + 2) as usize).await?;
                        return Ok(Data::Bulk(bulk));
                    }
                }
                None => return Err("protocol error 2".into()),
            },
            //Integers
            b':' => match self.get_integer() {
                Some(integer) => return Ok(Data::Integer(integer)),
                None => return Err("protocol error 3".into()),
            },
            //Simple Strings
            b'+' => return Ok(Data::SimpleString(self.get_bytes())),
            //Errors
            b'-' => return Ok(Data::Error(self.get_bytes())),
            //Unknown
            _ => return Err("protocol error 4".into()),
        }
    }
    /// Read bytes from the stream until crlf.
    async fn read<T>(&mut self, stream: &mut T) -> crate::Result<()>
    where
        T: BufReadExt + Unpin + std::marker::Send,
    {
        // read_until append bytes to Vec.
        self.buffer.clear();

        match stream.read_until(b'\n', &mut self.buffer).await {
            Ok(0) => Err(Box::new(super::Error::ConnectionClosed)),
            Ok(4..) => {
                self.buffer.pop().unwrap();
                let r = self.buffer.pop().unwrap();
                if r == b'\r' {
                    Ok(())
                } else {
                    Err(Box::new(super::Error::ProtcolError))
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset => {
                    Err(Box::new(super::Error::ConnectionClosed))
                }
                _ => Err(Box::new(e)),
            },
            _ => Err(Box::new(super::Error::ProtcolError)),
        }
    }
    /// Read the specified bytes from the stream.
    async fn read_bulk<T>(&mut self, stream: &mut T, len: usize) -> crate::Result<Vec<u8>>
    where
        T: Read + Unpin + std::marker::Send,
    {
        let mut bulk = vec![0; len];
        match stream.read_exact(&mut bulk).await {
            Ok(()) => {
                let ln = bulk.pop().unwrap();
                let cr = bulk.pop().unwrap();
                if cr == b'\r' && ln == b'\n' {
                    Ok(bulk)
                } else {
                    Err(Box::new(super::Error::ProtcolError))
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset => {
                    Err(Box::new(super::Error::ConnectionClosed))
                }
                _ => Err(Box::new(e)),
            },
        }
    }
    /// get a byte from the buffer.
    fn peek_byte(&mut self) -> u8 {
        *self.buffer.first().unwrap()
    }
    /// get all bytes from the buffer.    
    fn get_bytes(&mut self) -> Vec<u8> {
        self.buffer[1..].to_vec()
    }
    /// get integer from the buffer.
    fn get_integer(&mut self) -> Option<i64> {
        match std::str::from_utf8(&self.buffer[1..]) {
            Ok(integer) => match i64::from_str(integer) {
                Ok(integer) => Some(integer),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    }    
}

/// Parse Data struct
pub(crate) struct Parser {
    /// Data::Array iterator.
    inter: vec::IntoIter<Data>,
}

impl Parser {
    /// create Parser instance.   
    pub(crate) fn new(data: Data) -> Option<Parser> {
        let array = match data {
            Data::Array(array) => {
                if array.is_empty() {
                    return None;
                }
                array
            }
            _ => return None,
        };

        Some(Parser {
            inter: array.into_iter(),
        })
    }
    ///　Parse Data::Array to extract a string.
    pub(crate) fn next_string(&mut self) -> crate::Result<Option<String>> {
        match self.inter.next() {
            Some(Data::Bulk(bulk)) => Ok(Some(std::str::from_utf8(&bulk[..])?.to_uppercase())),
            Some(Data::SimpleString(string)) => {
                Ok(Some(std::str::from_utf8(&string[..])?.to_uppercase()))
            }
            None => Ok(None),
            _ => Err("protocol error 5".into()),
        }
    }
    ///　Parses Data::Array to extract a u64.
    pub(crate) fn next_u64(&mut self) -> crate::Result<Option<u64>> {
        match self.inter.next() {
            Some(Data::Bulk(bulk)) => Ok(Some(std::str::from_utf8(&bulk[..])?.parse()?)),
            Some(Data::SimpleString(string)) => {
                Ok(Some(std::str::from_utf8(&string[..])?.parse()?))
            }
            None => Ok(None),
            _ => Err("protocol error 6".into()),
        }
    }
    /// Parses Data::Array to extract bytes.
    pub(crate) fn next_bytes(&mut self) -> crate::Result<Option<Vec<u8>>> {
        match self.inter.next() {
            Some(Data::Bulk(bulk)) => Ok(Some(bulk)),
            Some(Data::SimpleString(string)) => Ok(Some(string)),
            None => Ok(None),
            _ => Err("protocol error 7".into()),
        }
    }
}
