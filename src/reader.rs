use crate::{FromCSV, HeaderCSV, err::*};
use anyhow::Result;
use encoding_rs::Encoding;
use memchr::memchr;
use std::str::from_utf8;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

const COMMA_LEN: usize = 1;
const QUOTE_LEN: usize = 1;

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
    skip_header: bool,
    custom_header: Option<Vec<String>>,
    allow_diff_field_num: bool,
    lazy_quote: bool,
    encoding: Option<&'static Encoding>,

    num_line: usize,
    offset: usize,
    field_per_record: usize,
    still_skip_header: bool,
}

impl<R: AsyncRead + std::marker::Unpin> Reader<R> {
    pub fn new(r: R) -> Self {
        Self {
            r: BufReader::new(r),
            comma: b',',
            skip_header: false,
            custom_header: None,
            allow_diff_field_num: false,
            lazy_quote: false,
            encoding: None,

            num_line: 0,
            offset: 0,
            field_per_record: 0,
            still_skip_header: false,
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

    pub fn with_skip_header(mut self, skip_header: bool) -> Self {
        self.skip_header = skip_header;
        self.still_skip_header = skip_header;
        self
    }

    pub fn with_custom_header(mut self, custom_header: Vec<String>) -> Self {
        self.custom_header = Some(custom_header.clone());
        self
    }

    pub fn with_allow_diff_field_num(mut self, allow_diff_field_num: bool) -> Self {
        self.allow_diff_field_num = allow_diff_field_num;
        self
    }

    pub fn with_lazy_quote(mut self, lazy_quote: bool) -> Self {
        self.lazy_quote = lazy_quote;
        self
    }

    pub fn with_encoding(mut self, encoding: &'static Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    pub async fn deserialize<T>(&mut self) -> Result<Vec<T>>
    where
        T: HeaderCSV + FromCSV,
    {
        let string_records = self.string_records().await?;
        let mut ret = Vec::new();
        for record in string_records {
            ret.push(T::from_csv(
                self.custom_header
                    .as_ref()
                    .unwrap_or(T::get_header().as_ref())
                    .as_ref(),
                &record,
            )?);
        }
        Ok(ret)
    }

    pub async fn string_records(&mut self) -> Result<Vec<Vec<String>>> {
        let records = self.bytes_records().await?;
        let mut ret = Vec::new();
        let is_none = self.encoding.is_none();
        let encoding = self.encoding.unwrap_or(encoding_rs::UTF_8);
        for record in records {
            let mut fields = Vec::new();
            for f in record {
                fields.push(if is_none {
                    to_utf8(&f)?
                } else {
                    to_encoding(&f, encoding)?
                });
            }
            ret.push(fields);
        }
        Ok(ret)
    }

    pub async fn bytes_records(&mut self) -> Result<Vec<Vec<Vec<u8>>>> {
        let mut records = Vec::new();
        loop {
            let record = self.read_record().await?;
            if record.is_eof {
                break;
            } else if self.still_skip_header {
                self.still_skip_header = false;
            } else {
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

fn to_utf8(bytes: &[u8]) -> Result<String> {
    Ok(from_utf8(bytes)?.to_string())
}

fn to_encoding(bytes: &[u8], encoding: &'static Encoding) -> Result<String> {
    let (str, _, _) = encoding.decode(bytes);
    Ok(str.to_string())
}
