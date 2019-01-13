use pipewerk_runner::config;
use std::fs;

fn main() {
    let f = fs::read_to_string("first-pipewerk.yml").unwrap();
    let jobs: config::Config = serde_yaml::from_str(&f).unwrap();
    println!("Hello, world!; {:?}", jobs);
}
