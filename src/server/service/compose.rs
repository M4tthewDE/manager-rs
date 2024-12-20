use std::path::PathBuf;

use anyhow::Context;
use tonic::{Request, Response, Status};
use tracing::error;

use crate::config::Config;
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
}
