#![feature(async_iterator)]

use anyhow::Result;
use memchr::memchr;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    sync::futures,
};
use tokio_stream::Stream;

const DELIMITER_LEN: usize = 1;
const QUOTE_LEN: usize = 1;

#[derive(Clone)]
struct Position {
    line: usize,
    col: usize,
}

pub struct Reader<R: AsyncRead + std::marker::Unpin> {
    r: BufReader<R>,
    delimiter: u8,
    num_line: usize,
    offset: usize,
    field_per_record: usize,
}

impl<R: AsyncRead + std::marker::Unpin> Reader<R> {
    pub async fn new(r: R) -> Result<Self> {
        Ok(Self {
            r: BufReader::new(r),
            delimiter: b',',
            num_line: 0,
            offset: 0,
            field_per_record: 0,
        })
    }

    pub async fn records(&mut self) -> Result<Vec<Vec<u8>>> {
        let mut records = Vec::new();
        loop {
            let record = self.read_record().await;
            match record {
                Ok(record) => {
                    records.push(record);
                }
                Err(e) => {
                    if e.to_string() == "EOF" {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(records)
    }

    async fn read_record(&mut self) -> Result<Vec<u8>> {
        let mut record_buf = Vec::new();
        let mut field_index = Vec::new();
        let mut field_position = Vec::new();
        let mut line_vec = self.read_line().await?;
        let mut line = line_vec.as_slice();
        let mut pos = Position {
            line: self.num_line,
            col: 0,
        };
        'PARSE_FIELD: loop {
            if line.len() == 0 || line[0] != b'"' {
                // No quote field
                let i = memchr(self.delimiter, &line);
                let field = match i {
                    None => &line[0..line.len() - length_nl(&line)],
                    Some(i) => &line[0..i],
                };
                record_buf.extend_from_slice(field);
                if let Some(i) = i {
                    line = &line[i + DELIMITER_LEN..];
                    pos.col += i + DELIMITER_LEN;
                    continue 'PARSE_FIELD;
                }
                break 'PARSE_FIELD;
            } else {
                // Quote field
                let field_pos = Position {
                    line: pos.line,
                    col: pos.col,
                };
                line = &line[QUOTE_LEN..];
                pos.col += QUOTE_LEN;
                loop {
                    let i = memchr(b'"', &line); //next quote
                    match i {
                        Some(i) => {
                            // Hit next quote
                            record_buf.extend_from_slice(&line[0..i]);
                            line = &line[i + QUOTE_LEN..];
                            pos.col += i + QUOTE_LEN;
                            let ch = line[0];
                            if ch == b'"' {
                                record_buf.push(b'"');
                                line = &line[QUOTE_LEN..];
                                pos.col += QUOTE_LEN;
                            } else if ch == self.delimiter {
                                line = &line[DELIMITER_LEN..];
                                pos.col += DELIMITER_LEN;
                                field_index.push(record_buf.len());
                                field_position.push(field_pos.clone());
                                continue 'PARSE_FIELD;
                            } else if length_nl(line) == line.len() {
                                field_index.push(record_buf.len());
                                field_position.push(field_pos.clone());
                                break 'PARSE_FIELD;
                            } else {
                                return Err(anyhow::anyhow!("unexpected character"));
                            }
                        }
                        None => {
                            match line.len() > 0 {
                                true => {
                                    // Hit end of line
                                    record_buf.extend_from_slice(line);
                                    pos.col += line.len();
                                    line_vec = self.read_line().await?;
                                    line = line_vec.as_slice();
                                }
                                false => {
                                    // Abrupt end of file (EOF or error)
                                    field_index.push(record_buf.len());
                                    field_position.push(field_pos.clone());
                                    break 'PARSE_FIELD;
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.field_per_record == 0 {
            self.field_per_record = field_index.len();
        } else if self.field_per_record != field_index.len() {
            return Err(anyhow::anyhow!("wrong number of fields"));
        }
        Ok(record_buf)
    }

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
        let n = self.r.read_until(delimiter, &mut buf).await;
        match n {
            Err(e) => Err(e.into()),
            Ok(0) => Err(anyhow::anyhow!("EOF")),
            Ok(n) => Ok((n, buf)),
        }
    }
}

// impl<R> Stream for Reader<R>
// where
//     R: AsyncRead + std::marker::Unpin,
// {
//     type Item = Result<Vec<u8>>;

//     fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {
//         let this = std::pin::Pin::get_mut(self);
//         let fut = this.read_record();
//         let res = futures::ready!(fut.poll(cx));
//         match res {
//             Ok(record) => {
//                 this.num_line += 1;
//                 std::task::Poll::Ready(Some(Ok(record)))
//             }
//             Err(e) => std::task::Poll::Ready(Some(Err(e))),
//         }
//     }
// }

fn length_nl(b: &[u8]) -> usize {
    if b.len() > 0 && *b.last().unwrap() == b'\n' {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn println_records(records: Vec<Vec<u8>>) {
        for record in records {
            for field in record {
                print!("{}", String::from_utf8(field).unwrap());
            }
        }
    }

    #[tokio::test]
    async fn read_all() {
        let data = "aa,bb,cc\ndd,ee,ff";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        println!("{:?}", records);
    }
}
