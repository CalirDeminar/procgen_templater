pub mod template;
pub mod word;
pub mod dictionary {
    use super::{
        template::template::{parse_template, Template},
        word::word::{parse_word, Word, WordType},
    };
    use rand::seq::SliceRandom;
    use regex::Regex;
    use std::{
        collections::{HashMap, HashSet},
        fs::{self, File},
        io::{self, BufRead},
        time::Instant,
    };
    use uuid::Uuid;

    pub static NOUN_WRAPPER: &str = "NOUN";
    pub static ADJECTIVE_WRAPPER: &str = "ADJECTIVE";
    pub static TAG_WRAPPER: &str = "TAG";
    pub static TAG_PARENT_WRAPPER: &str = "HAS_PARENT";

    pub static MAX_NESTED_TAG_DEPTH: usize = 5;

    #[derive(PartialEq, Debug, Clone)]
    pub struct Index {
        pub tag_children: HashMap<String, HashSet<String>>,
        pub tag_words: HashMap<(WordType, String), HashSet<Uuid>>,
        pub tag_templates: HashMap<String, HashSet<Uuid>>,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct Dictionary {
        pub words: HashMap<Uuid, Word>,
        pub templates: HashMap<Uuid, Template>,
        pub index: Index,
    }

    pub type SearchPattern = (WordType, Vec<Vec<String>>);

    impl Dictionary {
        pub fn inspect(self: &Self) {
            println!("Dictionary: ");
            println!("Words: {}", self.words.len());
            println!("Tags: ");
            for (k, v) in &self.index.tag_words {
                println!("{:?}: {}", k, v.len());
            }
        }
        pub fn get_random_word(self: &Self, pattern: SearchPattern) -> Option<&Word> {
            // tag arguments [[OR] AND [OR]]
            let (word_type, tags) = pattern;
            let mut word_pool: HashSet<Uuid> = HashSet::new();
            for or_set in &tags {
                let mut s: HashSet<Uuid> = HashSet::new();
                for or in or_set {
                    if self
                        .index
                        .tag_words
                        .contains_key(&(word_type.clone(), or.to_string()))
                    {
                        let tag_ids = self
                            .index
                            .tag_words
                            .get(&(word_type.clone(), or.to_string()))
                            .unwrap();
                        for id in tag_ids {
                            s.insert(*id);
                        }
                    }
                }
                if word_pool.len().eq(&0) {
                    word_pool = s;
                } else {
                    let pool_clone = word_pool.clone();
                    for word in pool_clone {
                        if !s.contains(&word) {
                            word_pool.remove(&word);
                        }
                    }
                }
            }
            let mut pool: Vec<&Word> = word_pool
                .iter()
                .map(|w| self.words.get(w).unwrap())
                .collect();
            pool.shuffle(&mut rand::thread_rng());

            if pool.first().is_some() {
                return Some(*pool.first().unwrap());
            }
            return None;
        }

        pub fn get_random_word_without(
            self: &Self,
            pattern: SearchPattern,
            exclude: Vec<String>,
        ) -> Option<&Word> {
            // tag arguments [[OR] AND [OR]]
            let (word_type, tags) = pattern;
            let mut word_pool: HashSet<Uuid> = HashSet::new();
            for or_set in &tags {
                let mut s: HashSet<Uuid> = HashSet::new();
                for or in or_set {
                    if self
                        .index
                        .tag_words
                        .contains_key(&(word_type.clone(), or.to_string()))
                    {
                        let tag_ids = self
                            .index
                            .tag_words
                            .get(&(word_type.clone(), or.to_string()))
                            .unwrap();
                        for id in tag_ids {
                            s.insert(*id);
                        }
                    }
                }
                if word_pool.len().eq(&0) {
                    word_pool = s;
                } else {
                    let pool_clone = word_pool.clone();
                    for word in pool_clone {
                        if !s.contains(&word) {
                            word_pool.remove(&word);
                        }
                    }
                }
            }
            let mut pool: Vec<&Word> = word_pool
                .iter()
                .map(|w| self.words.get(w).unwrap())
                .collect();
            pool.shuffle(&mut rand::thread_rng());
            pool.retain(|w| !w.tags.iter().any(|t| exclude.contains(t)));

            if pool.first().is_some() {
                return Some(*pool.first().unwrap());
            }
            return None;
        }

        pub fn get_random_template(self: &Self, tags: Vec<Vec<String>>) -> Option<&Template> {
            let mut pattern_pool: HashSet<Uuid> = HashSet::new();
            for or_set in &tags {
                let mut s: HashSet<Uuid> = HashSet::new();
                for or in or_set {
                    if self.index.tag_templates.contains_key(or) {
                        let tag_ids = self.index.tag_templates.get(or).unwrap();
                        for id in tag_ids {
                            s.insert(id.clone());
                        }
                    }
                }
                if pattern_pool.len().eq(&0) {
                    pattern_pool = s;
                } else {
                    let pool_clone = pattern_pool.clone();
                    for pattern in pool_clone {
                        if !s.contains(&pattern) {
                            pattern_pool.remove(&pattern);
                        }
                    }
                }
            }
            let mut pool: Vec<&Template> = pattern_pool
                .iter()
                .map(|p| self.templates.get(p).unwrap())
                .collect();
            pool.shuffle(&mut rand::thread_rng());
            if pool.first().is_some() {
                return Some(*pool.first().unwrap());
            }
            return None;
        }
    }

