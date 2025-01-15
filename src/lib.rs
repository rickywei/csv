pub mod err;
pub mod reader;
pub mod writer;

use anyhow::Result;

pub trait HeaderCSV {
    fn get_header() -> Vec<String>;
}

pub trait FromCSV: Sized {
    fn from_csv(header: &Vec<String>, record: &Vec<String>) -> Result<Self>;
}

pub trait ToCSV {
    fn to_csv(&self) -> Vec<String>;
}
