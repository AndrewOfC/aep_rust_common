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