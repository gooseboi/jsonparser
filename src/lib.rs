pub mod tokenizer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let parser = tokenizer::Tokenizer::from_str(
            r#"{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address": {
    "streetAddress": "21 2nd Street",
    "city": "New York",
    "state": "NY",
    "postalCode": "10021-3100"
  },
  "phoneNumbers": [
    {
      "type": "home",
      "number": "212 555-1234"
    },
    {
      "type": "office",
      "number": "646 555-4567"
    }
  ],
  "children": [],
  "spouse": null,
}"#,
        );
        let v: Vec<_> = parser.collect();
        println!("{:?}", v);
    }
}
