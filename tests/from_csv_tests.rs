#[cfg(test)]
mod from_csv_macro_test {

    use std::{
        fmt::{Debug, Display},
        str::FromStr,
    };

    use csv::{CSV, FromCSV};
    use macros::FromCSVMacro;

    #[test]
    #[allow(unused)]
    fn test_none() {
        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick {
            symbol: String,
            price: f64,
        }
        let tk = Tick::from_csv(&CSV {
            header: Vec::new(),
            record: Vec::new(),
        });
        assert_eq!(tk, Tick::default());
    }

    #[test]
    #[allow(unused)]
    fn test_one_field() {
        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            price: f64,
        }
        let tk = Tick::from_csv(&CSV {
            header: vec!["symbol".to_string()],
            record: vec!["ZVZZT".to_string()],
        });
        assert_eq!(tk, Tick {
            symbol: "ZVZZT".to_string(),
            price: f64::default(),
        });
    }

    #[test]
    #[allow(unused)]
    fn test_all_field() {
        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            #[csv(field = "price")]
            price: f64,
        }
        let tk = Tick::from_csv(&CSV {
            header: vec!["symbol".to_string(), "price".to_string()],
            record: vec!["ZVZZT".to_string(), "1.23".to_string()],
        });
        assert_eq!(tk, Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        });
    }

    #[test]
    #[allow(unused)]
    fn test_nested() {
        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Symbol {
            #[csv(field = "security")]
            security_id: String,
            #[csv(field = "exchange")]
            exchange: String,
        }

        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(flatten)]
            symbol: Symbol,
            #[csv(field = "price")]
            price: f64,
        }

        let tk = Tick::from_csv(&CSV {
            header: vec![
                "security".to_string(),
                "exchange".to_string(),
                "price".to_string(),
            ],
            record: vec!["ZVZZT".to_string(), "None".to_string(), (1.23).to_string()],
        });
        assert_eq!(tk, Tick {
            symbol: Symbol {
                security_id: "ZVZZT".to_string(),
                exchange: "None".to_string(),
            },
            price: 1.23,
        });
    }

    #[test]
    #[allow(unused)]
    fn test_enum() {
        #[derive(Default, PartialEq, Debug)]
        enum XEnum {
            XI32(i32),
            XString(String),
            #[default]
            X,
        }

        impl FromStr for XEnum {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "X" => Ok(XEnum::X),
                    num if num.parse::<i32>().is_ok() => {
                        Ok(XEnum::XI32(num.parse::<i32>().unwrap()))
                    }
                    _ => Ok(XEnum::XString(s.to_string())),
                }
            }
        }

        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "x_enum")]
            e: XEnum,
        }

        let mut tk = Tick::from_csv(&CSV {
            header: vec!["x_enum".to_string()],
            record: vec![1.to_string()],
        });
        assert_eq!(tk, Tick { e: XEnum::XI32(1) });
    }

    #[test]
    #[allow(unused)]
    fn test_generic() {
        #[derive(FromCSVMacro, Default, PartialEq, Debug)]
        struct Tick<T>
        where
            T: FromStr + Default + Debug,
            <T as FromStr>::Err: Debug,
        {
            #[csv(field = "price")]
            price: T,
        }
        let tk = Tick::from_csv(&CSV {
            header: vec!["price".to_string()],
            record: vec![(1.23).to_string()],
        });
        assert_eq!(tk, Tick { price: 1.23 });
    }
}
