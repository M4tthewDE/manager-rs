#[derive(Default)]
pub struct Cpu {
    pub name: String,
    pub usage: String,
    pub frequency: String,
}

impl Cpu {
    pub fn new(c: &super::proto::Cpu) -> Self {
        Self {
            name: c.name.clone(),
            usage: format!("{:.2}%", c.cpu_usage),
            frequency: format!("{}MHz", c.frequency),
        }
    }
}
