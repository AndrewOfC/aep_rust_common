use std::env;

pub fn find_config_file(arg0: &str, env_var: &str) -> Result<String, String> {
    let home = env::var("HOME").unwrap_or("".to_string());
    let default_path = format!(".:{home}/.config/{arg0}:/etc/{arg0}");
    let path = env::var(env_var).unwrap_or(default_path);
    let paths: Vec<&str> = path.split(':').collect();
    let target = format!("{}.yaml", arg0) ;

    for path in paths {
        let file_path = format!("{}/{}", path, target);
        if std::path::Path::new(&file_path).exists() {
            return Ok(file_path);
        }
    }
    Err(format!("no config file found for '{}'", arg0))
}
