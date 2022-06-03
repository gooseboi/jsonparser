pub mod parser;
pub mod tokenizer;

pub use parser::JsonVal;
pub use parser::Number;

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused_macros)]
    macro_rules! map {
        ($({$k:expr,$v:expr}),*) => {{
            use parser::MapType;
            let mut map = MapType::new();
            $(map.insert($k, $v);)*
            map
        }};
    }

    #[allow(unused_macros)]
    macro_rules! json_obj {
        (map $map:expr) => {
            JsonVal::Object($map)
        };
        ($({$k:expr,$v:expr}),*) => {{
            use parser::MapType;
            let mut map = MapType::new();
            $(map.insert($k.to_string(), $v);)*
            json_obj!(map map)
        }};
    }

    #[allow(unused_macros)]
    macro_rules! json_str {
        ($s:literal) => {
            JsonVal::String($s.to_string())
        };
        ($s:expr) => {
            JsonVal::String($s)
        };
    }

    #[allow(unused_macros)]
    macro_rules! json_num {
        ($num:expr; uint) => {
            JsonVal::Number(Number::UnsignedInt($num))
        };
        ($num:expr; int) => {
            JsonVal::Number(Number::SignedInt($num))
        };
        ($num:expr; float) => {
            JsonVal::Number(Number::Float($num))
        };
    }

    #[allow(unused_macros)]
    macro_rules! json_arr {
        ($arr:expr) => {
            JsonVal::Array($arr)
        };
    }

    #[test]
    fn wikipedia() {
        // Example taken from https://wikipedia.org/wiki/JSON
        let input = include_str!("../tests/wikipedia.json");
        let tokenizer = tokenizer::Tokenizer::from_str(&input);
        let parsed = parser::parse(tokenizer).expect("Expected valid json");
        if let JsonVal::Object(ref parsed) = parsed {
            assert_eq!(parsed["firstName"], json_str!("John"));
            assert_eq!(parsed["lastName"], json_str!("Smith"));
            assert_eq!(parsed["isAlive"], JsonVal::Boolean(true));
            assert_eq!(parsed["age"], json_num!(27; uint));
            assert_eq!(
                parsed["address"],
                json_obj!({"postalCode", json_str!("10021-3100")},
                               {"state", json_str!("NY")},
                               {"streetAddress",json_str!("21 2nd Street")},
                               {"city", json_str!("New York")})
            );
            assert_eq!(
                parsed["phoneNumbers"],
                json_arr!(vec![
                    json_obj!({"type", json_str!("home")},
                              {"number", json_str!("212 555-1234")}),
                    json_obj!({"type", json_str!("office")},
                              {"number", json_str!("646 555-4567")}),
                ])
            );
            assert_eq!(parsed["children"], JsonVal::Array(vec![]));
            assert_eq!(parsed["spouse"], JsonVal::Null);
        } else {
            unreachable!("Must parse as an object, {:#?}", parsed)
        }
    }

    #[test]
    fn jsonplaceholder() {
        // Example taken from https://jsonplaceholder.typicode.com/todos/?userId=1
        let input = include_str!("../tests/jsonplaceholder.json");
        let tokenizer = tokenizer::Tokenizer::from_str(&input);
        let parsed = parser::parse(tokenizer).expect("Expected valid json");
        if let JsonVal::Array(ref parsed) = parsed {
            for val in parsed {
                if let JsonVal::Object(val) = val {
                    assert!(matches!(
                        val["userId"],
                        JsonVal::Number(Number::UnsignedInt(_))
                    ));
                    assert!(matches!(val["id"], JsonVal::Number(Number::UnsignedInt(_))));
                    assert!(matches!(val["title"], JsonVal::String(_)));
                    assert!(matches!(val["completed"], JsonVal::Boolean(_)));
                } else {
                    unreachable!("Must parse as an object, {:#?}", parsed)
                }
            }
        } else {
            unreachable!("Must parse as an array, {:#?}", parsed)
        }
    }

    #[test]
    fn jsonorg() {
        // Example taken from https://www.json.org/example.html
        let input = include_str!("../tests/jsonorg.json");
        let tokenizer = tokenizer::Tokenizer::from_str(&input);
        let parsed = parser::parse(tokenizer).expect("Expected valid json");
        if let JsonVal::Object(parsed) = parsed {
            if let JsonVal::Object(ref widget) = parsed["widget"] {
                assert_eq!(widget["debug"], json_str!("on"));
                if let JsonVal::Object(ref window) = widget["window"] {
                    assert_eq!(window["title"], json_str!("Sample Konfabulator Widget"));
                    assert_eq!(window["name"], json_str!("main_window"));
                    assert_eq!(window["width"], json_num!(500; uint));
                    assert_eq!(window["height"], json_num!(500; uint));
                    if let JsonVal::Object(ref image) = widget["image"] {
                        assert_eq!(image["src"], json_str!("Images/Sun.png"));
                        assert_eq!(image["name"], json_str!("sun1"));
                        assert_eq!(image["hOffset"], json_num!(250; uint));
                        assert_eq!(image["vOffset"], json_num!(250; uint));
                        assert_eq!(image["alignment"], json_str!("center"));
                    } else {
                        unreachable!("Must parse as an object, {:#?}", parsed)
                    }
                    if let JsonVal::Object(ref text) = widget["text"] {
                        assert_eq!(text["data"], json_str!("Click Here"));
                        assert_eq!(text["size"], json_num!(36; uint));
                        assert_eq!(text["style"], json_str!("bold"));
                        assert_eq!(text["name"], json_str!("text1"));
                        assert_eq!(text["hOffset"], json_num!(250; uint));
                        assert_eq!(text["vOffset"], json_num!(100; uint));
                        assert_eq!(text["alignment"], json_str!("center"));
                        assert_eq!(
                            text["onMouseUp"],
                            json_str!("sun1.opacity = (sun1.opacity / 100) * 90;")
                        );
                    } else {
                        unreachable!("Must parse as an object, {:#?}", parsed)
                    }
                }
            } else {
                unreachable!("Must parse as an object, {:#?}", parsed)
            }
        } else {
            unreachable!("Must parse as an object, {:#?}", parsed)
        }
    }

    #[test]
    fn tsoding() {
        // Example taken from https://github.com/tsoding/haskell-json
        let input = r#"{
                "hello": [false,true,null,42,"foo\n\u1234\"", [1,-2,3.1415, 4e-6, 5E6, 0.123e+1]],
                "world": null
                }
            "#;
        let tokenizer = tokenizer::Tokenizer::from_str(&input);
        let parsed = parser::parse(tokenizer).expect("Expected valid json");
        if let JsonVal::Object(ref parsed) = parsed {
            if let JsonVal::Array(ref hello) = parsed["hello"] {
                assert_eq!(hello[0], JsonVal::Boolean(false));
                assert_eq!(hello[1], JsonVal::Boolean(true));
                assert_eq!(hello[2], JsonVal::Null);
                assert_eq!(hello[3], json_num!(42; uint));
                let s = String::from("foo\n\\u1234");
                println!("{}", s);
                //assert_eq!(hello[4], json_str!(s));
                if let JsonVal::Array(ref arr) = hello[5] {
                    assert_eq!(arr[0], json_num!(1; uint));
                    assert_eq!(arr[1], json_num!(-2; int));
                    assert_eq!(arr[2], json_num!(3.1415; float));
                    assert_eq!(arr[3], json_num!(0.000004; float));
                    assert_eq!(arr[4], json_num!(5000000f64; float));
                    assert_eq!(arr[5], json_num!(1.23; float));
                } else {
                }
            } else {
                unreachable!("Must parse as an array, {:#?}", parsed)
            }
        } else {
            unreachable!("Must parse as an object, {:#?}", parsed)
        }
    }
}
