use std::fs::{read_dir, remove_dir_all};
use std::sync::Arc;

use async_trait::async_trait;
use clap::crate_version;
use semver::Version;

use huber_common::config::Config;
use huber_common::di::DIContainer;
use huber_common::model::package::{Package, PackageSource};
use huber_common::result::Result;

use crate::component::github::{GithubClient, GithubClientTrait};
use crate::service::ServiceTrait;

pub(crate) trait UpdateTrait {
    fn reset(&self) -> Result<()>;
}

#[async_trait]
pub(crate) trait UpdateAsyncTrait {
    async fn has_update(&self) -> Result<(bool, String)>;
    async fn update(&self) -> Result<bool>;
}

#[derive(Debug)]
pub(crate) struct UpdateService {
    pub(crate) config: Option<Arc<Config>>,
    pub(crate) container: Option<Arc<DIContainer>>,
}
unsafe impl Send for UpdateService {}
unsafe impl Sync for UpdateService {}

impl ServiceTrait for UpdateService {
    fn set_shared_properties(&mut self, config: Arc<Config>, container: Arc<DIContainer>) {
        self.config = Some(config);
        self.container = Some(container);
    }
}

impl UpdateService {
    pub(crate) fn new() -> Self {
        Self {
            config: None,
            container: None,
        }
    }
}

impl UpdateTrait for UpdateService {
    fn reset(&self) -> Result<()> {
        let config = self.config.as_ref().unwrap();

        let bin_dir_path = config.bin_dir()?;
        if bin_dir_path.exists() {
            for entry in read_dir(bin_dir_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.file_name().unwrap().to_str().unwrap() == "huber" {
                    continue;
                }

                let _ = remove_dir_all(path);
            }
        }

        let _ = remove_dir_all(config.installed_pkg_root_dir()?);
        let _ = remove_dir_all(config.temp_dir()?);
        let _ = remove_dir_all(config.repo_root_dir()?);

        Ok(())
    }
}

#[async_trait]
impl UpdateAsyncTrait for UpdateService {
    async fn has_update(&self) -> Result<(bool, String)> {
        let config = self.config.as_ref().unwrap();
        let current_version = crate_version!();

        // Note: async closure is not stable yet. ex: async || -> Result<>, so can't use ? in async {}
        let client = GithubClient::new(
            config.github_credentials.clone(),
            config.git_ssh_key.clone(),
        );

        let pkg = create_huber_package();
        match client.get_latest_release("innobead", "huber", &pkg).await {
            Err(e) => Err(e),
            Ok(r) => Ok((
                Version::parse(current_version) >= Version::parse(&r.version),
                r.version,
            )),
        }
    }

    async fn update(&self) -> Result<bool> {
        if !self.has_update().await?.0 {
            return Ok(false);
        }

        let config = self.config.as_ref().unwrap();

        let client = GithubClient::new(
            config.github_credentials.clone(),
            config.git_ssh_key.clone(),
        );

        let pkg = create_huber_package();
        match client.get_latest_release("innobead", "huber", &pkg).await {
            Err(e) => Err(e),

            Ok(r) => {
                match client
                    .download_artifacts(&r, config.bin_dir().unwrap())
                    .await
                {
                    Err(e) => Err(e),
                    Ok(_r_) => Ok(true),
                }
            }
        }
    }
}

fn create_huber_package() -> Package {
    Package {
        name: "huber".to_string(),
        source: PackageSource::Github {
            owner: "innobead".to_string(),
            repo: "huber".to_string(),
        },
        targets: vec![],
        detail: None,
        version: None,
        description: None,
        release_kind: None,
    }
}
