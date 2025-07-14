use std::io::Write;

pub trait Descender<W: Write + ?Sized>: Send + Sync {

    fn set_root(&mut self, root: &str) -> Result<String, String>;

    fn get_string_field_or_parent(&self, path: &str, field: &str) -> Result<String, String>;
    fn get_int_field_or_parent(&self, path: &str, field: &str) -> Result<i64, String>;
    fn get_bool_field_or_parent(&self, path: &str, field: &str) -> Result<bool, String>;
    fn get_float_field_or_parent(&self, path: &str, field: &str) -> Result<f64, String>;

    fn write_completions(&self, writer: &mut dyn Write, ipath: &str, add_descriptions: bool, zsh_mode: bool) -> std::io::Result<()> ;
}

