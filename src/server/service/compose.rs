use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::config::Config;
use crate::docker;
use crate::docker::container::{ContainerCreationBody, HostConfig, PortBinding};
use crate::proto::DeployRequest;
use crate::proto::{
    compose_server::Compose, ComposeFile, ComposeFileDiff, DiffReply, DiffRequest, DiffResult,
    Empty, PushRequest,
};

#[derive(Debug)]
pub struct ComposeService {
    docker_compose_path: PathBuf,
}

impl From<Config> for ComposeService {
    fn from(config: Config) -> Self {
        Self {
            docker_compose_path: config.docker_compose_path,
        }
    }
}

#[tonic::async_trait]
impl Compose for ComposeService {
    async fn diff(&self, req: Request<DiffRequest>) -> Result<Response<DiffReply>, Status> {
        let diffs = self
            .calculate_diffs(req.get_ref())
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(DiffReply { diffs }))
    }

    async fn push(&self, req: Request<PushRequest>) -> Result<Response<Empty>, Status> {
        match self.push_file(req.get_ref()) {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(err) => {
                error!("push error: {err:?}");
                Err(Status::from_error(err.into()))
            }
        }
    }

    async fn deploy(&self, req: Request<DeployRequest>) -> Result<Response<Empty>, Status> {
        match self.handle_deploy(req.get_ref()).await {
            Ok(_) => Ok(Response::new(Empty {})),
            Err(err) => {
                error!("deploy error: {err:?}");
                Err(Status::from_error(err.into()))
            }
        }
    }
}

impl ComposeService {
    fn calculate_diffs(&self, req: &DiffRequest) -> anyhow::Result<Vec<ComposeFileDiff>> {
        let mut diffs = req
            .files
            .iter()
            .map(|f| self.diff(f.clone()))
            .collect::<anyhow::Result<Vec<ComposeFileDiff>>>()?;

        for dir_entry in self.docker_compose_path.read_dir()? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();

            if path.is_dir() {
                continue;
            }

            if Self::got_removed(
                &path.strip_prefix(&self.docker_compose_path)?.to_path_buf(),
                &req.files,
            ) {
                diffs.push(ComposeFileDiff {
                    path: path
                        .to_str()
                        .context("invalid path {dir_entry:?}")?
                        .to_string(),
                    result: DiffResult::Removed.into(),
                    content: "".to_string(),
                })
            }
        }

        Ok(diffs)
    }

    fn diff(&self, file: ComposeFile) -> anyhow::Result<ComposeFileDiff> {
        let mut path = self.docker_compose_path.clone();
        path.push(file.path.clone());

        if !path.exists() {
            return Ok(ComposeFileDiff {
                path: file.path,
                result: DiffResult::New.into(),
                content: file.content,
            });
        }

        let result = if std::fs::read_to_string(path)? == file.content {
            DiffResult::Same.into()
        } else {
            DiffResult::Modified.into()
        };

        Ok(ComposeFileDiff {
            path: file.path,
            result,
            content: file.content,
        })
    }

    fn got_removed(path: &PathBuf, files: &[ComposeFile]) -> bool {
        for file in files {
            if PathBuf::from(file.path.clone()) == *path {
                return false;
            }
        }

        true
    }

    fn push_file(&self, req: &PushRequest) -> anyhow::Result<()> {
        let file = req.file.clone().context("no file in {req:?}")?;
        let mut path = self.docker_compose_path.clone();
        path.push(file.path);
        let mut dir_path = path.clone();
        dir_path.pop();

        match req.diff_result() {
            DiffResult::Same => (),
            DiffResult::New | DiffResult::Modified => {
                std::fs::create_dir_all(dir_path)?;
                std::fs::write(path, file.content)?;
            }
            DiffResult::Removed => {
                std::fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    async fn handle_deploy(&self, req: &DeployRequest) -> anyhow::Result<()> {
        let mut path = self.docker_compose_path.clone();
        path.push(req.path.clone());

        let service_def: ServiceDefinition = toml::from_str(&std::fs::read_to_string(path)?)?;
        info!("Deploying service");

        info!("Pulling image {}:{}", service_def.image, service_def.tag);
        docker::image::pull(&service_def.image, &service_def.tag).await?;

        let mut port_bindings = HashMap::new();

        for port in service_def.ports {
            port_bindings.insert(
                format!("{}/{}", port.container_port, port.protocol),
                vec![PortBinding {
                    host_ip: port.host_ip,
                    host_port: port.host_port,
                }],
            );
        }

        let body = ContainerCreationBody {
            image: service_def.image,
            command: service_def.command,
            host_config: HostConfig {
                port_bindings,
                binds: service_def.binds,
            },
        };

        info!("Creating container {}", service_def.container_name);
        let id = docker::container::create(&service_def.container_name, body).await?;

        info!("Starting container {}", id);
        docker::container::start(&id).await
    }
}

#[derive(Deserialize, Clone, Debug)]
struct PortMapping {
    protocol: String,
    host_ip: String,
    host_port: String,
    container_port: String,
}

#[derive(Deserialize, Clone, Debug)]
struct ServiceDefinition {
    image: String,
    tag: String,
    container_name: String,
    command: Option<String>,
    binds: Option<Vec<String>>,
    ports: Vec<PortMapping>,
}
