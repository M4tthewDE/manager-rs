use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_stream::{wrappers::ReceiverStream, Stream};

use crate::{
    proto::{
        system_server::System, Cpu, CpuInfo, Disk, DiskInfo, DockerInfo, Empty, InfoReply,
        LogReply, MemoryInfo, Version,
    },
    subscriber::LogRelay,
};
use sysinfo::{CpuRefreshKind, Disks, RefreshKind};
use tonic::{Request, Response, Status};

use anyhow::Result;
use tracing::{error, info};

use crate::{config::Config, docker};

pub struct SystemService {
    info_reply: Arc<Mutex<InfoReply>>,
    log_relay: Arc<Mutex<LogRelay>>,
}

impl SystemService {
    pub fn new(config: Config, log_relay: Arc<Mutex<LogRelay>>) -> Self {
        let info_reply = Arc::new(Mutex::new(InfoReply::default()));
        let update_interval = Duration::from_millis(config.update_interval);

        info!("starting info updater with interval {:?}", update_interval);
        Self::start_updater(update_interval, Arc::clone(&info_reply));

        Self {
            info_reply,
            log_relay,
        }
    }

    fn start_updater(update_interval: Duration, info: Arc<Mutex<InfoReply>>) {
        tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(update_interval).await;
                match Self::info().await {
                    Ok(i) => match info.lock() {
                        Ok(mut info) => *info = i,
                        Err(err) => error!("{err:?}"),
                    },
                    Err(err) => error!("update error {err:?}"),
                }
            }
        });
    }

    async fn info() -> Result<InfoReply> {
        let mut sys = sysinfo::System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_all();

        Ok(InfoReply {
            name: sysinfo::System::name().unwrap_or_default(),
            kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
            os_version: sysinfo::System::os_version().unwrap_or_default(),
            host_name: sysinfo::System::host_name().unwrap_or_default(),

            memory_info: Some(Self::memory_info(&mut sys)),
            disk_info: Some(Self::disk_info()),
            cpu_info: Some(Self::cpu_info(&mut sys)),
            docker_info: Some(docker_info().await?),
        })
    }

    fn memory_info(sys: &mut sysinfo::System) -> MemoryInfo {
        MemoryInfo {
            total: sys.total_memory(),
            free: sys.free_memory(),
            available: sys.available_memory(),
            used: sys.used_memory(),
        }
    }

    fn disk_info() -> DiskInfo {
        DiskInfo {
            disks: Disks::new_with_refreshed_list()
                .list()
                .iter()
                .map(|d| Disk {
                    name: d.name().to_str().unwrap_or_default().to_string(),
                    kind: d.kind().to_string(),
                    file_system: d.file_system().to_str().unwrap_or_default().to_string(),
                    total_space: d.total_space(),
                    available_space: d.available_space(),
                })
                .collect(),
        }
    }

    fn cpu_info(sys: &mut sysinfo::System) -> CpuInfo {
        CpuInfo {
            cpus: sys
                .cpus()
                .iter()
                .map(|c| Cpu {
                    name: c.name().to_string(),
                    cpu_usage: c.cpu_usage(),
                    frequency: c.frequency(),
                })
                .collect(),
        }
    }
}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_info(&self, _: Request<Empty>) -> Result<Response<InfoReply>, Status> {
        return match self.info_reply.lock() {
            Ok(info_reply) => Ok(Response::new(info_reply.clone())),
            Err(err) => {
                error!("{err:?}");
                Err(Status::from_error("Lock is poisoned".into()))
            }
        };
    }

    type LogStream = Pin<Box<dyn Stream<Item = Result<LogReply, Status>> + Send>>;

    async fn log(
        &self,
        _: tonic::Request<Empty>,
    ) -> std::result::Result<tonic::Response<Self::LogStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        match self.log_relay.lock() {
            Ok(mut log_relay) => {
                log_relay.add_sender(tx.clone());
            }
            Err(err) => {
                error!("{err:?}");
            }
        }

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::LogStream))
    }
}

async fn docker_info() -> Result<DockerInfo, Status> {
    let version: Version = docker::version::version()
        .await
        .map_err(|e| Status::from_error(e.into()))?
        .into();

    let containers = docker::container::list()
        .await
        .map_err(|e| Status::from_error(e.into()))?;

    let mut container_list = Vec::new();
    for c in containers {
        let logs = docker::container::logs(&c.id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;
        let container = crate::proto::Container {
            id: c.id.clone(),
            names: c.names.clone(),
            image: c.image.clone(),
            command: c.command.clone(),
            created: c.created,
            ports: c.ports.iter().map(crate::proto::Port::from).collect(),
            status: c.status.clone(),
            logs,
        };
        container_list.push(container);
    }

    Ok(DockerInfo {
        version: Some(version),
        container_list,
    })
}
