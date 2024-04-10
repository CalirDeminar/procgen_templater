pub mod template {
    use std::collections::HashSet;

    use regex::Regex;
    use uuid::Uuid;

    use crate::dictionary::{
        dictionary::SearchPattern,
        word::word::{get_word_tags, get_wrapper_content, WordType},
    };

    static TEMPLATE_WRAPPER: &str = "TEMPLATE";

    static TEMPLATE_GLOBAL_REGEX: &str = r"(?:\[(\[(?:[a-zA-Z]+,? ?)+\],? ?)+\])|[a-zA-Z ]+";
    static TEMPLATE_FORMAT_TOP_REGEX: &str = r"\[(\[(?:[a-zA-Z]+,? ?)+\],? ?)+\]";
    static TEMPLATE_FORMAT_AND_GROUP_REGEX: &str = r"\[(?:[a-zA-Z]+,? ?)+\]";
    static TEMPLATE_FORMAT_OR_GROUP_REGEX: &str = r"([a-zA-Z ]+)+";

    #[derive(PartialEq, Debug, Clone)]
    pub struct TemplateElement {
        pub text: Option<String>,
        pub template: Option<SearchPattern>,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct Template {
        pub id: Uuid,
        pub template: Vec<TemplateElement>,
        pub tags: HashSet<String>,
    }

    pub fn parse_template(line: &str) -> Option<Template> {
        let template_wrapped = get_wrapper_content(TEMPLATE_WRAPPER, line);
        let format_template = Regex::new(TEMPLATE_FORMAT_TOP_REGEX).unwrap();
        let global_template = Regex::new(TEMPLATE_GLOBAL_REGEX).unwrap();
        let tags = get_word_tags(line);
        let mut output: Vec<TemplateElement> = Vec::new();
        if template_wrapped.is_some() {
            let p = template_wrapped.unwrap();
            let template_sections = global_template.captures_iter(&p);
            for section in template_sections {
                let section_text = section.get(0).unwrap().as_str();
                if format_template.is_match(section_text) {
                    output.push(TemplateElement {
                        text: None,
                        template: Some(parse_template_term(section_text)),
                    });
                } else {
                    output.push(TemplateElement {
                        text: Some(section_text.to_string()),
                        template: None,
                    });
                }
            }
            return Some(Template {
                id: Uuid::new_v4(),
                template: output,
                tags: HashSet::from_iter(tags),
            });
        }
        return None;
    }

    fn parse_template_term(line: &str) -> SearchPattern {
        // TODO - pull type as prefix from pattern format
        let word_type = WordType::Noun;
        let mut output: Vec<Vec<String>> = Vec::new();
        let top_template = Regex::new(TEMPLATE_FORMAT_TOP_REGEX).unwrap();
        let top_match = top_template.find(line);
        if top_match.is_some() {
            let top = top_match.unwrap().as_str();
            let and_group_template = Regex::new(TEMPLATE_FORMAT_AND_GROUP_REGEX).unwrap();
            for and_group in and_group_template.find_iter(top) {
                let mut or_vec: Vec<String> = Vec::new();
                let and = and_group.as_str();
                let or_group_template = Regex::new(TEMPLATE_FORMAT_OR_GROUP_REGEX).unwrap();
                for or_group in or_group_template.captures_iter(and) {
                    or_vec.push(or_group.get(0).unwrap().as_str().trim().to_string());
                }
                output.push(or_vec);
            }
        }
        return (word_type, output);
    }

    // example template string
    //  [[Metal, Wood]] [[Mammal]] Tavern

    #[test]
    fn test_parse_template() {
        assert!(parse_template(
            "TEMPLATE([[Metal, Wood]] [[Mammal]] Tavern), TAG(Institution), TAG(Restaurant)"
        )
        .unwrap()
        .template
        .len()
        .eq(&4));
        // len being Word, space, Word, Tavern
    }
}
