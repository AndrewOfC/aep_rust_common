use yaml_rust::Yaml;


pub(crate) const KEY_MATCH: usize = 1 ;
pub(crate) const INDEX_MATCH: usize = 3 ;

macro_rules! yaml_scalar {

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

#[cfg(test)]
mod tests {
    use crate::yaml_doccer::YamlDoccer;
    use super::*;
    
    const TEST_SOURCE:&str = r"---
        root:
            array:
                - string: str
                  number: 4
                  bool: true
                  real: 2.0

        parent_test:
            parent:
                description: 'foo'
            child1:
                parent: parent_test.parent
            child2:
                parent: parent_test.child1
" ;

    #[test]
    fn test_path() {
        
        let doccer = YamlDoccer::new(TEST_SOURCE) ;

        let i = yaml_scalar!(doccer, "root.array[0].number", i64);
        assert_eq!(i, 4) ;
        let s = yaml_scalar!(doccer, "root.array[0].string", &str) ;
        assert_eq!(s, "str") ;
        let b = yaml_scalar!(doccer, "root.array[0].bool", bool) ;
        assert_eq!(b, true) ;
        let f = yaml_scalar!(doccer, "root.array[0].real", f64) ;
        assert_eq!(f, 2.0) ;
    }

    #[test]
    #[should_panic(expected = "number in root.array.number is not a hash")]
    fn test_badvalue_hash() {
        let doccer = YamlDoccer::new(TEST_SOURCE);
        let _i = yaml_scalar!(doccer, "root.array.number", &str);
    }
    #[test]
    #[should_panic(expected = "root.array[0].number[1] 1 is not an array")]
    fn test_badvalue_array() {
        let doccer = YamlDoccer::new(TEST_SOURCE);
        let _i = yaml_scalar!(doccer, "root.array[0].number[1]", &str);
    }

    #[test]
    #[should_panic(expected = "numberX not found in root.array[0].numberX")]
    fn test_nosuch_member() {
        let doccer = YamlDoccer::new(TEST_SOURCE);
        let _i = yaml_scalar!(doccer, "root.array[0].numberX", &str);
    }

    #[test]
    #[should_panic(expected = "100 is out of bounds in root.array[100].number")]
    fn test_nosuch_index() {
        let doccer = YamlDoccer::new(TEST_SOURCE);
        let _i = yaml_scalar!(doccer, "root.array[100].number", &str);
    }

    #[test]
    fn test_badvalue_hash2() {
        let doccer = YamlDoccer::new(TEST_SOURCE);
        let child = doccer.yaml_descend_path("parent_test.child2").unwrap();
        let description = doccer.get_field_or_parent(child, "description") ;
    }
}

