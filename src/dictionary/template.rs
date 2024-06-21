pub mod template {
    use std::collections::HashSet;

    use regex::Regex;
    use uuid::Uuid;

    use crate::dictionary::{
        dictionary::{Dictionary, SearchPattern},
        word::word::{get_word_tags, WordType},
    };

    static TEMPLATE_WRAPPER: &str = "TEMPLATE";

    #[derive(PartialEq, Debug, Clone)]
    pub struct TemplateElement {
        pub text: Option<String>,
        pub template: Option<SearchPattern>,
    }

    impl Dictionary {
        pub fn render_template(self: &Self, template_id: &Uuid) -> Option<String> {
            let t = self.templates.get(&template_id);
            if t.is_some() {
                let template = t.unwrap();
                let components: Vec<String> = template
                    .template
                    .iter()
                    .map(|c| {
                        if c.template.is_some() {
                            self.get_random_word((
                                c.template.clone().unwrap().0,
                                c.template.clone().unwrap().1,
                            ))
                            .unwrap()
                            .base
                            .clone()
                        } else {
                            c.text.clone().unwrap()
                        }
                    })
                    .collect();
                return Some(components.join(" "));
            }
            return None;
        }
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct Template {
        pub id: Uuid,
        pub template: Vec<TemplateElement>,
        pub tags: HashSet<String>,
    }

    pub fn parse_template(line: &str) -> Option<Template> {
        if !line.contains(TEMPLATE_WRAPPER) {
            return None;
        }
        let and_groups_pattern =
            Regex::new(r"(ADJECTIVE|NOUN)\[(\[(?:[a-zA-Z, ]+)+\])+\]").unwrap();
        let subset_pattern =
            Regex::new(&format!(r"{}|(?:[a-zA-Z]+)", and_groups_pattern.as_str())).unwrap();
        let search_pattern = Regex::new(&format!(
            r"TEMPLATE\((?:(?:(?:{}\s?)+)+|(?:[A-Za-z ]+))+\)",
            subset_pattern.as_str()
        ))
        .unwrap();
        let search_result = search_pattern.find(line);

        let mut output = Template {
            id: Uuid::new_v4(),
            template: Vec::new(),
            tags: HashSet::new(),
        };
        if search_result.is_none() {
            return None;
        }
        for subset in subset_pattern
            .find_iter(search_result.unwrap().as_str())
            .map(|m| m.as_str())
        {
            let mut pattern: SearchPattern = (WordType::Noun, Vec::new());
            let and_groups = and_groups_pattern.captures(subset);
            if and_groups.is_some() {
                for and in and_groups.unwrap().iter().skip(1) {
                    let and_str = and.unwrap().as_str();
                    if and_str.eq("NOUN") {
                        pattern.0 = WordType::Noun;
                    } else if and_str.eq("ADJECTIVE") {
                        pattern.0 = WordType::Adjective;
                    } else if and_str.eq("TEMPLATE") {
                    } else {
                        let mut or_groups = and_str.replace("]", "");
                        or_groups = or_groups.replace("[", "");
                        let mut or_output: Vec<String> = Vec::new();
                        for section in or_groups.split(",") {
                            or_output.push(section.to_string());
                        }
                        pattern.1.push(or_output);
                    }
                }
                output.template.push(TemplateElement {
                    text: None,
                    template: Some(pattern),
                });
            } else {
                if !subset.eq(TEMPLATE_WRAPPER) {
                    output.template.push(TemplateElement {
                        text: Some(subset.to_string()),
                        template: None,
                    });
                }
            }
        }

        output.tags = HashSet::from_iter(get_word_tags(line));
        return Some(output);
    }

    // example template string
    //  [[Metal, Wood]] [[Mammal]] Tavern

    #[test]
    fn test_parse_template() {
        let test_string = "TEMPLATE(ADJECTIVE[[Metal, Wood]] NOUN[[Mammal]] Tavern), TAG(Institution), TAG(Restaurant)";

        let template = parse_template(test_string).unwrap();

        assert!(template.template.len().eq(&3));
        assert!(template.tags.len().eq(&2));
    }

    #[test]
    fn test_template_render() {
        use crate::dictionary::dictionary::build_dictionary;
        let dict = build_dictionary(vec![
            "TEMPLATE(ADJECTIVE[[Colour]] NOUN[[Metal]] Bull Pub), TAG(Restaurant)".to_string(),
            "ADJECTIVE(Blue), TAG(Colour)".to_string(),
            "NOUN(Steel), TAG(Metal), TAG(Ferrous), TAG(Alloy)".to_string(),
            "NOUN(Oak), TAG(Tree)".to_string(),
            "NOUN(Pear), TAG(Tree), TAG(Fruit)".to_string(),
            "TAG(Metal), HAS_PARENT(Material)".to_string(),
            "TAG(Tree), HAS_PARENT(Wood), HAS_PARENT(Plant)".to_string(),
            "TAG(Wood), HAS_PARENT(Material)".to_string(),
            "TAG(Fruit), HAS_PARENT(Food)".to_string(),
            "TAG(Restaurant), HAS_PARENT(Institution)".to_string(),
        ]);
        let template_keys = Vec::from_iter(dict.templates.keys());
        let template = template_keys.first().unwrap();
        assert!(dict
            .render_template(template)
            .unwrap()
            .eq("Blue Steel Bull Pub"));
    }
}
