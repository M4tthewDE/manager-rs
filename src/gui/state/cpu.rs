#[derive(Default)]
pub struct Cpu {
    pub name: String,
    pub usage: f32,
    pub frequency: String,
}

impl Cpu {
    pub fn new(c: &super::proto::Cpu) -> Self {
        Self {
            name: c.name.clone(),
            usage: c.cpu_usage,
            frequency: format!("{}MHz", c.frequency),
        }
    }
}
