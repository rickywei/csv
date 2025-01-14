#[cfg(test)]
mod to_csv_macro_test {
    use std::fmt::Display;

    use csv::{CSV, ToCSV};
    use macros::ToCSVMacro;

    #[test]
    #[allow(unused)]
    fn test_none() {
        #[derive(ToCSVMacro)]
        struct Tick {
            symbol: String,
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        assert_eq!(tk.to_csv(), CSV {
            header: Vec::new(),
            record: Vec::new()
        });
    }

    #[test]
    #[allow(unused)]
    fn test_one_field() {
        #[derive(ToCSVMacro)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        assert_eq!(tk.to_csv(), CSV {
            header: vec!["symbol".to_string()],
            record: vec!["ZVZZT".to_string()]
        });
    }

    #[test]
    #[allow(unused)]
    fn test_all_field() {
        #[derive(ToCSVMacro)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            #[csv(field = "price")]
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        assert_eq!(tk.to_csv(), CSV {
            header: vec!["symbol".to_string(), "price".to_string()],
            record: vec!["ZVZZT".to_string(), "1.23".to_string()]
        });
    }

    #[test]
    #[allow(unused)]
    fn test_nested() {
        #[derive(ToCSVMacro)]
        struct Symbol {
            #[csv(field = "security")]
            security_id: String,
            #[csv(field = "exchange")]
            exchange: String,
        }

        #[derive(ToCSVMacro)]
        struct Tick {
            #[csv(flatten)]
            symbol: Symbol,
            #[csv(field = "price")]
            price: f64,
        }

        let tk = Tick {
            symbol: Symbol {
                security_id: "ZVZZT".to_string(),
                exchange: "None".to_string(),
            },
            price: 1.23,
        };
        assert_eq!(tk.to_csv(), CSV {
            header: vec![
                "security".to_string(),
                "exchange".to_string(),
                "price".to_string()
            ],
            record: vec!["ZVZZT".to_string(), "None".to_string(), (1.23).to_string()]
        });
    }

    #[test]
    #[allow(unused)]
    fn test_enum() {
        enum XEnum {
            XI32(i32),
            XString(String),
            X,
        }
        impl Display for XEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    XEnum::XI32(val) => write!(f, "{}", val),
                    XEnum::XString(val) => write!(f, "{}", val),
                    XEnum::X => write!(f, "X"),
                }
            }
        }

        #[derive(ToCSVMacro)]
        struct Tick {
            #[csv(field = "x_enum")]
            e: XEnum,
        }

        let mut tk = Tick { e: XEnum::XI32(1) };
        assert_eq!(tk.to_csv(), CSV {
            header: vec!["x_enum".to_string(),],
            record: vec![1.to_string()]
        });

        tk.e = XEnum::X;
        assert_eq!(tk.to_csv(), CSV {
            header: vec!["x_enum".to_string(),],
            record: vec!["X".to_string()]
        });
    }

    #[test]
    #[allow(unused)]
    fn test_generic() {
        #[derive(ToCSVMacro)]
        struct Tick<T>
        where
            T: Display,
        {
            #[csv(field = "price")]
            price: T,
        }
        let tk = Tick { price: 1.23 };
        assert_eq!(tk.to_csv(), CSV {
            header: vec!["price".to_string()],
            record: vec![(1.23).to_string()]
        });
    }
}
