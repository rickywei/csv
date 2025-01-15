#[cfg(test)]
mod csv_derive_test {
    use std::fmt::Display;
    use std::str::FromStr;

    use anyhow::Result;
    use csv::{FromCSV, HeaderCSV, ToCSV, err::ErrorKind};
    use macros::{CSVFrom, CSVHeader, CSVTo};

    #[test]
    #[allow(unused)]
    fn test_none() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Tick {
            symbol: String,
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        let tk_from = Tick::from_csv(&Vec::new(), &Vec::new()).unwrap();

        assert_eq!(Tick::get_header(), Vec::<String>::new());
        assert_eq!(tk_from, Tick::default());
        assert_eq!(tk.to_csv(), Vec::<String>::new());
    }

    #[test]
    #[allow(unused)]
    fn test_one_field() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        let header = vec!["symbol".to_string()];
        let record = vec!["ZVZZT".to_string()];
        let tk_from = Tick::from_csv(&header, &record).unwrap();

        assert_eq!(Tick::get_header(), header);
        assert_eq!(tk_from, Tick {
            symbol: "ZVZZT".to_string(),
            price: f64::default(),
        });
        assert_eq!(tk.to_csv(), record);
    }

    #[test]
    #[allow(unused)]
    fn test_one_miss() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "symbol")]
            symbol: String,
            price: f64,
        }
        let tk = Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        let header = vec![];
        let record = vec!["ZVZZT".to_string()];
        let res = Tick::from_csv(&header, &record);
        assert_eq!(res.is_err(), true);
        assert_eq!(
            *res.err().unwrap().downcast_ref::<ErrorKind>().unwrap(),
            ErrorKind::ErrMissField("symbol".to_string())
        );
    }

    #[test]
    #[allow(unused)]
    fn test_all_field() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
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
        let header = vec!["symbol".to_string(), "price".to_string()];
        let record = vec!["ZVZZT".to_string(), (1.23).to_string()];
        let tk_from = Tick::from_csv(&header, &record).unwrap();

        assert_eq!(Tick::get_header(), header);
        assert_eq!(tk_from, Tick {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        });
        assert_eq!(tk.to_csv(), record);
    }

    #[test]
    #[allow(unused)]
    fn test_nested() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Symbol {
            #[csv(field = "security")]
            security_id: String,
            #[csv(field = "exchange")]
            exchange: String,
        }
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
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
        let header = vec![
            "security".to_string(),
            "exchange".to_string(),
            "price".to_string(),
        ];
        let record = vec!["ZVZZT".to_string(), "None".to_string(), (1.23).to_string()];
        let tk_from = Tick::from_csv(&header, &record).unwrap();

        assert_eq!(Tick::get_header(), header);
        assert_eq!(tk_from, Tick {
            symbol: Symbol {
                security_id: "ZVZZT".to_string(),
                exchange: "None".to_string()
            },
            price: 1.23,
        });
        assert_eq!(tk.to_csv(), record);
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
        impl Display for XEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    XEnum::XI32(val) => write!(f, "{}", val),
                    XEnum::XString(val) => write!(f, "{}", val),
                    XEnum::X => write!(f, "X"),
                }
            }
        }
        impl FromStr for XEnum {
            type Err = std::num::ParseIntError;

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                match s {
                    "X" => Ok(XEnum::X),
                    num if s.parse::<i32>().is_ok() => Ok(XEnum::XI32(s.parse().unwrap())),
                    _ => Ok(XEnum::XString(s.to_string())),
                }
            }
        }
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Tick {
            #[csv(field = "x_enum")]
            e: XEnum,
        }
        let tk = Tick {
            e: XEnum::XI32(123),
        };
        let header = vec!["x_enum".to_string()];
        let record = vec![(123).to_string()];
        let tk_from = Tick::from_csv(&header, &record).unwrap();

        assert_eq!(Tick::get_header(), header);
        assert_eq!(tk_from, tk);
        assert_eq!(tk.to_csv(), record);
    }

    #[test]
    #[allow(unused)]
    fn test_generic() {
        #[derive(CSVHeader, CSVFrom, CSVTo, Default, PartialEq, Debug)]
        struct Tick<T>
        where
            T: Display + Default + FromStr,
            <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
        {
            #[csv(field = "symbol")]
            symbol: T,
            price: f64,
        }
        let tk = Tick::<String> {
            symbol: "ZVZZT".to_string(),
            price: 1.23,
        };
        let header = vec!["symbol".to_string()];
        let record = vec!["ZVZZT".to_string()];
        let tk_from = Tick::from_csv(&header, &record).unwrap();

        assert_eq!(Tick::<String>::get_header(), header);
        assert_eq!(tk_from, Tick {
            symbol: "ZVZZT".to_string(),
            price: f64::default(),
        });
        assert_eq!(tk.to_csv(), record);
    }
}