    pub struct ParseResult {
        words: Vec<Word>,
        tag_children: HashMap<String, HashSet<String>>,
        pattern: Option<Template>,
    }

    pub fn build_dictionary(lines: Vec<String>) -> Dictionary {
        // let start = Instant::now();
        let mut output = Dictionary {
            words: HashMap::new(),
            templates: HashMap::new(),
            index: Index {
                tag_children: HashMap::new(),
                tag_words: HashMap::new(),
                tag_templates: HashMap::new(),
            },
        };
        let mut word_times: (f64, f64, f64) = (0.0, 0.0, 0.0);
        for line in &lines {
            let parse = parse_line(line, &mut word_times);

            for word in parse.words {
                output.words.insert(word.id.clone(), word);
            }

            if parse.pattern.is_some() {
                let pattern = parse.pattern.unwrap();
                output.templates.insert(pattern.id.clone(), pattern);
            }
            for (parent, children) in parse.tag_children {
                if !output.index.tag_children.contains_key(&parent) {
                    output
                        .index
                        .tag_children
                        .insert(parent.clone(), HashSet::new());
                }
                for child in children {
                    output
                        .index
                        .tag_children
                        .get_mut(&parent)
                        .unwrap()
                        .insert(child);
                }
            }
        }
        propegate_tag_children(&mut output);
        build_tag_index(&mut output);
        // println!(
        //     "Built in {}ms\nWord Parse: {:.0}ms\nTag Parse: {:.0}ms\nPattern Parse: {:.0}ms",
        //     Instant::now().duration_since(start).as_millis(),
        //     word_times.0 * 1000.0,
        //     word_times.1 * 1000.0,
        //     word_times.2 * 1000.0
        // );
        return output;
    }

    pub fn build_dictionary_from_folder(folder_path: &str) -> Dictionary {
        let paths = fs::read_dir(folder_path).unwrap();
        let mut output: Vec<String> = Vec::new();
        for path in paths {
            let filename = path.unwrap().file_name();
            let data = File::open(&format!("{}/{}", folder_path, filename.to_str().unwrap()))
                .expect(&format!("Cannot open: {}", filename.into_string().unwrap()));
            let lines = io::BufReader::new(data).lines();
            for l in lines {
                if l.is_ok() {
                    let line = l.unwrap();
                    if line.len() > 0 {
                        output.push(line);
                    }
                }
            }
        }
        return build_dictionary(output);
    }

    fn propegate_tag_children(dict: &mut Dictionary) {
        let mut processed_parents: HashSet<String> = HashSet::new();
        for parent in dict.index.tag_children.keys() {
            if !processed_parents.contains(parent) {
                let children = dict.index.tag_children.get(parent).unwrap();

                for word in dict
                    .words
                    .values_mut()
                    .filter(|w| w.tags.iter().any(|t| children.contains(t)))
                {
                    word.tags.insert(parent.clone());
                }
                for template in dict
                    .templates
                    .values_mut()
                    .filter(|tem| tem.tags.iter().any(|tag| children.contains(tag)))
                {
                    template.tags.insert(parent.clone());
                }
                processed_parents.insert(parent.to_string());
            }
        }
    }
    fn build_tag_index(dict: &mut Dictionary) {
        for _i in 0..MAX_NESTED_TAG_DEPTH {
            let ref_words = dict.words.clone();
            for word in ref_words.values() {
                for tag in &word.tags {
                    if !dict
                        .index
                        .tag_words
                        .contains_key(&(word.word_type.clone(), tag.to_string()))
                    {
                        dict.index
                            .tag_words
                            .insert((word.word_type.clone(), tag.clone()), HashSet::new());
                    }
                    dict.index
                        .tag_words
                        .get_mut(&(word.word_type.clone(), tag.clone()))
                        .unwrap()
                        .insert(word.id.clone());
                }
            }
            let ref_templates = dict.templates.clone();
            for template in ref_templates.values() {
                for tag in &template.tags {
                    if !dict.index.tag_templates.contains_key(&tag.to_string()) {
                        dict.index.tag_templates.insert(tag.clone(), HashSet::new());
                    }
                    dict.index
                        .tag_templates
                        .get_mut(tag)
                        .unwrap()
                        .insert(template.id.clone());
                }
            }
        }
    }

    fn parse_line(line: &str, times: &mut (f64, f64, f64)) -> ParseResult {
        let word_start = Instant::now();
        let words = parse_word(line);
        times.0 += word_start.elapsed().as_secs_f64();

        let tag_start = Instant::now();
        let tag_children = parse_tag_children(line);

        times.1 += tag_start.elapsed().as_secs_f64();
        let pattern_start = Instant::now();
        let pattern = parse_template(line);
        times.2 += pattern_start.elapsed().as_secs_f64();
        return ParseResult {
            words,
            tag_children,
            pattern,
        };
    }

