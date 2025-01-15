use std::fmt::Display;

use crate::{HeaderCSV, ToCSV, err::*};
use anyhow::Result;
use encoding_rs::Encoding;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

pub struct Writer<R> {
    w: BufWriter<R>,
    comma: u8,
    write_header: bool,
    custom_header: Option<Vec<String>>,
    use_crlf: bool,
    encoding: Option<&'static Encoding>,
}

impl<R: AsyncWrite + std::marker::Unpin> Writer<R> {
    pub fn new(w: R) -> Self {
        Writer {
            w: BufWriter::new(w),
            comma: b',',
            write_header: false,
            custom_header: None,
            use_crlf: false,
            encoding: None,
        }
    }

    pub fn with_comma(mut self, comma: u8) -> Result<Self> {
        match comma {
            b'\n' | b'\r' | b'"' => Err(ErrorKind::ErrInvalidDelim.into()),
            _ => {
                self.comma = comma;
                Ok(self)
            }
        }
    }

    pub fn with_write_header(mut self, write_header: bool) -> Self {
        self.write_header = write_header;
        self
    }

    pub fn with_custom_header(mut self, custom_header: Vec<String>) -> Self {
        self.custom_header = Some(custom_header);
        self
    }

    pub fn with_use_crlf(mut self, use_crlf: bool) -> Self {
        self.use_crlf = use_crlf;
        self
    }

    pub fn with_encoding(mut self, encoding: &'static Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    pub async fn serialize<T>(&mut self, records: &Vec<T>) -> Result<()>
    where
        T: HeaderCSV + ToCSV,
    {
        if self.write_header && self.custom_header.is_none() {
            self.custom_header = Some(T::get_header());
        }
        self.write_records(records.iter().map(|v| v.to_csv()).collect())
            .await?;
        Ok(())
    }

    pub async fn write_records<T>(&mut self, records: Vec<Vec<T>>) -> Result<()>
    where
        T: Display,
    {
        if self.write_header && self.custom_header.is_some() {
            self.write_record(self.custom_header.clone().unwrap())
                .await?;
        }
        for record in records {
            let _ = self
                .write_record(record.iter().map(|f| f.to_string()).collect())
                .await?;
        }
        let _ = self.w.flush().await?;
        Ok(())
    }

    async fn write_record(&mut self, record: Vec<String>) -> Result<()> {
        let comma = &[self.comma];
        for (n, field) in record.iter().enumerate() {
            let encoded_field;
            let mut field = match self.encoding {
                None => field.as_bytes(),
                Some(encoding) => {
                    let (ret, _, _) = encoding.encode(field);
                    encoded_field = ret;
                    encoded_field.as_ref()
                }
            };
            if n > 0 {
                let _ = self.w.write(comma).await?;
            }
            if !self.field_needs_quotes(field) {
                let _ = self.w.write(field).await?;
                continue;
            }
            let mut cnt = 0;
            let _ = self.w.write(b"\"").await?;
            while field.len() > 0 {
                cnt += 1;
                if cnt > 5 {
                    break;
                }
                let i = field
                    .iter()
                    .position(|&c| c == b'"' || c == b'\r' || c == b'\n');
                let i = i.unwrap_or(field.len());
                let _ = self.w.write(&field[..i]).await?;
                field = &field[i..];
                if field.len() > 0 {
                    match field[0] {
                        b'"' => {
                            let _ = self.w.write(b"\"\"").await?;
                        }
                        b'\r' => {
                            if !self.use_crlf {
                                let _ = self.w.write(b"\r").await?;
                            }
                        }
                        b'\n' => {
                            let _ = self
                                .w
                                .write(if self.use_crlf { b"\r\n" } else { b"\n" })
                                .await?;
                        }
                        _ => {}
                    };
                    field = &field[1..]
                }
            }
            let _ = self.w.write(b"\"").await?;
        }
        let _ = self
            .w
            .write(if self.use_crlf { b"\r\n" } else { b"\n" })
            .await?;
        Ok(())
    }

    fn field_needs_quotes(&self, field: &[u8]) -> bool {
        for &b in field {
            if b == b'\n' || b == b'\r' || b == b'"' || b == self.comma {
                return true;
            }
        }
        return false;
    }
}
