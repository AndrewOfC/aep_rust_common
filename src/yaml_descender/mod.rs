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
use crate::arrayparser::ArrayParser;
use std::string::String;
use yaml_rust::Yaml;
use regex::Regex;
mod write_completions;
mod yaml_descend_path;

const KEY_MATCH: usize = 1 ;
const PERIOD_MATCH: usize = 2 ;
const INDEX_MATCH: usize = 3 ;

pub struct YamlDescender {
    docs: Vec<Yaml>,
    ap: Box<dyn ArrayParser>,
    root: Yaml,
    re: Regex,
    parent_key: Yaml,
    description_key: Yaml,
    terminal_fields: HashSet<Yaml>
}

fn get_string_set(yaml: Yaml) -> Result<HashSet<Yaml>, String> {
    match yaml {
        Yaml::Array(a) => { Ok(HashSet::from_iter(a.iter().map(|y| y.clone())))  }
        _ => Err(String::from("not an array"))
    }
}

