use std::path::PathBuf;

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
        let diffs = req
            .get_ref()
            .files
            .iter()
            .map(|f| self.diff(f.clone()))
            .collect::<anyhow::Result<Vec<ComposeFileDiff>>>()
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(DiffReply { diffs }))
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
