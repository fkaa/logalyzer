use pest::iterators::*;
use crate::db::*;
use crate::logalang::Rule;

pub fn to_filter_rule(rule: Pairs<Rule>) -> FilterRule {
    panic!()
}

fn internal_to_filter_rule(rule: Pair<Rule>) -> Filter {
    for token in rule.into_inner() {
        match token.as_rule() {
            Rule::EOI => {}
            Rule::string => {}
            Rule::inner => {}
            Rule::char => {}
            Rule::operation => {}
            Rule::and => {}
            Rule::or => {}
            Rule::not => {}
            Rule::column_name => {}
            Rule::filter => {}
            Rule::expr => {}
            Rule::term => {}
            Rule::calculation => {}
            Rule::WHITESPACE => {}
        }
    }

    return Filter::ContainsString("hello".to_string());
}

#[cfg(test)]
mod test {
    use pest::iterators::*;
    use crate::logalang::Rule;
    use crate::logalang_converter::internal_to_filter_rule;

    #[test]
    fn test1() {
        // let filter = internal_to_filter_rule(rule);
    }
}