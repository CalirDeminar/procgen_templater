use std::{
    fs::{self, File},
    io::{self, BufRead},
};

use dictionary::dictionary::{build_dictionary, build_dictionary_from_folder, Dictionary};

pub mod dictionary;
pub mod people;

fn read_data_files() -> Vec<String> {
    let path_root = "./src/dictionary/data_files";
    let paths = fs::read_dir(path_root).unwrap();
    let mut output: Vec<String> = Vec::new();
    for path in paths {
        let filename = path.unwrap().file_name();
        let data = File::open(&format!("{}/{}", path_root, filename.to_str().unwrap()))
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
    return output;
}

pub fn build_default_dictionary() -> Dictionary {
    return build_dictionary(read_data_files());
}

fn main() {
    build_dictionary_from_folder("./src/dictionary/data_files");
}
