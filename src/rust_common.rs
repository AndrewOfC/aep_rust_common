use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;
use std::collections::HashSet;

pub(crate) const KEY_MATCH: usize = 1 ;
pub(crate) const INDEX_MATCH: usize = 3 ;

#[macro_export] macro_rules! yaml_scalar {

    ($d:expr, $path:expr, i64) => {
        {
            let yaml = $d.yaml_descend_path($path).unwrap().clone();
            yaml.as_i64().unwrap()
        }
    } ;
    ($d:expr, $path:expr, &str) => {
        {
            let yaml = $d.yaml_descend_path($path).unwrap().clone();
            yaml.as_str().unwrap().to_string()
        }
    } ;
    ($d:expr, $path:expr, bool) => {
        {
            let yaml = $d.yaml_descend_path($path).unwrap().clone();
            yaml.as_bool().unwrap()
        }
    } ;
    ($d:expr, $path:expr, f64) => {
        {
            let yaml = $d.yaml_descend_path($path).unwrap().clone();
            yaml.as_f64().unwrap()
        }
    } ;
}

pub fn sep(y: &Yaml, empty_path: bool) -> &str {
    if empty_path { return "" ; }
    match y {
        Yaml::Hash(_) => { "." }
        Yaml::Array(_) => { "" }
        _ => { "" }
    }
}

pub fn keys_starting_with<'a>(prefix : &str, map : &'a Hash, ignores: &HashSet<String>) -> Vec<&'a Yaml> {
    let mut keys = Vec::new();

    for (key, _) in map {
        if let Some(key_str) = key.as_str() {
            if ignores.contains(&key_str.to_string()) {
                continue ;
            }
            if key_str.starts_with(prefix) {
                keys.push(key);
            }
        }
    }
    keys.sort();
    keys
}