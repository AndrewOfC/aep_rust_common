use yaml_rust::{Yaml, YamlLoader};
use std::collections::HashSet;
use crate::arrayparser::{ArrayParser, BashArrayParser, ZshArrayParser};
use crate::yaml_descender;
use crate::yaml_descender::{YamlDescender, INDEX_MATCH, KEY_MATCH};
use crate::yaml_path::yaml_path;

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

    ///
    /// Create a descender from a string
    ///
    /// # Arguments
    ///   docstr:
    ///     string as yaml data
    ///   bash_or_zsh bool
    ///     - true - arrays are indexed with [index]
    ///     - false - arrays are indexed as @index
    ///
    pub fn new(docstr: &str, bash_or_zsh: bool) -> Result<YamlDescender, String> {
        let docs = match YamlLoader::load_from_str(docstr) {
            Ok(d) => d,
            Err(e) => return Err(format!("failed to parse yaml: {}", e))
        } ;

        let root = yaml_path(&docs[0], "completion-metadata.root").unwrap_or_else(|_| Yaml::String("".to_string()));

        let terminal_fields = match yaml_path(&docs[0], "completion-metadata.terminal-fields") {
            Ok(y) => match yaml_descender::get_string_set(y) {
                Ok(s) => s,
                Err(s) => {return Err(format!("terminal-field: {}", s))}
            }
            Err(_) => { HashSet::new() }
        } ;

        let parent_key = Self::get_parent_key();
        let ap = Self::get_ap(bash_or_zsh) ;

        Ok(YamlDescender {
            docs,
            re: ap.get_re(),
            parent_key,
            root,
            description_key: Self::get_description_key(),
            terminal_fields,
            ap
        })
    }

    ///
    /// Create a descender from a file
    ///
    pub fn new_from_file(path: &str, bash_or_zsh: bool) -> Result<YamlDescender, String> {
        let docstr = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) => return Err(format!("failed to read file: {}", e))
        } ;
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
                    terminal_fields: HashSet::new(),
                    ap: ap
                })
            }
            Yaml::Array(_a) => {
                Ok(YamlDescender { docs: vec![yaml.clone()],
                    re: ap.get_re(),
                    parent_key: Self::get_parent_key(),
                    root: Yaml::String("".to_string()),
                    description_key: Self::get_description_key(),
                    terminal_fields: HashSet::new(),
                    ap: ap
                },)
            }
            _ => { return Err(String::from("cannot create from scalar types"))}
        }
    }

    ///
    /// Given a path to an item consisting of key1.key2[0] into a YAML tree such as:
    /// return the item as a YAML entity.
    /// # Example
    /// ```rust
    /// use yaml_rust::Yaml;
    /// use aep_rust_common::yaml_descender::YamlDescender;
    /// let s = r"---
    ///  tree:
    ///    sub-array:
    ///            - one
    ///            - two
    ///            - three: 3
    ///              six: 6
    ///              nine: 9
    /// " ;
    ///   let descender = YamlDescender::new(&s, true).unwrap() ;
    ///   let x = match descender.yaml_descend_path("tree.sub-array[2].nine").unwrap() {
    ///      Yaml::Integer(i) => i,
    ///      _ => panic!("expected an int")
    ///   } ;
    ///   assert_eq!(*x, 9) ;
    ///
    /// ```
    ///
    pub fn yaml_descend_path(&self, path: &str) -> Result<&Yaml, String> {
        let re = &self.re ;
        let mut current = &self.docs[0];
        if path == "" {
            return Ok(current);
        }

        if self.root.as_str().unwrap_or("") != "" {
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

    ///
    /// In a YAML hash extract the given field.  If not found in the hash,
    /// if the hash has a 'parent' field, get the parent in the 'root'
    /// document specified by a path and check for the field there.
    /// This is done recursively.
    ///
    /// # Example
    /// ```rust
    /// let s = r"tree:
    ///   parent2:
    ///     key: value
    /// parent1:
    ///   parent: tree.parent2
    /// child:
    ///   parent: parent1" ;
    ///
    ///  use aep_rust_common::descender::Descender;
    /// use aep_rust_common::yaml_descender::YamlDescender;
    ///  let descender = YamlDescender::new(s, true).unwrap() ;
    ///  let s = descender.get_string_field_or_parent("child", "key").unwrap() ;
    ///  assert_eq!(s, "value") ;
    ///
    /// ```
    ///
    ///
    pub fn get_field_or_parent(&self, child: &Yaml, field: &str) -> Result<Yaml, String> {

        let h = match child {
            Yaml::Hash(h) => h,
            _ => return Err(format!("{} is not a hash", field))
        } ;
        let field_key = Yaml::String(String::from(field));

        // let keyvector = h.keys().collect::<Vec<&Yaml>>();
        if h.contains_key(&field_key) {
            return Ok(h[&field_key].clone());
        } else if h.contains_key(&self.parent_key) {
            let parent_path = match &h[&self.parent_key] {
                Yaml::String(s) => s.as_str(),
                _ => return Err("parent is not a string".parse().unwrap())
            } ;
            let parent= match self.yaml_descend_path(parent_path) {
                Ok(yaml) => yaml,
                Err(e) => return Err(e)
            } ;
            return self.get_field_or_parent(parent, field);
        }
        Err(format!("field {} not found", field))
    }

    pub(crate) fn has_terminal_field(&self, yaml: &Yaml) -> bool {
        let h = match yaml.as_hash() {
            Some(h) => h,
            None => return false
        } ;

        for k in h.keys() {
            if self.terminal_fields.contains(k) {
                return true;
            }
        }
        false
    }
}