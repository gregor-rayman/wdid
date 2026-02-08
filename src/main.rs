mod error;
mod paths;

fn main() {
    match paths::AppPaths::new() {
        Ok(paths) => {
            println!("Config dir: {:?}", paths.config_dir);
            println!("Data dir: {:?}", paths.data_dir);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
