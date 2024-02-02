use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "logalang.pest"]
pub struct LogalangParser;

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test() {
        let input = "a=\"b\"";

        let mut result = LogalangParser::parse(Rule::filter, input).unwrap();

        let first = result.next().unwrap();
        assert_eq!(Rule::filter, first.as_rule());

        let matches = first.into_inner().collect::<Vec<_>>();
        assert_eq!(Rule::column_name, matches[0].as_rule());
        assert_eq!(Rule::expr, matches[1].as_rule());
    }

    #[test]
    fn test2() {
        let input = "asdf=!\"b1234\"";

        let mut result = LogalangParser::parse(Rule::filter, input).unwrap();

        let first = result.next().unwrap();
        assert_eq!(Rule::filter, first.as_rule());

        let matches = first.into_inner().collect::<Vec<_>>();
        assert_eq!(Rule::column_name, matches[0].as_rule());
        assert_eq!(Rule::expr, matches[1].as_rule());

        let expr_matches = matches[1].clone().into_inner().collect::<Vec<_>>();
        assert_eq!(Rule::not, expr_matches[0].as_rule());
        assert_eq!(Rule::string, expr_matches[1].as_rule());
    }
}
