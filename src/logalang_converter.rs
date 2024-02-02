use pest::iterators::*;
use crate::db::*;
use crate::logalang::Rule;

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

    FilterRule { column_name, rules: filters }
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
            Filter::ContainsString(first_pair.as_str().to_string())
        }
        m => panic!("{:?}", m), // Assuming all other rules are unreachable
    }
}

#[cfg(test)]
mod test {
    use pest::Parser;
    use crate::logalang::LogalangParser;

    use super::*;

    #[test]
    fn test_to_filter_rule() {
        // Define a sample input Pairs<Rule>
        let input = LogalangParser::parse(Rule::filter, "columnName=\"A\"").unwrap();
        // Call the to_filter_rule function
        let result = to_filter_rule(input);

        // Define expected output
        let expected_column_name = "columnName".to_string();

        // Assert that the result matches the expected output
        assert_eq!(result.column_name, expected_column_name);
    }
}
