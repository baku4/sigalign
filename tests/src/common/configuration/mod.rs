use std::path::PathBuf;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use crate::common::directory_path::get_root_dir;

fn config_dir_path() -> Result<PathBuf> {
    let mut path = get_root_dir()?;
    path.push("config");
    Ok(path)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestSetting {
    // satisfy_cutoff
    pub satisfy_cutoff: SatisfyCutoff,
    // validation_with_033_and_dpm
    pub val_with_033_and_dpm: ValidationWith033AndDpm,
}
impl TestSetting {
    pub fn from_env() -> Result<Self> {
        let env = TestEnvironment::from_env()?;
        let config_file = env.get_config_file_path()?;
        Self::from_config_file(&config_file)
    }
    pub fn from_env_str(env: &str) -> Result<Self> {
        let env = TestEnvironment::from_str(env)?;
        let config_file = env.get_config_file_path()?;
        Self::from_config_file(&config_file)
    }
    fn from_config_file(config_file: &PathBuf) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(
                config::File::from(config_file.as_ref())
                    .required(true)
                    .format(config::FileFormat::Yaml)
            )
            .add_source(
                config::Environment::with_prefix("TEST")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;
        Ok(settings.try_deserialize::<Self>()?)
    }
}

enum TestEnvironment {
    Local,
    CI,
}
impl TestEnvironment {
    fn from_env() -> Result<Self> {
        let env = std::env::var("TEST_ENVIRONMENT")
            .unwrap_or("local".to_string());
        Self::from_str(&env)
    }
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "local" => Ok(TestEnvironment::Local),
            "ci" => Ok(TestEnvironment::CI),
            _ => bail!("Unknown environment: {}", s),
        }
    }
    fn get_config_file_path(&self) -> Result<PathBuf> {
        let mut path = config_dir_path()?;
        match self {
            TestEnvironment::Local => path.push("local.yaml"),
            TestEnvironment::CI => path.push("ci.yaml"),
        }
        Ok(path)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SatisfyCutoff {
    pub seed_start: u64,
    pub seed_count: u64,
    pub query_interval: u32,
    pub max_subst_percent: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidationWith033AndDpm {
    pub seed_start: u64,
    pub seed_count: u64,
    pub max_subst_percent: u32,
}
