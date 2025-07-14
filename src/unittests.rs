const SOURCE1: &str = r#"---
field1:
    field1a: value1
    field1b: value2

field2: foo

"# ;

mod u_tests {
    use crate::descender::Descender;
    use crate::find_config_file::find_config_file;
    use crate::strwriter::StrWriter;
    use crate::unittests::SOURCE1;
    use crate::yaml_descender::YamlDescender;
    use crate::yaml_scalar;
    use std::io::{BufWriter, Write};
    use yaml_rust::Yaml;

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

    fn get_descender() -> Box<dyn Descender<dyn Write>> {
        Box::new(YamlDescender::new_from_file("test_data.yaml"))
    }

    fn input_output_check(input: &str, output: &str) {
        let d = get_descender();
        let mut result_buffer = StrWriter::new() ;
        d.write_completions(&mut result_buffer, &input, false, false).expect("write failed") ;
        let result_str = result_buffer.to_string().expect("write failed") ;
        assert_eq!(result_str, output);
    }

    #[test]
    fn test_empty() {
        let d  = Box::new(YamlDescender::new(SOURCE1));
        let mut output = BufWriter::new(Vec::new());
        d.write_completions(&mut output, "", false, false).expect("write failed") ;
        let s = String::from_utf8(output.into_inner().unwrap()).unwrap();
        assert_eq!(s, "field1\nfield2\n");
    }

    #[test]
    fn test_one_path() {
        let d  = Box::new(YamlDescender::new(SOURCE1));
        let mut output = BufWriter::new(Vec::new());
        d.write_completions(&mut output, "f", false, false).expect("write failed") ;
        let s = String::from_utf8(output.into_inner().unwrap()).unwrap();
        assert_eq!(s, "field1\nfield2\n");
    }


    #[test]
    fn test_array1() {
        input_output_check("array", "array@0\narray@1\narray@2\n") ;
    }
    #[test]
    #[ignore]
    fn test_array2() {
        input_output_check("array@2", "array@2@0\narray@2@1\narray@2@2\n") ;
    }

    #[test]
    fn test_array3() {
        input_output_check("array[", "array@0\narray@1\narray@2\n") ;
    }
    #[test]
    fn test_field_terminator() {
        input_output_check("level1.", "level1.level2\nlevel1.level2a\nlevel1.level2b\n") ;
    }

    #[test]
    fn test_level_drop() {
        input_output_check("level1.level2", "level1.level2\nlevel1.level2a\nlevel1.level2b\n") ;
        input_output_check("level1.level2a", "level1.level2a\n") ;
    }

    #[test]
    fn test_gpio() {
        input_output_check("G", "GPIO.pins\nGPIO.words\n") ;
    }

    #[test]
    fn test_gpio_p() {
        input_output_check("GPIO.p", "GPIO.pins@0\nGPIO.pins@1\n") ;
    }
    #[test]
    fn test_gpio_pin0() {
        input_output_check("GPIO.pins@0.", "GPIO.pins@0.clear\nGPIO.pins@0.function\nGPIO.pins@0.level\nGPIO.pins@0.set\n") ;
    }

    #[test]
    fn test_descending() {

        input_output_check("ulev", "ulevel.level1.level2.level3\n") ;

        return ;
    }



    #[test]
    fn test_path() {

        let doccer = YamlDescender::new(TEST_SOURCE) ;

        let i = yaml_scalar!(doccer, "root.array@0.number", i64);
        assert_eq!(i, 4) ;
        let s = yaml_scalar!(doccer, "root.array@0.string", &str) ;
        assert_eq!(s, "str") ;
        let b = yaml_scalar!(doccer, "root.array@0.bool", bool) ;
        assert_eq!(b, true) ;
        let f = yaml_scalar!(doccer, "root.array@0.real", f64) ;
        assert_eq!(f, 2.0) ;
    }

    #[test]
    #[should_panic(expected = "number in root.array.number is not a hash")]
    fn test_badvalue_hash() {
        let doccer = YamlDescender::new(TEST_SOURCE);
        let _i = crate::yaml_scalar!(doccer, "root.array.number", &str);
    }
    #[test]
    #[should_panic(expected = "root.array@0.number@1 1 is not an array")]
    fn test_badvalue_array() {
        let doccer = YamlDescender::new(TEST_SOURCE);
        let _i = crate::yaml_scalar!(doccer, "root.array@0.number@1", &str);
    }

    #[test]
    #[should_panic(expected = "numberX not found in root.array@0.numberX")]
    fn test_nosuch_member() {
        let doccer = YamlDescender::new(TEST_SOURCE);
        let _i = crate::yaml_scalar!(doccer, "root.array@0.numberX", &str);
    }

    #[test]
    #[should_panic(expected = "100 is out of bounds in root.array@100.number")]
    fn test_nosuch_index() {
        let doccer = YamlDescender::new(TEST_SOURCE);
        let _i = crate::yaml_scalar!(doccer, "root.array@100.number", &str);
    }

    #[test]
    fn test_parent_lookup() {
        let doccer = YamlDescender::new(TEST_SOURCE);
        let child = doccer.yaml_descend_path("parent_test.child2").unwrap();
        let description = doccer.get_field_or_parent(child, "description") ;
        assert_eq!(description, Ok(Yaml::String("foo".to_string()))) ;
    }

    #[test]
    #[should_panic(expected = "no config file found for 'foo'")]
    fn test_config_file_not_found() {
        let _bogus = find_config_file("foo", "bar").unwrap() ;
    }

}