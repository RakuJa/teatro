pub enum Command {
    Refresh { device: Device },
}

pub enum Device {
    ToGui,
    ToBackend,
}
