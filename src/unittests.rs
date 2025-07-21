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
const SOURCE1: &str = r#"---
field1:
    field1a: value1
    field1b: value2

field2: foo

"# ;

mod u_tests {
    use crate::yaml_path::yaml_path_field;
    use crate::yaml_path::yaml_path;
    use crate::descender::Descender;
    use crate::find_config_file::find_config_file;
    use crate::strwriter::StrWriter;
    use crate::unittests::SOURCE1;
    use crate::yaml_descender::YamlDescender;
    use crate::{yaml_scalar};
    use std::io::{BufWriter, Write};
    use lazy_static::lazy_static;
    use yaml_rust::{Yaml, YamlLoader};

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

    lazy_static! {
        static ref YamlData: Yaml = YamlLoader::load_from_str(TEST_SOURCE).unwrap()[0].clone() ;
        static ref ZshDescender:YamlDescender =    YamlDescender::new_from_file("test_data.yaml", false).unwrap() ;
        static ref BashDescender: YamlDescender =  YamlDescender::new_from_file("test_data.yaml", true).unwrap() ;
    }

    fn input_output_check(d: &YamlDescender, input: &str, output: &str) {
        let mut result_buffer = StrWriter::new() ;
        d.write_completions(&mut result_buffer, &input, false).expect("write failed") ;
        let result_str = result_buffer.to_string().expect("write failed") ;
        assert_eq!(result_str, output);
    }

    #[test]
    fn test_empty() {
        let d  = &BashDescender ;
        let mut output = BufWriter::new(Vec::new());
        d.write_completions(&mut output, "", false).expect("write failed") ;
        let s = String::from_utf8(output.into_inner().unwrap()).unwrap();
        assert_eq!(s, "GPIO\narray\nlevel1\nlevel1b\nlevel1c\nulevel\nxlevel\n");
    }

    #[test]
    fn test_one_path() {
        let d  = Box::new(YamlDescender::new(SOURCE1, true).unwrap());
        let mut output = BufWriter::new(Vec::new());
        d.write_completions(&mut output, "f", false).expect("write failed") ;
        let s = String::from_utf8(output.into_inner().unwrap()).unwrap();
        assert_eq!(s, "field1\nfield2\n");
    }


    #[test]
    fn test_array1() {
        input_output_check(&BashDescender, "array", "array[0]\narray[1]\narray[2]\n") ;
        input_output_check(&ZshDescender, "array", "array@0\narray@1\narray@2\n") ;
    }
    #[test]
    #[ignore]
    fn test_array2() {
        input_output_check(&BashDescender, "array@2", "array@2@0\narray@2@1\narray@2@2\n");
    }

    #[test]
    fn test_array3() {
        input_output_check(&ZshDescender, "array@", "array@0\narray@1\narray@2\n") ;
        input_output_check(&BashDescender, "array[", "array[0]\narray[1]\narray[2]\n") ;
    }
    #[test]
    fn test_field_terminator() {
        input_output_check(&BashDescender, "level1.", "level1.level2\nlevel1.level2a\nlevel1.level2b\n") ;
    }

    #[test]
    fn test_level_drop() {
        input_output_check(&BashDescender, "level1.level2", "level1.level2\nlevel1.level2a\nlevel1.level2b\n") ;
        input_output_check(&BashDescender, "level1.level2a", "level1.level2a\n") ;
    }

    #[test]
    fn test_gpio() {
        input_output_check(&BashDescender, "G", "GPIO.pins\nGPIO.words\n") ;
    }

    #[test]
    fn test_gpio_p() {
        input_output_check(&ZshDescender, "GPIO.p", "GPIO.pins@0\nGPIO.pins@1\n") ;
        input_output_check(&BashDescender, "GPIO.p", "GPIO.pins[0]\nGPIO.pins[1]\n") ;
    }
    #[test]
    fn test_gpio_pin0() {
        input_output_check(&ZshDescender, "GPIO.pins@0.", "GPIO.pins@0.clear\nGPIO.pins@0.function\nGPIO.pins@0.level\nGPIO.pins@0.set\n") ;
        input_output_check(&BashDescender, "GPIO.pins[0].", "GPIO.pins[0].clear\nGPIO.pins[0].function\nGPIO.pins[0].level\nGPIO.pins[0].set\n") ;
    }

    #[test]
    fn test_descending() {

        input_output_check(&BashDescender, "ulev", "ulevel.level1.level2.level3\n") ;

        return ;
    }



    #[test]
    fn test_path() {

        let i = yaml_scalar!(&YamlData, "root.array@0.number", i64);
        assert_eq!(i, Ok(4)) ;
        let i2 = yaml_scalar!(&YamlData, "root.array@0", "number", i64);
        assert_eq!(i2, Ok(4)) ;

        let s = yaml_scalar!(&YamlData, "root.array@0.string", String) ;
        assert_eq!(s, Ok("str".to_string())) ;
        let s2 = yaml_scalar!(&YamlData, "root.array@0", "string", String) ;
        assert_eq!(s2, Ok("str".to_string())) ;

        let b = yaml_scalar!(&YamlData, "root.array@0.bool", bool) ;
        assert_eq!(b, Ok(true)) ;
        let b2 = yaml_scalar!(&YamlData, "root.array@0", "bool", bool) ;
        assert_eq!(b2, Ok(true)) ;

        let f = yaml_scalar!(&YamlData, "root.array@0.real", f64) ;
        assert_eq!(f, Ok(2.0)) ;
        let f2 = yaml_scalar!(&YamlData, "root.array@0", "real", f64) ;
        assert_eq!(f2, Ok(2.0)) ;
    }

    #[test]
    #[should_panic]
    fn test_safe_path_error() {
        let yaml =  &YamlData ;
        let f = yaml_scalar!(yaml, "foo.array@0.real", f64) ;
        assert_eq!(f.unwrap(), 2.0) ;
    }

    #[test]
    #[should_panic(expected = "number in root.array.number is not a hash")]
    fn test_badvalue_hash() {
        let yaml =  &YamlData ;
        let _i = yaml_scalar!(yaml, "root.array.number", i64).unwrap();
    }
    #[test]
    #[should_panic(expected = "root.array@0.number@1 1 is not an array")]
    fn test_badvalue_array() {
        let yaml = &YamlData ;
        let _i = yaml_scalar!(yaml, "root.array@0.number@1", i64).unwrap();
    }

    #[test]
    #[should_panic(expected = "numberX not found in root.array@0.numberX")]
    fn test_nosuch_member() {
        let _i = crate::yaml_scalar!(&YamlData, "root.array@0.numberX", i64).unwrap();
    }

    #[test]
    #[should_panic(expected = "100 is out of bounds in root.array@100.number")]
    fn test_nosuch_index() {
        let _i = crate::yaml_scalar!(&YamlData, "root.array@100.number", i64).unwrap();
    }

    #[test]
    fn test_parent_lookup() {
        let doccer = YamlDescender::new(TEST_SOURCE, true).unwrap();
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