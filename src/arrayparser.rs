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
use regex::Regex;

/// A trait for parsing an array-like string structure using regular expressions.
///
/// The `ArrayParser` trait provides a blueprint for implementing types that
/// can handle parsing or extracting data from array-like string structures
/// using regular expressions.
///
/// # Required Methods
///
/// - `get_re`: Returns a `Regex` instance to be used for parsing.
///
/// # Methods
///
/// ## `fn get_re() -> Regex;`
/// - **Description:**
///     This method returns a `Regex` object that can be used to handle
///     parsing array-like patterns in strings.
/// - **Return:**
///     - A `Regex` instance that represents the pattern for parsing array data.
/// - **Implementation:**
///     This has to be implemented by any type that implements the `ArrayParser` trait.
/// 
///
/// # Notes
/// - Users of this trait must ensure to handle the `regex` crate dependency in their project.
/// - Proper error handling is advised when dealing with complex regular expressions.
pub trait ArrayParser : Send + Sync {
    fn get_re(&self) -> Regex;
    fn apply_index(&self, index: usize) -> String;

    fn array_ending(&self, s: &str) -> bool ;
}

pub struct BashArrayParser {
}

impl BashArrayParser {
    pub fn new() -> Self {
        Self{}
    }
}

pub struct ZshArrayParser {
}

impl ZshArrayParser {
    pub fn new() -> Self {
        Self{}
    }
}

impl ArrayParser for BashArrayParser {
    fn get_re(&self) -> Regex {
        Regex::new(r"([^.\[\]\\]+)(\.)?|(?:\[(\d+)]?)?").unwrap()
    }

    fn apply_index(&self, index: usize) -> String {
        format!("[{}]", index)
    }

    fn array_ending(&self, s: &str) -> bool {
        s.ends_with('[')
    }
}

impl ArrayParser for ZshArrayParser {
    fn get_re(&self) -> Regex {
        Regex::new(r"([^.\[\]\\@]+)(\.)?|(?:@(\d+))?").unwrap()
    }
    fn apply_index(&self, index: usize) -> String {
        format!("@{}", index)
    }

    fn array_ending(&self, s: &str) -> bool {
        s.ends_with('@')
    }
}