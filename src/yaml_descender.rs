use std::io::Write;
use std::string::String;
use yaml_rust::{Yaml, YamlLoader};
use regex::Regex;
use crate::descender::Descender;
use crate::rust_common::{keys_starting_with, sep};

const  WHOLE_MATCH: usize = 0 ;
const KEY_MATCH: usize = 1 ;
const PERIOD_MATCH: usize = 2 ;
const INDEX_MATCH: usize = 3 ;
const ARRAY_MATCH: usize = 4;

pub struct YamlDescender {
    docs: Vec<Yaml>,
    root: Yaml,
    re: Regex,
    parent_key: Yaml,
}

/// Descends into tree like objects such as yaml or (coming soon) json
impl YamlDescender {
    // traditional array accessor "([^.\[\]\\]+)(\.)?|(?:\[(\d+)]?)?"
    // more zsh friendly "([^.\[\]\\-]+)(\.)?|(?:-(\d+))?"gm
    pub fn get_re() -> Regex {
        Regex::new(r"([^.\[\]\\-]+)(\.)?|(?:-(\d+))?").unwrap()
    }

    fn get_parent_key() -> Yaml {
        Yaml::String("parent".to_string())
    }

    pub fn new(docstr: &str) -> YamlDescender {
        let docs = YamlLoader::load_from_str(docstr).expect("Failed to parse YAML");
        let re = YamlDescender::get_re() ;
        let parent_key = Self::get_parent_key();
        YamlDescender { docs, re, parent_key, root: Yaml::String("".to_string()) }
    }

    pub fn new_from_file(path: &str) -> YamlDescender {
        let docstr = std::fs::read_to_string(path)
            .expect(format!("Failed to read file: {}", path).as_str());
        YamlDescender::new(&docstr)
    }

    fn new_from_yaml(yaml: &Yaml) -> Result<YamlDescender, String> {
        match yaml {
            Yaml::Hash(_h) => {
                Ok(YamlDescender { docs: vec![yaml.clone()],
                    re: Self::get_re(), parent_key: Self::get_parent_key(), root: Yaml::String("".to_string()) })
            }
            Yaml::Array(_a) => {
                Ok(YamlDescender { docs: vec![yaml.clone()],
                    re: Self::get_re(), parent_key: Self::get_parent_key(), root: Yaml::String("".to_string()) })
            }
            _ => { return Err(String::from("cannot create from scalar types"))}
        }

    }

    pub fn yaml_descend_path(&self, path: &str) -> Result<&Yaml, String> {
        let re = &self.re ;
        let mut current = &self.docs[0];

        if self.root.as_str().unwrap() != "" {
            match current {
                Yaml::Hash(h) => {
                    let ykey = Yaml::String(self.root.as_str().unwrap().to_string());
                    if !h.contains_key(&ykey) {
                        return Err(format!("{} not found in {}", self.root.as_str().unwrap(), path));
                    }
                    current = &h[&ykey];
                }
                _ => return Err(format!("{} in {} is not a hash", self.root.as_str().unwrap(), path))
            }
        }

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

    pub fn get_field_or_parent(&self, child: &Yaml, field: &str) -> Result<Yaml, String> {

        let h = match child {
            Yaml::Hash(h) => h,
            _ => return Err(format!("{} is not a hash", field))
        } ;
        let field_key = Yaml::String(String::from(field)); // todo move to metadata or other persistent struct

        
        // let keyvector = h.keys().collect::<Vec<&Yaml>>();

        if h.contains_key(&field_key) {
            return Ok(h[&field_key].clone());
        } else if h.contains_key(&self.parent_key) {
            let parent_path = h[&self.parent_key].as_str().unwrap();
            let parent= self.yaml_descend_path(parent_path).unwrap() ;
            return self.get_field_or_parent(parent, field);
        }
        Err(format!("field {} not found", field))
    }
}

impl Descender<dyn Write> for YamlDescender {

    fn set_root(&mut self, path: &str) {
        self.root = Yaml::String(path.to_string());
    }

    fn get_string_field_or_parent(&self, path: &str, field: &str) -> Result<String, String> {
        let child = self.yaml_descend_path(path).unwrap();
        let value = self.get_field_or_parent(&child, field) ;
        match value {
            Ok(v) => match v {
                Yaml::String(s) => Ok(s),
                _ => Err(format!("{}.{} is not a string", path, field))
            }
            _ => Err(format!("{}.{} not found", path, field))
        }
    }

    fn get_int_field_or_parent(&self, path: &str, field: &str) -> Result<i64, String> {
        let child = self.yaml_descend_path(path).unwrap();
        let value_r = self.get_field_or_parent(&child, field) ;
        match value_r {
            Ok(v) => match v {
                Yaml::Integer(i) => Ok(i),
                _ => Err(format!("{}.{} is not an integer", path, field))
            }
            Err(e) => Err(e)
        }
    }

