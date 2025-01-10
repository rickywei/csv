use std::fmt::Display;

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