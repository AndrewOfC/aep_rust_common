use lazy_static::lazy_static;
use regex::Regex;
use std::thread::current;
use yaml_rust::Yaml;

const WHOLE_MATCH: usize = 0;
const KEY_MATCH: usize = 1;
const PERIOD_MATCH: usize = 2;
const INDEX_MATCH: usize = 3;
const ARRAY_MATCH: usize = 4;

lazy_static! {
    static ref RE: Regex = Regex::new(r"([^.\[\]\\@]+)(\.)?|(?:@(\d+))?").unwrap();
    static ref ParentKey : Yaml = Yaml::String("parent".to_string());
}

///
/// Extract a value from a yaml tree given a 'path'
/// '.' will separate hash members
/// '@n' where n is an index into a list
///
pub fn yaml_path(yaml: &Yaml, path: &str) -> Result<Yaml, String> {
    let mut current = yaml;
    for captures in RE.captures_iter(path) {
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
                _ => return Err(format!("{} in {} is not a hash", key, path)),
            } // if str
        } else if captures.get(INDEX_MATCH).is_some() {
            let index = captures
                .get(INDEX_MATCH)
                .unwrap()
                .as_str()
                .parse::<usize>()
                .unwrap();
            match current {
                Yaml::Array(a) => {
                    if index >= a.len() {
                        return Err(format!("{} is out of bounds in {}", index, path));
                    }
                    current = &a[index];
                }
                _ => return Err(format!("{} {} is not an array", path, index)),
            }
        }
        // if index
        else {
            return Err(format!("{} is not a valid path", path));
        }
    }
    Ok(current.clone())
}

pub fn yaml_path_field(yaml: &Yaml, path: &str, field: &str) -> Result<Yaml, String> {
    let y = yaml_path(yaml, path);
    match y {
        Ok(y) => match y {
            Yaml::Hash(h) => {
                let ykey = Yaml::String(field.to_string());
                if !h.contains_key(&ykey) {
                    return Err(format!("{} not found in {}", field, path));
                }
                Ok(h[&ykey].clone())
            }
            _ => Err(format!("{} is not a hash", path)),
        },
        Err(e) => Err(e),
    }
}

pub fn yaml_field_parent(root: &Yaml, yaml: &Yaml, field: &str) -> Result<Yaml, String> {
    match yaml {
        Yaml::Hash(h) => {
            let ykey = Yaml::String(field.to_string());
            if !h.contains_key(&ykey) {
                if h.contains_key(&ParentKey) {
                    let parent_path = match &h[&ParentKey] {
                        Yaml::String(s) => s.to_string(),
                        _ => return Err(format!("{:?} is not a string", &h[&ParentKey])),
                    } ;

                    let parent_yaml = match yaml_path(root, &parent_path) {
                        Ok(y) => y,
                        Err(e) => return Err(e),
                    } ;
                    return yaml_field_parent(root, &parent_yaml, field) ;
                }
                return Err(format!("{} not found", field)) ;
            }
            Ok(h[&ykey].clone())
        }
        _ => Err(format!("{:?} is not a hash", yaml)),
    }
}

/// extract a value in the rustiest way possible
#[macro_export]
macro_rules! yaml_scalar {
    ($yaml:expr, $path:expr, f64) => {{
        match yaml_path($yaml, $path) {
            Ok(y) => match y.as_f64() {
                Some(v) => Ok(v) as Result<f64, String>,
                None => Err(format!("{} is not a float", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, $field:expr, f64) => {{
        match yaml_path_field($yaml, $path, $field) {
            Ok(y) => match y.as_f64() {
                Some(v) => Ok(v) as Result<f64, String>,
                None => Err(format!("{} is not a float", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, i64) => {{
        match yaml_path($yaml, $path) {
            Ok(y) => match y.as_i64() {
                Some(v) => Ok(v) as Result<i64, String>,
                None => Err(format!("{} is not an integer", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, $field:expr, i64) => {{
        match yaml_path_field($yaml, $path, $field) {
            Ok(y) => match y.as_i64() {
                Some(v) => Ok(v) as Result<i64, String>,
                None => Err(format!("{} is not an integer", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, bool) => {{
        match yaml_path($yaml, $path) {
            Ok(y) => match y.as_bool() {
                Some(v) => Ok(v) as Result<bool, String>,
                None => Err(format!("{} is not a bool", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, $field:expr, bool) => {{
        match yaml_path_field($yaml, $path, $field) {
            Ok(y) => match y.as_bool() {
                Some(v) => Ok(v) as Result<bool, String>,
                None => Err(format!("{} is not a bool", $path)),
            },
            Err(e) => Err(e),
        }
    }};
    ($yaml:expr, $path:expr, String) => {{
        match yaml_path($yaml, $path) {
            Ok(y) => match y.as_str() {
                Some(v) => Ok(v.to_string()) as Result<String, String>,
                None => Err(format!("{} is not a string", $path)),
            },
            Err(e) => Err(e),
        }
    }};
        ($yaml:expr, $path:expr, $field:expr, String) => {{
        match yaml_path_field($yaml, $path, $field) {
            Ok(y) => match y.as_str() {
                Some(v) => Ok(v.to_string()) as Result<String, String>,
                None => Err(format!("{} is not a string", $path)),
            },
            Err(e) => Err(e),
        }
    }};

}
