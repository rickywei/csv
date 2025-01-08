use std::str::from_utf8;

use anyhow::{Error, Result};
use memchr::memchr;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

const COMMA_LEN: usize = 1;
const QUOTE_LEN: usize = 1;

#[derive(Clone)]
struct Position {
    line: usize,
    col: usize,
}

pub struct Reader<R: AsyncRead + std::marker::Unpin> {
    r: BufReader<R>,
    comma: u8,
    num_line: usize,
    offset: usize,
    field_per_record: usize,
    hit_eof: bool,
}

impl<R: AsyncRead + std::marker::Unpin> Reader<R> {
    pub async fn new(r: R) -> Result<Self> {
        Ok(Self {
            r: BufReader::new(r),
            comma: b',',
            num_line: 0,
            offset: 0,
            field_per_record: 0,
            hit_eof: false,
        })
    }

    pub fn with_delimiter(mut self, comma: u8) -> Self {
        // TODO: check invalid delimiter
        self.comma = comma;
        self
    }

    pub async fn records(&mut self) -> Result<Vec<Vec<Vec<u8>>>> {
        let mut records = Vec::new();
        let mut record_buf = Vec::new();
        loop {
            let record = self.read_record(&mut record_buf).await;
            if self.hit_eof {
                break;
            }
            match record {
                Ok(record) => {
                    records.push(record.iter().map(|f| f.to_vec()).collect());
                }
                Err(e) => {
                    if is_eof(&e) {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(records)
    }

    async fn read_record<'a>(&mut self, record_buf: &'a mut Vec<u8>) -> Result<Vec<&'a [u8]>> {
        record_buf.clear();
        let mut field_index = Vec::new();
        let mut field_position = Vec::new();
        let mut line_vec = self.read_line().await?;
        // skip empty line
        while !self.hit_eof && line_vec.len() == length_nl(&line_vec) {
            println!("{:?}", line_vec);
            line_vec = self.read_line().await?;
        }
        let mut line = line_vec.as_slice();
        let mut pos = Position {
            line: self.num_line,
            col: 1,
        };
        'PARSE_FIELD: loop {
            if line.len() == 0 || line[0] != b'"' {
                // No quote field
                let i = memchr(self.comma, &line);
                let field = match i {
                    None => &line[0..line.len() - length_nl(&line)],
                    Some(i) => &line[0..i],
                };
                record_buf.extend_from_slice(field);
                field_index.push(record_buf.len());
                field_position.push(pos.clone());
                if let Some(i) = i {
                    line = &line[i + COMMA_LEN..];
                    pos.col += i + COMMA_LEN;
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
                            let ch = if line.len() > 0 { line[0] } else { b'\0' };
                            if ch == b'"' {
                                record_buf.push(b'"');
                                line = &line[QUOTE_LEN..];
                                pos.col += QUOTE_LEN;
                            } else if ch == self.comma {
                                line = &line[COMMA_LEN..];
                                pos.col += COMMA_LEN;
                                field_index.push(record_buf.len());
                                field_position.push(field_pos.clone());
                                continue 'PARSE_FIELD;
                            } else if length_nl(line) == line.len() {
                                field_index.push(record_buf.len());
                                field_position.push(field_pos.clone());
                                break 'PARSE_FIELD;
                            } else {
                                return Err(anyhow::anyhow!("unexpected character {}", ch));
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
                                    if line.len() > 0 {
                                        pos.line += 1;
                                        pos.col = 1;
                                    }
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
        } else if self.field_per_record != field_index.len() && !self.hit_eof {
            return Err(anyhow::anyhow!(
                "wrong number of fields expect:{} got:{}",
                self.field_per_record,
                field_index.len()
            ));
        }

        let mut ret = Vec::new();
        let mut pre_idx = 0;
        for idx in field_index {
            ret.push(&record_buf[pre_idx..idx]);
            pre_idx = idx;
        }

        Ok(ret)
    }

    async fn read_line(&mut self) -> Result<Vec<u8>> {
        let mut line = Vec::new();
        let res = self.read_slice(&mut line).await;
        match res {
            Err(e) => {
                if is_eof(&e) {
                    if line.len() > 0 && line.last().unwrap() == &b'\r' {
                        line.pop();
                    }
                } else {
                    return Err(e);
                }
            }
            Ok(_) => {}
        };
        let n = line.len();
        self.num_line += 1;
        self.offset += n;
        if n >= 2 && line[n - 2] == b'\r' && line[n - 1] == b'\n' {
            line[n - 2] = b'\n';
            line.truncate(n - 1);
        }
        Ok(line)
    }

    async fn read_slice(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let n = self.r.read_until(b'\n', buf).await;
        match n {
            Err(e) => Err(e.into()),
            Ok(0) => {
                self.hit_eof = true;
                Err(anyhow::anyhow!("EOF"))
            }
            Ok(n) => Ok(n),
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

fn is_eof(e: &Error) -> bool {
    e.to_string() == "EOF"
}
