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
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;
use std::collections::HashSet;


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