    fn get_bool_field_or_parent(&self, path: &str, field: &str) -> Result<bool, String> {
        let child = self.yaml_descend_path(path).unwrap();
        let value_r = self.get_field_or_parent(&child, field) ;
         match value_r {
            Ok(v) => match v {
                Yaml::Boolean(b) => Ok(b),
                _ => Err(format!("{}.{} is not a bool", path, field))
            }
            Err(e) => Err(e)
        }
    }

    fn get_float_field_or_parent(&self, path: &str, field: &str) -> Result<f64, String> {
        let child = self.yaml_descend_path(path).unwrap();
        let value_r = self.get_field_or_parent(&child, field) ;

        match value_r {
            Ok(v) => match v {
                Yaml::Real(r) => match r.parse::<f64>() {
                    Ok(f) => Ok(f),
                    Err(_) => Err(format!("{}.{} contains invalid float value", path, field))
                },
                _ => Err(format!("{}.{} is not a float", path, field))
            }
            Err(e) => Err(e)
        }

    }
/*
    fn get_child(self, path: &str) -> Result<Self, String>
    where
        Self: Sized
    {
        todo!()
    }*/



    fn write_completions(&self, writer: &mut dyn Write, ipath: &str, add_descriptions: bool, zsh_mode: bool) -> std::io::Result<()>
    {
        let mut path = ipath ;
        let doc = &self.docs[0] ;
        let re = YamlDescender::get_re() ;
        let metadata = self.get_metadata(&self.docs[0], add_descriptions) ;
        let mut current = if !metadata.has_root() { doc } else { &doc.as_hash().unwrap()[&metadata.root] } ;
        let mut current_path = String::from("") ;
        let mut empty_path = true ;
        let mut match_iter = re.captures_iter(path).peekable();

        while let Some(captures) = match_iter.next() {
            let last = match_iter.peek().is_none();
            let mut key = captures.get(KEY_MATCH).map_or("", |m| m.as_str()) ;
            let index = captures.get(INDEX_MATCH).map_or(0, |m| m.as_str().parse::<usize>().unwrap()) ;
            let mut terminated = captures.get(PERIOD_MATCH).is_some() ;

            loop {
                match current {
                    Yaml::Hash(hash) => {
                        let ykey = Yaml::String(key.to_string());
                        // let key_vector = hash.iter().map(|(k,v)|k.as_str().unwrap() ).collect::<Vec<&str>>();

                        if terminated {
                            // no need to search for members starting with key
                            if !hash.contains_key(&ykey) { return Ok(()); }
                            current = &hash[&ykey]; // next

                            current_path += ykey.as_str().unwrap();
                            empty_path = false;
                            current_path += sep(current, empty_path);
                            if !last { break; }
                            path = "";
                            key = "";
                            terminated = false;
                            continue;
                        }

                        let keys = keys_starting_with(&key, hash, &metadata.ignore_fields);
                        if keys.is_empty() { return Ok(()); }

                        if keys.len() == 1 {
                            let ykey = keys[0];
                            if hash.contains_key(ykey) {
                                current = &hash[ykey];
                                key = ykey.as_str().unwrap();
                                current_path += ykey.as_str().unwrap();
                                empty_path = false;
                                if !metadata.has_terminal_field(current) {
                                    current_path += sep(current, empty_path);
                                }
                                if !last { break; }
                                key = "";
                                path = "";
                                continue;
                            }
                        }

                        if hash.len() == 1 {
                            let key = hash.keys().next().unwrap();
                            current = &hash[key];
                            current_path += key.as_str().unwrap();
                            empty_path = false;
                            current_path += sep(current, empty_path);
                            continue;
                        }

                        let sep = sep(current, empty_path);
                        for key in keys.iter().map(|k| k.as_str().unwrap()) {
                            writer.write_fmt(format_args!("{}{}\n", current_path, key))?;
                        }
                        return Ok(());
                    }
                    Yaml::Array(array) => {
                        if captures.get(INDEX_MATCH).is_some() {
                            if index >= array.len() {
                                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Index out of bounds"));
                            }
                            current = &array[index];
                            current_path += format!("-{}", index).as_str();
                            if !metadata.has_terminal_field(current) {
                                current_path += sep(current, empty_path);
                            }
                            break;
                        }
                        if array.len() == 1 {
                            current_path += "-0";
                            empty_path = false;
                            current = &array[0];
                            if !metadata.has_terminal_field(current) {
                                current_path += sep(current, empty_path);
                            }
                            break;
                        }
                        for (index, _) in array.iter().enumerate() {
                            let mut path2 = current_path.to_string();

                            if path2.ends_with("-") {
                                path2.truncate(path.len() - 1);
                            }
                            let index_str = format!("-{}", index);
                            writer.write_fmt(format_args!("{}{}\n", path2, index_str))?;
                        }
                        return Ok(());
                    }
                    _ => { break; }
                }
            } // while true
        } // for captures
        writer.write_fmt(format_args!("{}\n", current_path))?;
        Ok(())
    }


}
