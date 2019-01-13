#[derive(Debug, Serialize, Deserialize)]
struct Job {
    name: String,
    image: Option<String>,
    commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    jobs: Vec<Job>,
}

impl Config {
    pub fn from_str(config_str: &str) -> Result<Config, String> {
        match serde_yaml::from_str(&config_str) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("{}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_simple_config() {
        let simple_config = "\n
        jobs:\n
          - name: test\n
            commands:\n
              - cargo test\n
        ";
        let config: Config = Config::from_str(&simple_config).unwrap();
        assert_eq!(config.jobs.len(), 1);
        let test_job = &config.jobs[0];
        assert_eq!(test_job.name, "test");
        assert_eq!(test_job.commands, vec!["cargo test"]);
    }
}
