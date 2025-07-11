use std::fmt::Debug;
use std::os::unix::raw::ino_t;
use std::str::FromStr;
use regex::Regex;
use yaml_rust::{Yaml, YamlLoader};


const KEY_MATCH: usize = 1 ;
const INDEX_MATCH: usize = 3 ;

pub struct YamlDoccer {
    docs: Vec<Yaml>
}

impl YamlDoccer {
    
    pub fn new(docstr: &str) -> YamlDoccer {
        let docs = YamlLoader::load_from_str(docstr).expect("Failed to parse YAML");
        YamlDoccer { docs }
    }

    pub fn yaml_descend_path(&self, path: &str) -> Result<&Yaml, String> {
        let re = Regex::new(r"([^.\[\]\\]+)(\.)?|(?:\[(\d+)]?)?").unwrap();
        let mut current = &self.docs[0];
        for captures in re.captures_iter(path) {
            if captures.get(KEY_MATCH).is_some() {
                let key = captures.get(KEY_MATCH).unwrap().as_str();
                match current {
                    Yaml::Hash(h) => {
                        current = &h[&Yaml::String(key.to_string())];
                    }
                    _ => return Err(format!("{} {} is not a hash", path, key))
                } // if str
            } else if captures.get(INDEX_MATCH).is_some() {
                let index = captures.get(INDEX_MATCH).unwrap().as_str().parse::<usize>().unwrap();
                match current {
                    Yaml::Array(a) => {
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



    pub fn yaml_descend_value<T: std::str::FromStr>(&self, path:&str) -> Result<T, String>  {
        let yaml = self.yaml_descend_path(path).unwrap().clone();
        match yaml {
            Yaml::Integer(i) => Ok(i.to_string().parse::<T>().unwrap()),
            Yaml::Real(r) => Ok(r.parse::<T>().unwrap()),
            Yaml::String(s) => Ok(s.parse::<T>().unwrap()),
            _ => Err(format!("{} is not a valid type", path))
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let src = r"---
        root:
            key:
                - subkey: str
                  number: 4

" ;
        let doccer = YamlDoccer::new(src) ;

        let value : String = doccer.yaml_descend_value("root.key[0].subkey").unwrap() ;
        assert_eq!(value, "str") ;
        let value: i64 = doccer.yaml_descend_value("root.key[0].number").unwrap() ;
        assert_eq!(value, 4) ;

    }
}

