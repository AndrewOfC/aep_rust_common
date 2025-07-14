use crate::yaml_descender::YamlDescender;
use std::collections::HashSet;
use std::string::String;
use yaml_rust::Yaml;

pub struct Metadata {
    pub root: Yaml,
    pub terminus: Yaml,
    pub ignore_fields: HashSet<String>,
    pub get_descriptions: bool,
    
    has_terminus: bool
}

impl Metadata {
    fn new(root: Yaml, terminus: Yaml, ignores: Vec<&Yaml>, get_descriptions: bool) -> Self {
        let h: HashSet<String> = ignores.into_iter()
            .map(|y| y.as_str().unwrap().to_string())
            .collect();
        let has_terminus = terminus.as_str().is_some();
        Metadata {
            root: root,
            terminus: terminus,
            ignore_fields: h,
            get_descriptions: get_descriptions,
            has_terminus: has_terminus,
        }
    }
    
    fn default() -> Self {
        let root = Yaml::BadValue;
        let terminus = Yaml::BadValue;
        Metadata::new(root, terminus, Vec::new(), false)
    }
    
    pub fn has_root(&self) -> bool {
        self.root.as_str().is_some()
    }
    
    pub fn has_terminus(&self) -> bool {
        self.has_terminus
    }
    pub fn has_terminal_field(&self, yaml: &Yaml) -> bool {
        self.has_terminus && match yaml {
            Yaml::Hash(h) => {
                 h.contains_key(&self.terminus)
            },
            _ => { false }
        }
    }
}

impl YamlDescender {
    
    pub fn get_metadata(&self, doc: &Yaml, get_descriptions: bool) -> Metadata {
        let metadata_r = self.get_field_or_parent(doc,"completion-metadata") ;
        let metadata = match metadata_r {
            Ok(y) => y,
            Err(_) => return Metadata::default()
        } ;

        let root = self.get_field_or_parent(&metadata, "root").unwrap_or_else(|_| Yaml::BadValue);

        let terminus = self.get_field_or_parent(&metadata, "root").unwrap_or_else(|_| Yaml::BadValue);


        // let ignore_fields = match self.get_field_or_parent(&metadata, "ignore_fields") {
        //     Ok(y) => {
        //         match y {
        //             Yaml::Array(arr) => arr.iter().collect(),
        //             _ => Vec::new()
        //         }
        //     },
        //     Err(_) => Vec::new()
        // };
        Metadata::new(root.clone(), terminus.clone(),
                      Vec::new(), get_descriptions)
    }
}