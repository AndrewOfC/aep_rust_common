// 
// SPDX-License-Identifier: MIT
// 
// Copyright (c) 2025 Andrew Ellis Page
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
// 
use std::collections::HashSet;
use crate::arrayparser::{ArrayParser, ZshArrayParser};
use crate::arrayparser::BashArrayParser;
use std::io::Write;
use std::string::String;
use yaml_rust::{Yaml, YamlLoader};
use regex::Regex;
use crate::descender::Descender;
use crate::rust_common::{keys_starting_with, sep};
use crate::{yaml_path, yaml_scalar};
use crate::yaml_path::yaml_path;

const  WHOLE_MATCH: usize = 0 ;
const KEY_MATCH: usize = 1 ;
const PERIOD_MATCH: usize = 2 ;
const INDEX_MATCH: usize = 3 ;
const ARRAY_MATCH: usize = 4;

pub struct YamlDescender {
    docs: Vec<Yaml>,
    ap: Box<dyn ArrayParser>,
    root: Yaml,
    re: Regex,
    parent_key: Yaml,
    description_key: Yaml,
    terminal_field: HashSet<Yaml>
}

fn get_string_set(yaml: Yaml) -> Result<HashSet<Yaml>, String> {
    match yaml {
        Yaml::Array(a) => { Ok(HashSet::from_iter(a.iter().map(|y| y.clone())))  }
        _ => Err(String::from("not an array"))
    }
}

/// Descends into tree like objects such as yaml or (coming soon) json
impl YamlDescender {
    // traditional array accessor "([^.\[\]\\]+)(\.)?|(?:\[(\d+)]?)?"
    // more zsh friendly "([^.\[\]\\@]+)(\.)?|(?:@(\d+))?"

    fn get_ap(bash_or_zsh: bool) -> Box<dyn ArrayParser> {
        if bash_or_zsh {
            Box::new(BashArrayParser::new())
        }
        else {
            Box::new(ZshArrayParser::new())
        }
    }

    fn get_parent_key() -> Yaml {
        Yaml::String("parent".to_string())
    }

    fn get_description_key() -> Yaml {
        Yaml::String("description".to_string())
    }

    pub fn new(docstr: &str, bash_or_zsh: bool) -> Result<YamlDescender, String> {
        let docs = YamlLoader::load_from_str(docstr).expect("Failed to parse YAML");

        let root = match yaml_path(&docs[0], "completion-metadata.root") {
            Ok(y) => y,
            Err(_) => Yaml::String("".to_string())
        } ;

        let terminal_field = match yaml_path(&docs[0], "completion-metadata.terminal-fields") {
            Ok(y) => match get_string_set(y) {
                Ok(s) => s,
                Err(s) => {return Err(format!("terminal-field: {}", s))}
            }
            Err(_) => { HashSet::new() }
        } ;

        let parent_key = Self::get_parent_key();
        let ap = Self::get_ap(bash_or_zsh) ;

        Ok(YamlDescender {
            docs: docs,
            re: ap.get_re(),
            parent_key: parent_key,
            root: root,
            description_key: Self::get_description_key(),
            terminal_field: terminal_field,
            ap: ap
        })
    }

    pub fn new_from_file(path: &str, bash_or_zsh: bool) -> Result<YamlDescender, String> {
        let docstr = std::fs::read_to_string(path)
            .expect(format!("Failed to read file: {}", path).as_str());
        YamlDescender::new(&docstr, bash_or_zsh)
    }

    pub fn new_from_yaml(yaml: &Yaml, bash_or_zsh : bool ) -> Result<YamlDescender, String> {
        let ap = Self::get_ap(bash_or_zsh) ;
        match yaml {
            Yaml::Hash(_h) => {
                Ok(YamlDescender { docs: vec![yaml.clone()],
                    re: ap.get_re(),
                    parent_key: Self::get_parent_key(),
                    root: Yaml::String("".to_string()),
                    description_key: Self::get_description_key(),
                    terminal_field: HashSet::new(),
                    ap: ap
                })
            }
            Yaml::Array(_a) => {
                Ok(YamlDescender { docs: vec![yaml.clone()],
                    re: ap.get_re(),
                    parent_key: Self::get_parent_key(),
                    root: Yaml::String("".to_string()),
                    description_key: Self::get_description_key(),
                    terminal_field: HashSet::new(),
                    ap: ap
                },)
            }
            _ => { return Err(String::from("cannot create from scalar types"))}
        }
    }

    pub fn yaml_descend_path(&self, path: &str) -> Result<&Yaml, String> {
        let re = &self.re ;
        let mut current = &self.docs[0];
        if path == "" {
            return Ok(current);
        }

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
                            return Err(format!("{} not found", key));
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

    fn has_terminal_field(&self, yaml: &Yaml) -> bool {
        let h = match yaml.as_hash() {
            Some(h) => h,
            None => return false
        } ;

        for k in h.keys() {
            if self.terminal_field.contains(k) {
                return true;
            }
        }
        false
    }
}

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
            Err(e) => return Err(format!("{}.{} not found", path, field))
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
                                key = ykey.as_str().unwrap();
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

                        let sep = sep(current, empty_path);
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
                    let description : &str = match h[&self.description_key].as_str() {
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
                return Err("no description found".to_string());
            }
            _ => return Err(format!("{:?} is not a hash", yaml))
        }

    }
}
