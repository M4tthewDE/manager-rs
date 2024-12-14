use crate::state::{
    info::{cpu::Cpu, disk::Disk, memory::Memory, Info},
    proto::{system_client::SystemClient, Empty, InfoReply},
    State, StateChangeMessage,
};
use anyhow::Result;

impl From<&InfoReply> for Info {
    fn from(i: &InfoReply) -> Self {
        let disks = i
            .disk_info
            .clone()
            .map(|disk_info| disk_info.disks.iter().map(Disk::from).collect())
            .unwrap_or_default();
        let memory = Memory::from(&i.memory_info.unwrap_or_default());
        let cpus = i
            .cpu_info
            .clone()
            .map(|cpu_info| cpu_info.cpus.iter().map(Cpu::from).collect())
            .unwrap_or_default();

        Self {
            name: i.name.clone(),
            kernel_version: i.kernel_version.clone(),
            os_version: i.os_version.clone(),
            host_name: i.host_name.clone(),
            memory,
            disks,
            cpus,
        }
    }
}

pub async fn update_info(server_address: String) -> Result<StateChangeMessage> {
    let mut client = SystemClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    let response = client.get_info(request).await?;
    let info = Info::from(response.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.info = info;
    }))
}
