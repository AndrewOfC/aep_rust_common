use yaml_rust::{Yaml, YamlLoader};
use regex::Regex;
use crate::rust_common::{INDEX_MATCH, KEY_MATCH};
use crate::yaml_descent;

pub struct YamlDoccer {
    docs: Vec<Yaml>,
    re: Regex,
    parent_key: Yaml,
}


impl YamlDoccer {

    pub fn new(docstr: &str) -> YamlDoccer {
        let docs = YamlLoader::load_from_str(docstr).expect("Failed to parse YAML");
        let re = Regex::new(r"([^.\[\]\\]+)(\.)?|(?:\[(\d+)]?)?").unwrap();
        let parent_key = Yaml::String(String::from("parent"));
        YamlDoccer { docs, re, parent_key }
    }

    pub fn new_from_file(path: &str) -> YamlDoccer {
        let docstr = std::fs::read_to_string(path)
            .expect(format!("Failed to read file: {}", path).as_str());
        YamlDoccer::new(&docstr)
    }

    pub fn yaml_descend_path(&self, path: &str) -> Result<&Yaml, String> {
        let re = &self.re ;
        let mut current = &self.docs[0];
        for captures in re.captures_iter(path) {
            if captures.get(KEY_MATCH).is_some() {
                let key = captures.get(KEY_MATCH).unwrap().as_str(); 
                match current {
                    Yaml::Hash(h) => {
                        let ykey = Yaml::String(key.to_string());
                        if !h.contains_key(&ykey) {
                            return Err(format!("{} not found in {}", key, path));
                        }
                        current = &h[&ykey];
                    }
                    _ => return Err(format!("{} in {} is not a hash", key, path))
                } // if str
            } else if captures.get(INDEX_MATCH).is_some() {
                let index = captures.get(INDEX_MATCH).unwrap().as_str().parse::<usize>().unwrap();
                match current {
                    Yaml::Array(a) => {
                        if index >= a.len() {
                            return Err(format!("{} is out of bounds in {}", index, path));
                        }
                        current = &a[index];
                    }
                    _ => return Err(format!("{} {} is not an array", path, index))
                }
            } // if index
            else {
                return Err(format!("{} is not a valid path", path))
            }
        } // for
        Ok(current)
    }

    pub fn get_field_or_parent(&self, child: &Yaml, field: &str) -> String {
        let doc= &self.docs[0] ;
        let h = child.as_hash().unwrap();
        let field_key = Yaml::String(String::from(field)); // todo move to metadata or other persistent struct

        // let keyvector = h.keys().collect::<Vec<&Yaml>>();
        if h.contains_key(&self.parent_key) {
            let parent_path = h[&self.parent_key].as_str().unwrap();
            let parent= self.yaml_descend_path(parent_path).unwrap() ;
            return self.get_field_or_parent(parent, field);
        }

        if !h.contains_key(&field_key) {
            return String::from("");
        }
        return h[&field_key].as_str().unwrap().to_string()
    }
}