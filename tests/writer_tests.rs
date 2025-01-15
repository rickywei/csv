#[cfg(test)]
mod writer_test {

    use csv::err::ErrorKind;
    use csv::writer::Writer;
    use encoding_rs::GBK;
    use std::str::from_utf8;

    #[tokio::test]
    async fn test_simple() {
        let data = vec![vec!["abc"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "abc\n".as_bytes());
    }

    #[tokio::test]
    async fn test_simple_use_crlf() {
        let data = vec![vec!["abc"]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_use_crlf(true)
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, "abc\r\n".as_bytes());
    }

    #[tokio::test]
    async fn test_quote1() {
        let data = vec![vec![r#""abc""#]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"\"\"abc\"\"\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_quote2() {
        let data = vec![vec!["a\"b"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"a\"\"b\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_quote3() {
        let data = vec![vec!["\"a\"b\""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"\"\"a\"\"b\"\"\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_space() {
        let data = vec![vec![" abc"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, " abc\n".as_bytes());
    }

    #[tokio::test]
    async fn test_field1() {
        let data = vec![vec!["abc,def"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"abc,def\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_field2() {
        let data = vec![vec!["abc", "def"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "abc,def\n".as_bytes());
    }

    #[tokio::test]
    async fn test_multiline1() {
        let data = vec![vec!["abc"], vec!["def"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "abc\ndef\n".as_bytes());
    }

    #[tokio::test]
    async fn test_multiline2() {
        let data = vec![vec!["abc\ndef"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"abc\ndef\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_use_crlf1() {
        let data = vec![vec!["abc\ndef"]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_use_crlf(true)
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, "\"abc\r\ndef\"\r\n".as_bytes());
    }

    #[tokio::test]
    async fn test_use_crlf2() {
        let data = vec![vec!["abc\rdef"]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_use_crlf(true)
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, "\"abcdef\"\r\n".as_bytes());
    }

    #[tokio::test]
    async fn test_no_use_crlf() {
        let data = vec![vec!["abc\rdef"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\"abc\rdef\"\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty1() {
        let data = vec![vec![""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty2() {
        let data = vec![vec!["", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, ",\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty3() {
        let data = vec![vec!["", "", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, ",,\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty4() {
        let data = vec![vec!["", "", "a"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, ",,a\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty5() {
        let data = vec![vec!["", "a", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, ",a,\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty6() {
        let data = vec![vec!["", "a", "a"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, ",a,a\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty7() {
        let data = vec![vec!["a", "", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "a,,\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty8() {
        let data = vec![vec!["a", "", "a"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "a,,a\n".as_bytes());
    }

    #[tokio::test]
    async fn test_empty9() {
        let data = vec![vec!["a", "a", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "a,a,\n".as_bytes());
    }

    #[tokio::test]
    async fn test_full() {
        let data = vec![vec!["a", "a", "a"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(out, "a,a,a\n".as_bytes());
    }

    #[tokio::test]
    async fn test_comma1() {
        let data = vec![vec!["a", "a", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_comma(b'|')
            .unwrap()
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, "a|a|\n".as_bytes());
    }

    #[tokio::test]
    async fn test_comma2() {
        let data = vec![vec![",", ",", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_comma(b'|')
            .unwrap()
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, ",|,|\n".as_bytes());
    }

    #[tokio::test]
    async fn test_invalid_comma() {
        let mut out = Vec::new();
        let wt = Writer::new(&mut out).with_comma(b'"');
        assert_eq!(wt.is_err(), true);
        assert_eq!(
            *wt.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrInvalidDelim
        );
    }

    #[tokio::test]
    async fn test_header() {
        let header = vec!["h1", "h2", "h3"]
            .into_iter()
            .map(String::from)
            .collect();
        let data = vec![vec!["a", "b", "c"]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_custom_header(header)
            .with_write_header(true)
            .write_records(data)
            .await
            .unwrap();
        assert_eq!(out, "h1,h2,h3\na,b,c\n".as_bytes());
    }

    #[tokio::test]
    async fn test_utf8() {
        let data = vec![vec!["你好，", "こんにちは", "💖"]];
        let mut out = Vec::new();
        Writer::new(&mut out).write_records(data).await.unwrap();
        assert_eq!(from_utf8(&out).unwrap(), "你好，,こんにちは,💖\n");
    }

    #[tokio::test]
    async fn test_gbk() {
        let data = vec![vec!["你好，", "こんにちは", ""]];
        let mut out = Vec::new();
        Writer::new(&mut out)
            .with_encoding(GBK)
            .write_records(data)
            .await
            .unwrap();
        let (expect, _, _) = GBK.encode("你好，,こんにちは,\n");
        assert_eq!(out, expect.to_vec());
    }
}
