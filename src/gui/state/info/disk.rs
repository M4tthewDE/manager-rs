use humansize::DECIMAL;

pub struct Disk {
    pub name: String,
    pub kind: String,
    pub file_system: String,
    pub total_space: String,
    pub available_space: String,
}

impl Disk {
    pub fn new(d: &crate::state::proto::Disk) -> Self {
        Self {
            name: d.name.clone(),
            kind: d.kind.clone(),
            file_system: d.file_system.clone(),
            total_space: humansize::format_size(d.total_space, DECIMAL),
            available_space: humansize::format_size(d.available_space, DECIMAL),
        }
    }
}

impl Default for Disk {
    fn default() -> Self {
        Self {
            name: "n/a".to_string(),
            kind: "n/a".to_string(),
            file_system: "n/a".to_string(),
            total_space: "n/a".to_string(),
            available_space: "n/a".to_string(),
        }
    }
}
