pub mod pattern {
    use std::collections::HashSet;

    use regex::Regex;
    use uuid::Uuid;

    use crate::dictionary::word::word::{get_word_tags, get_wrapper_content};

    static PATTERN_WRAPPER: &str = "TEMPLATE";

    static PATTERN_GLOBAL_REGEX: &str = r"(?:\[(\[(?:[a-zA-Z]+,? ?)+\],? ?)+\])|[a-zA-Z ]+";
    static PATTERN_FORMAT_TOP_REGEX: &str = r"\[(\[(?:[a-zA-Z]+,? ?)+\],? ?)+\]";
    static PATTERN_FORMAT_AND_GROUP_REGEX: &str = r"\[(?:[a-zA-Z]+,? ?)+\]";
    static PATTERN_FORMAT_OR_GROUP_REGEX: &str = r"([a-zA-Z ]+)+";

    pub struct Pattern {
        pub id: Uuid,
        pub pattern: Vec<Vec<String>>,
        pub tags: HashSet<String>,
    }

    // TODO - use parse_pattern_term to parse a broader pattern with free text

    fn parse_pattern(line: &str) {
        let pattern_wrapped = get_wrapper_content(PATTERN_WRAPPER, line);
        if pattern_wrapped.is_some() {
            let p = pattern_wrapped.unwrap();
            let global_pattern = Regex::new(PATTERN_GLOBAL_REGEX).unwrap();
            let pattern_sections = global_pattern.captures_iter(&p);
            for section in pattern_sections {
                println!("{:#?}", section);
            }
        } else {
            println!("Not Wrapped Pattern")
        }
    }

    fn parse_pattern_term(line: &str) -> Vec<Vec<String>> {
        let mut output: Vec<Vec<String>> = Vec::new();
        let top_pattern = Regex::new(PATTERN_FORMAT_TOP_REGEX).unwrap();
        let top_match = top_pattern.find(line);
        if top_match.is_some() {
            let top = top_match.unwrap().as_str();
            let and_group_pattern = Regex::new(PATTERN_FORMAT_AND_GROUP_REGEX).unwrap();
            for and_group in and_group_pattern.find_iter(top) {
                let mut or_vec: Vec<String> = Vec::new();
                let and = and_group.as_str();
                let or_group_pattern = Regex::new(PATTERN_FORMAT_OR_GROUP_REGEX).unwrap();
                for or_group in or_group_pattern.captures_iter(and) {
                    or_vec.push(or_group.get(0).unwrap().as_str().trim().to_string());
                }
                output.push(or_vec);
            }
        }
        return output;
    }

    // example pattern string
    //  [[Metal, Wood]] [[Mammal]] Tavern

    #[test]
    fn test_parse_pattern() {
        parse_pattern("TEMPLATE([[Metal, Wood]] [[Mammal]] Tavern)");
    }
}
