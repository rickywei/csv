#[cfg(test)]
mod test {

    use csv::reader::ErrorKind;
    use csv::reader::Reader;
    use encoding_rs::GBK;

    #[allow(unused)]
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

    fn to_string_records_gbk(records: Vec<Vec<Vec<u8>>>) -> Vec<Vec<String>> {
        records
            .iter()
            .map(|record| {
                record
                    .iter()
                    .map(|f| {
                        let (str, _, _) = GBK.decode(f);
                        str.to_string()
                    })
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
    async fn test_crlf() {
        let data = "a,b\r\nc,d\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b"], vec![
            "c", "d"
        ]]);
    }

    #[tokio::test]
    async fn test_bare_cr() {
        let data = "a,b\rc,d\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b\rc", "d"]]);
    }

    #[tokio::test]
    async fn test_rfc4180() {
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
    async fn test_no_eol() {
        let data = "a,b,c";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }

    #[tokio::test]
    async fn test_semicolon() {
        let data = "a;b;c\n";
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_delimiter(b';')
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }

    #[tokio::test]
    async fn test_multiline() {
        let data = r#""two
line","one line","three
line
field""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![
            "two\nline",
            "one line",
            "three\nline\nfield"
        ]]);
    }

    #[tokio::test]
    async fn test_blank_line() {
        let data = "a,b,c\n\nd,e,f\n\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"], vec![
            "d", "e", "f"
        ]]);
    }

    #[tokio::test]
    async fn test_blank_line_field_count() {
        let data = "a,b,c\n\nd,e,f\n\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"], vec![
            "d", "e", "f"
        ]]);
    }

    #[tokio::test]
    async fn test_leading_space() {
        let data = "a,  b,    c";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "  b", "    c"]]);
    }

    #[tokio::test]
    async fn test_lazy_quote() {
        let data = r#"a "word","1"2",a","b"#;
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_lazy_quote(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![
            r#"a "word""#,
            r#"1"2"#,
            r#"a""#,
            "b"
        ],]);
    }

    #[tokio::test]
    async fn test_bare_quote() {
        let data = r#"a "word","1"2",a""#;
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_lazy_quote(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![
            r#"a "word""#,
            r#"1"2"#,
            r#"a""#
        ],]);
    }

    #[tokio::test]
    async fn test_bare_double_quote() {
        let data = r#"a""b,c"#;
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_lazy_quote(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![r#"a""b"#, "c"],]);
    }

    #[tokio::test]
    async fn test_bad_double_quote() {
        let data = r#"a""b,c"#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 2)
        );
    }

    #[tokio::test]
    async fn test_bad_bare_quote() {
        let data = r#"a "word","b""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 3)
        );
    }

    #[tokio::test]
    async fn test_bad_trailing_quote() {
        let data = r#""a word",b""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 11)
        );
    }

    #[tokio::test]
    async fn test_extraneous_quote() {
        let data = r#""a "word","b""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 4)
        );
    }

    #[tokio::test]
    async fn test_bad_field_count() {
        let data = "a,b,c\nd,e";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrFieldNum(2, 3, 3, 2)
        );
    }

    #[tokio::test]
    async fn test_bad_field_count_multiple() {
        let data = "a,b,c\nd,e\nf";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrFieldNum(2, 3, 3, 2)
        );
    }

    #[tokio::test]
    async fn test_field_count() {
        let data = "a,b,c\nd,e\nf";
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_allow_diff_field_num(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["a", "b", "c"],
            vec!["d", "e"],
            vec!["f"]
        ]);
    }

    #[tokio::test]
    async fn test_trailing_comma_eof() {
        let data = "a,b,c,";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c", ""]]);
    }

    #[tokio::test]
    async fn test_trailing_comma_eol() {
        let data = "a,b,c,\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c", ""]]);
    }

    #[tokio::test]
    async fn test_trailing_comma_space_eof() {
        let data = "a,b,c, ";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c", " "]]);
    }

    #[tokio::test]
    async fn test_trailing_comma_space_eol() {
        let data = "a,b,c, \n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c", " "]]);
    }

    #[tokio::test]
    async fn test_trailing_comma_line3() {
        let data = "a,b,c\nd,e,f\ng,hi,";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["a", "b", "c"],
            vec!["d", "e", "f"],
            vec!["g", "hi", ""]
        ]);
    }

    #[tokio::test]
    async fn test_comma_field() {
        let data = r#"x,y,z,w
x,y,z,
x,y,,
x,,,
,,,
"x","y","z","w"
"x","y","z",""
"x","y","",""
"x","","",""
"","","",""
"#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["x", "y", "z", "w"],
            vec!["x", "y", "z", ""],
            vec!["x", "y", "", ""],
            vec!["x", "", "", ""],
            vec!["", "", "", ""],
            vec!["x", "y", "z", "w"],
            vec!["x", "y", "z", ""],
            vec!["x", "y", "", ""],
            vec!["x", "", "", ""],
            vec!["", "", "", ""],
        ]);
    }

    #[tokio::test]
    async fn test_trailing_comma() {
        let data = "a,b,\nc,d,e";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", ""], vec![
            "c", "d", "e"
        ]]);
    }

    #[tokio::test]
    async fn test_start_line1() {
        let data = "a,\"b\nc\"d,e";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(2, 2)
        );
    }

    #[tokio::test]
    async fn test_start_line2() {
        let data = "a,b\n\"d\n\n,e";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(4, 3)
        );
    }

    #[tokio::test]
    async fn test_crlf_in_quoted_field() {
        let data = "A,\"Hello\r\nHi\",B\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![
            "A",
            "Hello\nHi",
            "B"
        ]]);
    }

    #[tokio::test]
    async fn test_trailing_cr() {
        let data = "field1,field2\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field1", "field2"]]);
    }

    #[tokio::test]
    async fn test_quoted_trailing_cr() {
        let data = "\"field\"\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field"]]);
    }

    #[tokio::test]
    async fn test_quoted_trailing_crcr() {
        let data = "\"field\"\r\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 7)
        );
    }

    #[tokio::test]
    async fn test_field_cr() {
        let data = "field\rfield";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field\rfield"]]);
    }

    #[tokio::test]
    async fn test_field_crcr() {
        let data = "field\r\rfield\r\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field\r\rfield\r"]]);
    }

    #[tokio::test]
    async fn test_field_crcrlf() {
        let data = "field\r\r\nfield\r\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field\r"], vec![
            "field\r"
        ]]);
    }

    #[tokio::test]
    async fn test_field_crcrlfcr() {
        let data = "field\r\r\n\rfield\r\r\n\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["field\r"], vec![
            "\rfield\r"
        ]]);
    }

    #[tokio::test]
    async fn test_field_crcrlfcrcr() {
        let data = "field\r\r\n\r\rfield\r\r\n\r\r";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["field\r"],
            vec!["\r\rfield\r"],
            vec!["\r"]
        ]);
    }

    #[tokio::test]
    async fn test_multi_field_crcrlfcrcr() {
        let data = "field1,field2\r\r\n\r\rfield1,field2\r\r\n\r\r,";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![
            vec!["field1", "field2\r"],
            vec!["\r\rfield1", "field2\r"],
            vec!["\r\r", ""]
        ]);
    }

    #[tokio::test]
    async fn test_quoted_field_multi_lf() {
        let data = "\"\n\n\n\n\"";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["\n\n\n\n"],]);
    }

    #[tokio::test]
    async fn test_multi_crlf() {
        let data = "\r\n\r\n\r\n\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), Vec::<Vec<String>>::new());
    }

    #[tokio::test]
    async fn test_quote_with_trailing_crlf() {
        let data = "\"foo\"bar\"\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 5)
        );
    }

    #[tokio::test]
    async fn test_lazy_quote_with_trailing_crlf() {
        let data = "\"foo\"\"bar\"\r\n";
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_lazy_quote(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["foo\"bar"],]);
    }

    #[tokio::test]
    async fn test_double_quote_with_trailing_crlf() {
        let data = "\"foo\"\"bar\"\r\n";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["foo\"bar"],]);
    }

    #[tokio::test]
    async fn test_even_quotes() {
        let data = r#""""""""""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![r#"""""#],]);
    }

    #[tokio::test]
    async fn test_odd_quotes() {
        let data = r#"""""""""#;
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await;
        assert_eq!(records.is_err(), true);
        assert_eq!(
            *records.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrQuote(1, 8)
        );
    }

    #[tokio::test]
    async fn test_bad_comma1() {
        let data = "";
        let rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_delimiter(b'\n');
        assert_eq!(rd.is_err(), true);
        assert_eq!(
            *rd.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrInvalidDelim
        );
    }

    #[tokio::test]
    async fn test_bad_comma2() {
        let data = "";
        let rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_delimiter(b'\r');
        assert_eq!(rd.is_err(), true);
        assert_eq!(
            *rd.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrInvalidDelim
        );
    }

    #[tokio::test]
    async fn test_bad_comma3() {
        let data = "";
        let rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_delimiter(b'\"');
        assert_eq!(rd.is_err(), true);
        assert_eq!(
            *rd.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrInvalidDelim
        );
    }

    #[tokio::test]
    async fn test_header() {
        let data = "h1,h2,h3\na,b,c\n";
        let mut rd = Reader::new(data.as_bytes())
            .await
            .unwrap()
            .with_has_header(true)
            .unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec!["a", "b", "c"]]);
    }

    #[tokio::test]
    async fn test_utf8() {
        let data = "你好，,こんにちは";
        let mut rd = Reader::new(data.as_bytes()).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records(records), vec![vec![
            "你好，",
            "こんにちは"
        ]]);
    }

    #[tokio::test]
    async fn test_gbk() {
        let data = "你好，,こんにちは";
        let (data, _, _) = GBK.encode(data);
        let mut rd = Reader::new(&data[..]).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records_gbk(records), vec![vec![
            "你好，",
            "こんにちは"
        ]]);
    }

    #[tokio::test]
    async fn test_gbk2() {
        let data = "你\r好，,こんにちは\n\"世\n界\",\"再见\r\n\"";
        let (data, _, _) = GBK.encode(data);
        let mut rd = Reader::new(&data[..]).await.unwrap();
        let records = rd.records().await.unwrap();
        assert_eq!(to_string_records_gbk(records), vec![
            vec!["你\r好，", "こんにちは"],
            vec!["世\n界", "再见\n"]
        ]);
    }
}
