use anyhow::{Error, Result};
use memchr::memchr;
use std::fmt::{Debug, Display};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

const COMMA_LEN: usize = 1;
const QUOTE_LEN: usize = 1;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    ErrInvalidDelim,
    ErrEOF,
    ErrQuote(usize, usize),
    ErrChar(usize, usize, u8),
    ErrFieldNum(usize, usize, usize, usize),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ErrInvalidDelim => write!(f, "Invalid Delimiter"),
            ErrorKind::ErrEOF => write!(f, "EOF"),
            ErrorKind::ErrQuote(line, col) => {
                write!(f, "line:{} col:{} Error Quote", line, col)
            }
            ErrorKind::ErrChar(line, col, ch) => {
                write!(f, "line:{} col:{} Unexpected Character {}", line, col, ch)
            }
            ErrorKind::ErrFieldNum(line, col, expect, got) => {
                write!(
                    f,
                    "line:{} col:{} Wrong Number Of Fields, Expect:{} Got:{}",
                    line, col, expect, got
                )
            }
        }
    }
}

impl std::error::Error for ErrorKind {}

#[derive(Clone)]
struct Position {
    line: usize,
    col: usize,
}

#[derive(Default)]
struct Slice {
    line: Vec<u8>,
    is_eof: bool,
}

struct Record {
    fields: Vec<Vec<u8>>,
    is_eof: bool,
}

pub struct Reader<R: AsyncRead + std::marker::Unpin> {
    r: BufReader<R>,
    comma: u8,
    allow_diff_field_num: bool,
    has_header: bool,
    lazy_quote: bool,

    num_line: usize,
    offset: usize,
    field_per_record: usize,
    remain_header: bool,
}

impl<R: AsyncRead + std::marker::Unpin> Reader<R> {
    pub async fn new(r: R) -> Result<Self> {
        Ok(Self {
            r: BufReader::new(r),
            comma: b',',
            allow_diff_field_num: false,
            has_header: false,
            lazy_quote: false,

            num_line: 0,
            offset: 0,
            field_per_record: 0,
            remain_header: false,
        })
    }

    pub fn with_delimiter(mut self, comma: u8) -> Result<Self> {
        // TODO: check invalid delimiter
        match comma {
            b'\n' | b'\r' | b'"' => Err(ErrorKind::ErrInvalidDelim.into()),
            _ => {
                self.comma = comma;
                Ok(self)
            }
        }
    }

    pub fn with_allow_diff_field_num(mut self, allow_diff_field_num: bool) -> Result<Self> {
        self.allow_diff_field_num = allow_diff_field_num;
        Ok(self)
    }

    pub fn with_has_header(mut self, has_header: bool) -> Result<Self> {
        self.has_header = has_header;
        self.remain_header = has_header;
        Ok(self)
    }

    pub fn with_lazy_quote(mut self, lazy_quote: bool) -> Result<Self> {
        self.lazy_quote = lazy_quote;
        Ok(self)
    }

    pub async fn records(&mut self) -> Result<Vec<Vec<Vec<u8>>>> {
        let mut records = Vec::new();
        loop {
            let record = self.read_record().await?;
            if record.is_eof {
                break;
            } else {
                if self.remain_header {
                    self.remain_header = false;
                    continue;
                }
                records.push(record.fields);
            }
        }
        Ok(records)
    }

    async fn read_record<'a>(&mut self) -> Result<Record> {
        let mut record_buf = Vec::new();
        let mut field_index = Vec::new();
        let mut field_position = Vec::new();
        let mut s = Slice::default();
        // skip empty line
        while !s.is_eof {
            s = self.read_line().await?;
            if s.line.len() == length_nl(&s.line) {
                s.line.clear();
                continue;
            }
            break;
        }
        if s.is_eof {
            return Ok(Record {
                fields: Vec::new(),
                is_eof: true,
            });
        }

