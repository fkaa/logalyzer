use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use crate::db::sanitize_filter;

#[derive(Parser)]
#[grammar = "logalang.pest"]
pub struct LogalangParser;

pub fn to_filter_rule(mut rule: Pairs<Rule>) -> FilterRule {
    let mut column_name = String::new();

    // Iterate over pairs
    let pair = rule.next().unwrap();
    let filter = {
        let mut rule_filter_pairs = pair.into_inner();

        column_name = rule_filter_pairs.next().unwrap().as_str().to_string();

        let filter_pairs = rule_filter_pairs.next().unwrap().into_inner();
        let filter = to_filter(filter_pairs);
        filter
    };

    FilterRule {
        column_name,
        rules: filter,
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
    pub(crate) rules: Filter,
}

impl FilterRule {
    pub fn get_sql(&self) -> String {
        self.rules.get_sql(&self.column_name)
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

pub fn parse_line(line: &str) -> Result<Filter, pest::error::Error<Rule>> {
    return Ok(Filter::ContainsString(line.to_string()));
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn test_parse_line_into_filter_rule() {
        let result = parse_line("a");

        assert_matches!(
            result,
            Ok(filter) => {
                assert_matches!(filter, Filter::ContainsString(text) => {
                    assert_eq!(text, "a");
                })
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
    fn filter_rule_get_sql_single() {
        let filter = FilterRule {
            column_name: "message".to_string(),
            rules: Filter::ContainsString("bla".to_string()),
        };

        assert_eq!(filter.get_sql(), "WHERE message LIKE '%bla%'");
    }
}
