pub mod people {
    use crate::dictionary::{dictionary::Dictionary, word::word::WordType};

    #[derive(PartialEq, Debug, Clone, Hash, Eq)]
    pub enum Gender {
        Male,
        Female,
        Ambigious,
    }

    pub fn build_name(dict: &Dictionary, gender: Option<Gender>) -> String {
        let mut gender_term = "AmbiguousGender";
        if gender.is_some() {
            let g = gender.unwrap();
            if g.eq(&Gender::Male) {
                gender_term = "Male";
            } else if g.eq(&Gender::Female) {
                gender_term = "Female";
            }
        }

        let first = dict
            .get_random_word((
                WordType::Noun,
                vec![vec!["FirstName".to_string()], vec![gender_term.to_string()]],
            ))
            .unwrap();
        let last = dict
            .get_random_word((WordType::Noun, vec![vec!["LastName".to_string()]]))
            .unwrap();
        return format!("{} {}", first.base, last.base);
    }

    pub fn build_description(dict: &Dictionary) -> String {
        let hair_colour = dict
            .get_random_word((WordType::Adjective, vec![vec!["HairColour".to_string()]]))
            .unwrap();
        let hair_style = dict
            .get_random_word((
                WordType::Adjective,
                vec![vec!["HairStyle".to_string()], vec!["Personal".to_string()]],
            ))
            .unwrap();
        let hair_state = dict
            .get_random_word((
                WordType::Adjective,
                vec![vec!["HairState".to_string()], vec!["Personal".to_string()]],
            ))
            .unwrap();
        let eye_colour = dict
            .get_random_word((WordType::Adjective, vec![vec!["EyeColour".to_string()]]))
            .unwrap();
        let build = dict
            .get_random_word((
                WordType::Adjective,
                vec![vec!["Build".to_string()], vec!["Personal".to_string()]],
            ))
            .unwrap();
        return format!(
            "They are {} with {} {} {} hair and {} eyes",
            build.base, hair_state.base, hair_style.base, hair_colour.base, eye_colour.base
        );
    }

    #[test]
    fn test_helpers() {
        use crate::build_default_dictionary;
        let dict = build_default_dictionary();
        dict.inspect();
        for _i in 0..100 {
            build_name(&dict, None);
            build_description(&dict);
        }
    }
}