        let Slice { line, is_eof } = s;
        let mut line = line.as_slice();
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
                // Check to make sure a quote does not appear in field.
                if !self.lazy_quote {
                    if let Some(j) = memchr(b'"', field) {
                        let col = pos.col + j;
                        return Err(ErrorKind::ErrQuote(self.num_line, col).into());
                    }
                }
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
                    if let Some(i) = i {
                        // Hit next quote
                        record_buf.extend_from_slice(&line[0..i]);
                        line = &line[i + QUOTE_LEN..];
                        pos.col += i + QUOTE_LEN;
                        let ch = if line.len() > 0 { line[0] } else { b'\0' };
                        if ch == b'"' {
                            // `""` sequence (append quote)
                            record_buf.push(b'"');
                            line = &line[QUOTE_LEN..];
                            pos.col += QUOTE_LEN;
                        } else if ch == self.comma {
                            // `",` sequence (end of field)
                            line = &line[COMMA_LEN..];
                            pos.col += COMMA_LEN;
                            field_index.push(record_buf.len());
                            field_position.push(field_pos.clone());
                            continue 'PARSE_FIELD;
                        } else if length_nl(line) == line.len() {
                            // `"\n` sequence (end of line)
                            field_index.push(record_buf.len());
                            field_position.push(field_pos.clone());
                            break 'PARSE_FIELD;
                        } else if self.lazy_quote {
                            // `"` sequence (bare quote)
                            record_buf.push(b'"');
                        } else {
                            // `"*` sequence (invalid non-escaped quote)
                            return Err(
                                ErrorKind::ErrQuote(self.num_line, pos.col - QUOTE_LEN).into()
                            );
                        }
                    } else if line.len() > 0 {
                        // Hit end of line (copy all data so far)
                        record_buf.extend_from_slice(line);
                        pos.col += line.len();
                        s = self.read_line().await?;
                        line = s.line.as_slice();
                        if line.len() > 0 {
                            pos.line += 1;
                            pos.col = 1;
                        }
                        if s.is_eof {
                            s.is_eof = false;
                        }
                    } else {
                        if !self.lazy_quote {
                            return Err(ErrorKind::ErrQuote(pos.line, pos.col).into());
                        }
                        field_index.push(record_buf.len());
                        field_position.push(field_pos);
                        break 'PARSE_FIELD;
                    }
                }
            }
        }

        if self.allow_diff_field_num {
            // do nothing
        } else if self.field_per_record == 0 {
            self.field_per_record = field_index.len();
        } else if self.field_per_record != field_index.len() {
            return Err(ErrorKind::ErrFieldNum(
                self.num_line,
                pos.col,
                self.field_per_record,
                field_index.len(),
            )
            .into());
        }

        let mut record = Record {
            fields: Vec::new(),
            is_eof: is_eof,
        };
        let mut pre_idx = 0;
        for idx in field_index {
            record.fields.push(record_buf[pre_idx..idx].to_vec());
            pre_idx = idx;
        }

        Ok(record)
    }

    async fn read_line(&mut self) -> Result<Slice> {
        let Slice {
            mut line,
            mut is_eof,
        } = self.read_slice().await?;
        let mut n = line.len();
        if n > 0 && is_eof {
            is_eof = false;
            if line[n - 1] == b'\r' {
                line.pop();
                n -= 1;
            }
        }
        self.num_line += 1;
        self.offset += n;
        if n >= 2 && line[n - 2] == b'\r' && line[n - 1] == b'\n' {
            line[n - 2] = b'\n';
            line.pop();
        }
        Ok(Slice { line, is_eof })
    }

    async fn read_slice(&mut self) -> Result<Slice> {
        let mut s = Slice::default();
        loop {
            let n = self.r.read_until(b'\n', &mut s.line).await;
            match n {
                Err(e) => return Err(e.into()),
                Ok(0) => {
                    s.is_eof = true;
                    return Ok(s);
                }
                Ok(_) => {
                    if *s.line.last().unwrap() == b'\n' {
                        return Ok(s);
                    }
                }
            };
        }
    }
}

fn length_nl(b: &[u8]) -> usize {
    if b.len() > 0 && *b.last().unwrap() == b'\n' {
        1
    } else {
        0
    }
}

fn is_eof(e: &Error) -> bool {
    match e.downcast_ref::<ErrorKind>() {
        Some(ErrorKind::ErrEOF) => true,
        _ => false,
    }
}
