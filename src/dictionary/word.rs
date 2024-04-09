pub mod word {
    use std::collections::HashSet;

    use regex::Regex;
    use uuid::Uuid;

    use crate::dictionary::dictionary::{ADJECTIVE_WRAPPER, NOUN_WRAPPER, TAG_WRAPPER};

    #[derive(PartialEq, Debug, Clone, Hash, Eq)]
    pub enum WordType {
        Noun,
        Adjective,
    }

    #[derive(PartialEq, Debug, Clone, Hash, Eq)]
    pub enum WordRelationType {
        BaseNoun,
        Adjective,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct Word {
        pub id: Uuid,
        pub base: String,
        pub word_type: WordType,
        pub recipie: Option<Vec<Vec<String>>>,
        pub tags: HashSet<String>,
        pub related: HashSet<(WordRelationType, Uuid)>,
    }

    fn get_wrapper_content(wrapper: &str, line: &str) -> Option<String> {
        let regex = Regex::new(&format!(r"{}\(([a-zA-Z0-9]+)\)", wrapper)).unwrap();
        let capture = regex.captures(line);
        if capture.is_some() {
            return Some(capture.unwrap().get(1).unwrap().as_str().to_string());
        } else {
            return None;
        }
    }

    fn get_word_tags(line: &str) -> Vec<String> {
        let regex = Regex::new(&format!(r",?\s?{}\(([a-zA-Z0-9]+)\)", TAG_WRAPPER)).unwrap();
        return regex
            .find_iter(line)
            .map(|m| {
                regex
                    .captures(m.as_str())
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string()
            })
            .collect();
    }

    pub fn parse_word(line: &str) -> Vec<Word> {
        let mut output: Vec<Word> = vec![];
        let noun_value = get_wrapper_content(NOUN_WRAPPER, line);
        let adjective_value = get_wrapper_content(ADJECTIVE_WRAPPER, line);
        let noun_id = Uuid::new_v4();
        let adjective_id = Uuid::new_v4();
        let tags: HashSet<String> = if noun_value.is_some() || adjective_value.is_some() {
            HashSet::from_iter(get_word_tags(line).iter().cloned())
        } else {
            HashSet::new()
        };
        if noun_value.is_some() {
            let mut rel: HashSet<(WordRelationType, Uuid)> = HashSet::new();
            if adjective_value.is_some() {
                rel.insert((WordRelationType::Adjective, adjective_id));
            }
            output.push(Word {
                id: noun_id.clone(),
                base: noun_value.unwrap().clone(),
                word_type: WordType::Noun,
                recipie: None,
                tags: tags.clone(),
                related: rel,
            });
        }

        if adjective_value.is_some() {
            let mut rel: HashSet<(WordRelationType, Uuid)> = HashSet::new();
            if output.len() > 0 {
                rel.insert((WordRelationType::BaseNoun, noun_id));
            }
            output.push(Word {
                id: adjective_id.clone(),
                base: adjective_value.unwrap(),
                word_type: WordType::Adjective,
                recipie: None,
                tags: tags.clone(),
                related: rel,
            });
        }
        return output;
    }

    #[test]
    fn parse_word_test() {
        let steel_words = parse_word("NOUN(steel), ADJECTIVE(steely), TAG(metal), TAG(ferrous)");
        let steel_noun = steel_words.get(0).unwrap();
        let steel_adj = steel_words.get(1).unwrap();
        assert!(steel_noun.word_type.eq(&WordType::Noun));
        assert!(steel_noun.related.len().eq(&1));
        assert!(steel_noun.tags.len().eq(&2));
        assert!(steel_adj.word_type.eq(&WordType::Adjective));
        assert!(steel_adj.related.len().eq(&1));
        assert!(steel_adj.tags.len().eq(&2));
    }
}
