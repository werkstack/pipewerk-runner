#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub ssh_keys: Vec<String>,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_image")]
    image: String,
    ssh_keys: Vec<String>,
    pub jobs: Vec<Job>,
}

fn default_image() -> String {
    "busybox".to_string()
}

impl Config {
    pub fn from_str(config_str: &str) -> Result<Config, String> {
        match Config::parse_yml(&config_str) {
            Ok(mut config) => {
                config.set_job_defaults();
                Ok(config)
            }
            Err(err) => Err(err),
        }
    }

    fn parse_yml(config_str: &str) -> Result<Config, String> {
        match serde_yaml::from_str(&config_str) {
            Ok(config) => Ok(config),
            Err(err) => Err(format!("{}", err)),
        }
    }

    pub fn set_job_defaults(&mut self) {
        let jobs = &self.jobs;

        let new_jobs: Vec<Job> = jobs
            .into_iter()
            .map(|job| job.clone_with_defaults(self))
            .collect();
        self.jobs = new_jobs;
    }
}

impl Job {
    pub fn clone_with_defaults(&self, config: &Config) -> Job {
        let mut job = self.clone();
        if job.image == "" {
            job.image = config.image.clone();
        }
        if job.ssh_keys.is_empty() {
            job.ssh_keys = config.ssh_keys.clone();
        }
        job
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_simple_config() {
        let simple_config = r#"
        jobs:
          - name: test
            commands:
              - cargo test
        "#;
        let config: Config = Config::from_str(&simple_config).unwrap();
        assert_eq!(config.image, "busybox");
        assert_eq!(config.jobs.len(), 1);
        let test_job = &config.jobs[0];
        assert_eq!(test_job.name, "test");
        assert_eq!(test_job.image, "busybox");
        assert_eq!(test_job.commands, vec!["cargo test"]);
    }
}
