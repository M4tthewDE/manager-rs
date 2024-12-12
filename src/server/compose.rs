use std::path::PathBuf;

use anyhow::Context;
use tonic::{Request, Response, Status};

use crate::{
    config::Config,
    proto::{
        compose_server::{Compose, ComposeServer},
        ComposeFile, ComposeFileDiff, DiffReply, DiffRequest, DiffResult,
    },
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

pub fn service(config: Config) -> ComposeServer<ComposeService> {
    ComposeServer::new(ComposeService::from(config))
}

#[tonic::async_trait]
impl Compose for ComposeService {
    async fn diff(&self, req: Request<DiffRequest>) -> Result<Response<DiffReply>, Status> {
        let diffs = self
            .calculate_diffs(req.get_ref())
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(DiffReply { diffs }))
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
            let name = dir_entry.file_name().to_str().context("")?.to_string();

            if Self::got_removed(&name, &req.files) {
                diffs.push(ComposeFileDiff {
                    name: name.to_string(),
                    result: DiffResult::Removed.into(),
                })
            }
        }

        Ok(diffs)
    }

    fn got_removed(name: &str, files: &[ComposeFile]) -> bool {
        for file in files {
            if *file.name == *name {
                return false;
            }
        }

        true
    }
}

impl ComposeService {
    fn diff(&self, file: ComposeFile) -> anyhow::Result<ComposeFileDiff> {
        for dir_entry in self.docker_compose_path.read_dir()? {
            let dir_entry = dir_entry?;
            if *dir_entry.file_name() == *file.name {
                return self.diff_dir_entry(file);
            }
        }

        Ok(ComposeFileDiff {
            name: file.name,
            result: DiffResult::New.into(),
        })
    }

    fn diff_dir_entry(&self, file: ComposeFile) -> anyhow::Result<ComposeFileDiff> {
        let mut file_path = self.docker_compose_path.clone();
        file_path.push(file.name.clone());
        let content = std::fs::read_to_string(file_path)?;

        if content == file.content {
            Ok(ComposeFileDiff {
                name: file.name,
                result: DiffResult::Same.into(),
            })
        } else {
            Ok(ComposeFileDiff {
                name: file.name,
                result: DiffResult::Modified.into(),
            })
        }
    }
}
