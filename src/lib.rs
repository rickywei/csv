pub mod err;
pub mod reader;
pub mod writer;

pub trait FromCSV {
    fn from_record(header: &Vec<&str>, record: &Vec<&str>) -> Self;
}

pub trait ToCSV {
    fn to_header(&self) -> Vec<String>;
    fn to_record(&self) -> Vec<String>;
}