    fn parse_tag_children(line: &str) -> HashMap<String, HashSet<String>> {
        if !line.contains(TAG_PARENT_WRAPPER) {
            return HashMap::new();
        }
        let mut output: HashMap<String, HashSet<String>> = HashMap::new();
        let child_tag_regx =
            Regex::new(&format!(r",?\s?{}\(([a-zA-Z0-9]+)\)", TAG_WRAPPER)).unwrap();
        let parent_tag_regex =
            Regex::new(&format!(r",?\s?{}\(([a-zA-Z0-9]+)\)", TAG_PARENT_WRAPPER)).unwrap();

        let child_tags: Vec<String> = child_tag_regx
            .find_iter(line)
            .map(|m| {
                child_tag_regx
                    .captures(m.as_str())
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string()
            })
            .collect();
        let parent_tags: Vec<String> = parent_tag_regex
            .find_iter(line)
            .map(|m| {
                parent_tag_regex
                    .captures(m.as_str())
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string()
            })
            .collect();
        for c in &parent_tags {
            for p in &child_tags {
                if !output.contains_key(c) {
                    output.insert(c.to_string(), HashSet::new());
                }
                output
                    .get_mut(&c.to_string())
                    .unwrap()
                    .insert(p.to_string());
            }
        }
        return output;
    }

    #[test]
    fn test_tag_parser() {
        assert!(parse_tag_children("NOUN(Steel), TAG(Metal), TAG(Ferrous)")
            .len()
            .eq(&0));
        let subject_with_parent =
            parse_tag_children("TAG(Wood), HAS_PARENT(Plant), HAS_PARENT(Material)");
        assert!(subject_with_parent.len().eq(&2));
        assert!(subject_with_parent.get("Material").unwrap().len().eq(&1));
    }

    #[test]
    fn test_build_dictionary() {
        let dict = build_dictionary(vec![
            "TEMPLATE(NOUN[[Metal]] Bull Pub), TAG(Restaurant)".to_string(),
            "NOUN(Steel), TAG(Metal), TAG(Ferrous), TAG(Alloy)".to_string(),
            "NOUN(Oak), TAG(Tree)".to_string(),
            "NOUN(Pear), TAG(Tree), TAG(Fruit)".to_string(),
            "TAG(Metal), HAS_PARENT(Material)".to_string(),
            "TAG(Tree), HAS_PARENT(Wood), HAS_PARENT(Plant)".to_string(),
            "TAG(Wood), HAS_PARENT(Material)".to_string(),
            "TAG(Fruit), HAS_PARENT(Food)".to_string(),
            "TAG(Restaurant), HAS_PARENT(Institution)".to_string(),
        ]);
        assert!(dict.words.len().eq(&3));
        assert!(dict.index.tag_words.len().eq(&9));
        assert!(dict.templates.len().eq(&1));
    }

    #[test]
    fn test_random_word() {
        let dict = build_dictionary(vec![
            "NOUN(Steel), TAG(Metal), TAG(Ferrous), TAG(Alloy)".to_string(),
            "NOUN(Oak), TAG(Tree)".to_string(),
            "NOUN(Pear), TAG(Tree), TAG(Fruit)".to_string(),
            "TAG(Metal), HAS_PARENT(Material)".to_string(),
            "TAG(Tree), HAS_PARENT(Wood), HAS_PARENT(Plant)".to_string(),
            "TAG(Wood), HAS_PARENT(Material)".to_string(),
            "TAG(Fruit), HAS_PARENT(Food)".to_string(),
        ]);
        assert!(dict
            .get_random_word((
                WordType::Noun,
                vec![vec!["Wood".to_string()], vec!["Fruit".to_string()]]
            ))
            .unwrap()
            .base
            .eq(&"Pear"));
    }

    #[test]
    fn test_random_template() {
        let dict = build_dictionary(vec![
            "TEMPLATE(NOUN[[Metal]] Bull Pub), TAG(Restaurant)".to_string(),
            "NOUN(Steel), TAG(Metal), TAG(Ferrous), TAG(Alloy)".to_string(),
            "NOUN(Oak), TAG(Tree)".to_string(),
            "NOUN(Pear), TAG(Tree), TAG(Fruit)".to_string(),
            "TAG(Metal), HAS_PARENT(Material)".to_string(),
            "TAG(Tree), HAS_PARENT(Wood), HAS_PARENT(Plant)".to_string(),
            "TAG(Wood), HAS_PARENT(Material)".to_string(),
            "TAG(Fruit), HAS_PARENT(Food)".to_string(),
            "TAG(Restaurant), HAS_PARENT(Institution)".to_string(),
        ]);
        let t = dict
            .get_random_template(vec![vec!["Restaurant".to_string()]])
            .unwrap();
        assert!(t.template.len().eq(&4));
        assert!(dict.render_template(&t.id).unwrap().eq("Steel Bull Pub"));
    }
}
