use std::thread::current;
use regex::Regex;
use yaml_rust::Yaml;
use lazy_static::lazy_static;

const  WHOLE_MATCH: usize = 0 ;
const KEY_MATCH: usize = 1 ;
const PERIOD_MATCH: usize = 2 ;
const INDEX_MATCH: usize = 3 ;
const ARRAY_MATCH: usize = 4;

lazy_static! {
    static ref RE: Regex = Regex::new(r"([^.\[\]\\@]+)(\.)?|(?:@(\d+))?").unwrap();
}

///
/// Extract a value from a yaml tree given a 'path'
/// '.' will separate hash members
/// '@n' where n is an index into a list
///
pub fn yaml_path(yaml: &Yaml, path: &str) -> Result<Yaml, String>
{
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
        } 
    Ok(current.clone())
}

/// extract a value in the rustiest way possible
#[macro_export] macro_rules! yaml_scalar {
    ($yaml:expr, $path:expr, f64) => {
        {
           match yaml_path($yaml, $path) {
                Ok(y) => {
                    match y.as_f64() {
                        Some(v) => Ok(v) as Result<f64, String>,
                        None => Err(format!("{} is not a float", $path))
                    }

                }
                Err(e) => Err(e)
                }
        }
    } ;
    ($yaml:expr, $path:expr, i64) => {
        {
           match yaml_path($yaml, $path) {
                Ok(y) => {
                    match y.as_i64() {
                        Some(v) => Ok(v) as Result<i64, String>,
                        None => Err(format!("{} is not a float", $path))
                    }
                }
                Err(e) => Err(e)
                }
        }
    } ;
        ($yaml:expr, $path:expr, bool) => {
        {
           match yaml_path($yaml, $path) {
                Ok(y) => {
                    match y.as_bool() {
                        Some(v) => Ok(v) as Result<bool, String>,
                        None => Err(format!("{} is not a float", $path))
                    }

                }
                Err(e) => Err(e)
                }
           }
    } ;
        ($yaml:expr, $path:expr, String) => {
        {
           match yaml_path($yaml, $path) {
                Ok(y) => {
                    match y.as_str() {
                        Some(v) => Ok(v.to_string()) as Result<String, String>,
                        None => Err(format!("{} is not a float", $path))
                    }

                }
                Err(e) => Err(e) 
                }
        }
    } ;

}
