#[cfg(test)]
mod to_csv_macro_test {
    use csv::ToCSV;
    use macros::ToCSVMacro;
    use std::fmt::Display;

    // #[test]
    // fn test_header1() {
    //     #[derive(ToCSVMacro)]
    //     #[allow(unused)]
    //     struct Tick {
    //         symbol: String,
    //         price: f64,
    //     }
    //     let tk = Tick {
    //         symbol: "ZVZZT".to_string(),
    //         price: 1.23,
    //     };
    //     assert_eq!(tk.to_header(), Vec::<String>::new());
    // }

    // #[test]
    // fn test_header2() {
    //     #[allow(unused)]
    //     #[derive(ToCSVMacro)]
    //     struct Tick {
    //         #[csv(field = "symbol")]
    //         symbol: String,
    //         price: f64,
    //     }
    //     let tk = Tick {
    //         symbol: "ZVZZT".to_string(),
    //         price: 1.23,
    //     };
    //     assert_eq!(tk.to_header(), vec!["symbol".to_string()]);
    // }

    #[test]
    fn test_header3() {
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
        assert_eq!(tk.to_header(), vec!["security", "exchange", "price"]);
    }
}
