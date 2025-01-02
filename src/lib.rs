use std::str::Bytes;

use encoding_rs::{Decoder, Encoding};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
};

use anyhow::Result;

pub struct Reader<R: AsyncRead + std::marker::Unpin> {
    r: BufReader<R>,
    decoder: Decoder,
}

impl<R: AsyncRead + std::marker::Unpin> Reader<R> {
    pub async fn new(r: R, decoder: Decoder) -> Result<Self> {
        Ok(Self {
            r: BufReader::new(r),
            decoder: decoder,
        })
    }

    // pub async fn read(&mut self) -> String {
    // }

    async fn read_line(&mut self) -> Result<Vec<u8>> {
        let (n, mut line) = self.read_slice(b'\n').await?;
        if n >= 2 && line[n - 2] == b'\r' && line[n - 1] == b'\n' {
            line[n - 2] = b'\n';
            line.truncate(n - 1);
        }
        Ok(line)
    }

    async fn read_slice(&mut self, delimiter: u8) -> Result<(usize, Vec<u8>)> {
        let mut buf = Vec::new();
        let n = self.r.read_until(delimiter, &mut buf).await?;
        Ok((n, buf))
    }
}
