#[cfg(test)]
mod test {

    use csv::reader::Reader;

    fn println_records(records: Vec<Vec<Vec<u8>>>) {
        for record in records {
            println!(
                "{:?}",
                record
                    .iter()
                    .map(|f| String::from_utf8_lossy(f))
                    .collect::<Vec<_>>()
            );
        }
    }

    fn to_string_records(records: Vec<Vec<Vec<u8>>>) -> Vec<Vec<String>> {
        records
            .iter()
            .map(|record| {
                record
                    .iter()
                    .map(|f| String::from_utf8_lossy(f).to_string())
                    .collect()
            })
            .collect()
    }

    #[tokio::test]
    async fn test_simple() {
        let data = "a,b,c\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }

    #[tokio::test]
    async fn test_CRLF() {
        let data = "a,b\r\nc,d\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b"], vec![
            "c", "d"
        ]]);
    }

    #[tokio::test]
    async fn test_bare_CR() {
        let data = "a,b\rc,d\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b\rc", "d"]]);
    }

    #[tokio::test]
    async fn test_RFC4180() {
        let data = r#"#field1,field2,field3
"aaa","bb
b","ccc"
"a,a","b""bb","ccc"
zzz,yyy,xxx
"#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["#field1", "field2", "field3"],
            vec!["aaa", "bb\nb", "ccc"],
            vec!["a,a", "b\"bb", "ccc"],
            vec!["zzz", "yyy", "xxx"]
        ]);
    }

    #[tokio::test]
    async fn test_no_EOL() {
        let data = "a,b,c";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }

    #[tokio::test]
    async fn test_delimiter() {
        let data = "a;b;c\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap().with_delimiter(b';');
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }
   
    #[tokio::test]
    async fn test_multiline() {
        let data = r#""two
line",
"one line",
"three
line
field""#;
println!("{}", data);
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["two\nline", "one line", "three\nline\nfield"]]);
    }
}
