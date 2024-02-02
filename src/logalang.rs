use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use crate::db::sanitize_filter;

#[derive(Parser)]
#[grammar = "logalang.pest"]
pub struct LogalangParser;

pub fn to_filter_rule(rule: Pairs<Rule>) -> FilterRule {
    let mut column_name = String::new();
    let mut filters = Vec::new();

    // Iterate over pairs
    for pair in rule {
        let mut rule_filter_pairs = pair.into_inner();

        column_name = rule_filter_pairs.next().unwrap().as_str().to_string();

        let filter_pairs = rule_filter_pairs.next().unwrap().into_inner();
        let filter = to_filter(filter_pairs);
        filters.push(filter);
    }

    FilterRule {
        column_name,
        rules: filters,
    }
}

fn to_filter(pairs: Pairs<Rule>) -> Filter {
    let mut inner_pairs = pairs.into_iter();

    let first_pair = inner_pairs.next().unwrap();
    match first_pair.as_rule() {
        Rule::not => {
            // If it's a NOT expression
            let inner_filter = to_filter(inner_pairs);
            Filter::Not(Box::new(inner_filter))
        }
        Rule::and => {
            // If it's an AND expression
            let mut filters = Vec::new();
            for pair in first_pair.into_inner() {
                let inner_filter = to_filter(Pairs::single(pair));
                filters.push(inner_filter);
            }
            Filter::And(Box::new(filters[0].clone()), Box::new(filters[1].clone()))
        }
        Rule::or => {
            // If it's an OR expression
            let mut filters = Vec::new();
            for pair in first_pair.into_inner() {
                let inner_filter = to_filter(Pairs::single(pair));
                filters.push(inner_filter);
            }
            Filter::Or(Box::new(filters[0].clone()), Box::new(filters[1].clone()))
        }
        Rule::string => {
            // If it's a string literal
            // Strip the \"
            let s = first_pair.as_str().to_string();
            let s = &s[1..];
            let s = &s[..s.len() - 1];

            Filter::ContainsString(s.to_string())
        }
        m => panic!("{:?}", m), // Assuming all other rules are unreachable
    }
}

#[derive(Debug)]
pub struct FilterRule {
    pub(crate) column_name: String,
    pub(crate) rules: Vec<Filter>,
}

impl FilterRule {
    pub fn get_sql(&self) -> String {
        if self.rules.is_empty() {
            return String::new();
        }

        format!(
            "WHERE {}",
            self.rules
                .iter()
                .map(|r| r.get_sql(&self.column_name))
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    }
}

#[derive(Debug, Clone)]
pub enum Filter {
    And(Box<Filter>, Box<Filter>),
    Or(Box<Filter>, Box<Filter>),
    Not(Box<Filter>),
    ContainsString(String),
}

impl Filter {
    fn get_sql(&self, column_name: &str) -> String {
        match self {
            Filter::And(left, right) => {
                format!(
                    "{} AND {}",
                    left.get_sql(column_name),
                    right.get_sql(column_name)
                )
            }
            Filter::Or(left, right) => {
                format!(
                    "{} OR {}",
                    left.get_sql(column_name),
                    right.get_sql(column_name)
                )
            }
            Filter::Not(other_filter) => {
                format!("NOT ({})", other_filter.get_sql(column_name))
            }
            Filter::ContainsString(pat) => {
                format!("{column_name} LIKE '%{}%'", sanitize_filter(pat))
            }
        }
    }
}

pub fn parse_line(line: &str) -> Result<FilterRule, pest::error::Error<Rule>> {
    let result = LogalangParser::parse(Rule::filter, line)?;

    Ok(to_filter_rule(result))
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn test_parse_line_into_filter_rule() {
        let result = parse_line("columnName=\"a\"");

        assert_matches!(
            result,
            Ok(filter) => {
                assert_eq!(filter.column_name, "columnName");

                assert_matches!(&filter.rules[..], [filter] => {
                    assert_matches!(filter, Filter::ContainsString(text) => {
                        assert_eq!(text, "a");
                    })
                });
            }
        );
    }

    #[test]
    fn filter_get_sql_contains() {
        let filter = Filter::ContainsString("blabla".into());

        assert_eq!(filter.get_sql("message"), "message LIKE '%blabla%'");
    }

    #[test]
    fn filter_get_sql_not() {
        let filter = Filter::Not(Box::new(Filter::ContainsString("blabla".into())));

        assert_eq!(filter.get_sql("message"), "NOT (message LIKE '%blabla%')");
    }

    #[test]
    fn filter_get_sql_and() {
        let filter = Filter::And(
            Box::new(Filter::ContainsString("lhs".into())),
            Box::new(Filter::ContainsString("rhs".into())),
        );

        assert_eq!(
            filter.get_sql("message"),
            "message LIKE '%lhs%' AND message LIKE '%rhs%'"
        );
    }

    #[test]
    fn filter_get_sql_or() {
        let filter = Filter::Or(
            Box::new(Filter::ContainsString("lhs".into())),
            Box::new(Filter::ContainsString("rhs".into())),
        );

        assert_eq!(
            filter.get_sql("message"),
            "message LIKE '%lhs%' OR message LIKE '%rhs%'"
        );
    }

    #[test]
    fn filter_rule_get_sql_none() {
        let filter = FilterRule {
            column_name: "message".to_string(),
            rules: vec![],
        };

        assert_eq!(filter.get_sql(), "");
    }

    #[test]
    fn filter_rule_get_sql_single() {
        let filter = FilterRule {
            column_name: "message".to_string(),
            rules: vec![Filter::ContainsString("bla".to_string())],
        };

        assert_eq!(filter.get_sql(), "WHERE message LIKE '%bla%'");
    }

    #[test]
    fn filter_rule_get_sql_multiple() {
        let filter = FilterRule {
            column_name: "message".to_string(),
            rules: vec![
                Filter::ContainsString("bla1".to_string()),
                Filter::ContainsString("bla2".to_string()),
            ],
        };

        assert_eq!(
            filter.get_sql(),
            "WHERE message LIKE '%bla1%' AND message LIKE '%bla2%'"
        );
    }

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
