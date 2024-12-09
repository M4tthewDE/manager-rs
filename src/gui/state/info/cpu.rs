use crate::state::proto;

#[derive(Default)]
pub struct Cpu {
    pub name: String,
    pub usage: f32,
    pub frequency: String,
}

impl From<&proto::Cpu> for Cpu {
    fn from(c: &proto::Cpu) -> Self {
        Self {
            name: c.name.clone(),
            usage: c.cpu_usage,
            frequency: format!("{}MHz", c.frequency),
        }
    }
}
