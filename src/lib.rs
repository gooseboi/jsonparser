pub mod tokenizer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let input = include_str!("../test.json");
        let parser = tokenizer::Tokenizer::from_str(&input);
        let v: Vec<_> = parser.collect();
        println!("{:#?}", v);
        println!("{}", input);
    }
}
