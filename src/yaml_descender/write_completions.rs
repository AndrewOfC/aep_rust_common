use std::io::Write;
use yaml_rust::Yaml;
use crate::descender::Descender;
use crate::rust_common::{keys_starting_with, sep};
use crate::yaml_descender::{YamlDescender, INDEX_MATCH, KEY_MATCH, PERIOD_MATCH};

impl Descender<dyn Write> for YamlDescender {

    fn set_root(&mut self, path: &str) -> Result<String, String> {
        let old_root = self.root.as_str().unwrap().to_string();
        self.root = Yaml::String(path.to_string());
        Ok(old_root)
    }

    fn get_string_field_or_parent(&self, path: &str, field: &str) -> Result<String, String> {
        let child = match self.yaml_descend_path(path) {
            Ok(yaml) => yaml,
            Err(e) => return Err(e)
        } ;
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
        let child = match self.yaml_descend_path(path) {
            Ok(yaml) => yaml,
            Err(_) => return Err(format!("{}.{} not found", path, field))
        };
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

    fn write_completions(&self, writer: &mut dyn Write, ipath: &str, add_descriptions: bool) -> std::io::Result<()>
    {
        let mut path = ipath ;
        let doc = &self.docs[0] ;
        let re = &self.ap.get_re() ;
        let mut current_path = String::from("") ;
        let mut empty_path = true ;
        let mut match_iter = re.captures_iter(path).peekable();
        let ap = &self.ap ;

        let mut current = if self.root.as_str().unwrap() == "" { doc } else { &doc.as_hash().unwrap()[&self.root] } ;


        while let Some(captures) = match_iter.next() {
            let last = match_iter.peek().is_none();
            let mut key = captures.get(KEY_MATCH).map_or("", |m| m.as_str()) ;
            let index = captures.get(INDEX_MATCH).map_or(0, |m| m.as_str().parse::<usize>().unwrap()) ;
            let mut terminated = captures.get(PERIOD_MATCH).is_some() ;

            loop {
                match current {
                    Yaml::Hash(hash) => {
                        if self.has_terminal_field(current) {
                            break ;
                        }
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

                        let keys = keys_starting_with(&key, hash, &Default::default());
                        if keys.is_empty() { return Ok(()); }

                        if keys.len() == 1 {
                            let ykey = keys[0];
                            if hash.contains_key(ykey) {
                                current = &hash[ykey];
                                current_path += ykey.as_str().unwrap();
                                empty_path = false;
                                if !self.has_terminal_field(current) {
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

                        let mut has_descriptions = add_descriptions ;
                        let mut completions:Vec<String> = Vec::new() ;
                        let mut descriptions:Vec<String> = Vec::new() ;

                        for key in keys.iter().map(|k| k.as_str().unwrap()) { // |k| k.as_str().unwrap()
                            //writer.write_fmt(format_args!("{}{}\n", current_path, key))?;
                            completions.push(format_args!("{}{}", current_path, key).to_string());
                            if has_descriptions {
                                let description = self.get_description(&hash[&Yaml::String(key.to_string())]);
                                match description {
                                    Ok(d) => {
                                        descriptions.push(d);
                                    }
                                    Err(_) => {
                                        has_descriptions = false;
                                        continue ;
                                    }
                                }
                            } // has_descriptions
                        } // for
                        if has_descriptions {
                            println!("__descriptions__"); // tag for zsh completion function
                        }
                        if has_descriptions {
                            for (i, c) in completions.iter().enumerate() {
                                writer.write_fmt(format_args!("{}\n{}\n", c, descriptions[i]))?;
                            }
                        }
                        else {
                            for c in completions {
                                writer.write_fmt(format_args!("{}\n", c))?;
                            }
                        }
                        return Ok(());
                    }
                    Yaml::Array(array) => {
                        if captures.get(INDEX_MATCH).is_some() {
                            if index >= array.len() {
                                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Index out of bounds"));
                            }
                            current = &array[index];
                            current_path += &ap.apply_index(index) ;
                            if !self.has_terminal_field(current) {
                                current_path += sep(current, empty_path);
                            }
                            break;
                        }
                        if array.len() == 1 {
                            current_path += &ap.apply_index(0);
                            empty_path = false;
                            current = &array[0];
                            if !self.has_terminal_field(current) {
                                current_path += sep(current, empty_path);
                            }
                            break;
                        }
                        for (index, _) in array.iter().enumerate() {
                            let mut path2 = current_path.to_string();

                            if ap.array_ending(&path2) {
                                path2.truncate(path.len() - 1);
                            }
                            let index_str = &ap.apply_index(index);
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

    fn get_description(&self, yaml: &Yaml) -> Result<String, String> {
        match yaml {
            Yaml::Hash(h) => {
                if h.contains_key(&self.description_key) {
                    match h[&self.description_key].as_str() {
                        Some(s) => return Ok(s.to_string()),
                        None => {return Err("description is not a string".to_string()) ;}
                    } ;
                }
                if h.contains_key(&self.parent_key) {
                    let parent_path = h[&self.parent_key].as_str().unwrap();
                    let parent= match self.yaml_descend_path(parent_path) {
                        Ok(p) => p,
                        Err(e) => return Err(e)
                    } ;
                    return self.get_description(&parent);
                }
                Err("no description found".to_string())
            }
            _ => Err(format!("{:?} is not a hash", yaml))
        }

    }
}