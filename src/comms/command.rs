use crate::states::knob_value_update::KnobValueUpdate;

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Refresh {
        device: Device,
    },
    PadPressed {
        key: u8,
        device: Device,
    },
    WhiteKeyPressed {
        key: u8,
        device: Device,
    },
    BlackKeyPressed {
        key: u8,
        device: Device,
    },
    KnobPercentageChanged {
        knob: u8,
        value: KnobValueUpdate,
        device: Device,
    },
    LoopPressed {
        device: Device,
    },
    ShufflePressed {
        device: Device,
    },
    SkipTrackPressed {
        device: Device,
    },
    MutePressed {
        device: Device,
    },
    PausePressed {
        device: Device,
    },
    StopAllPressed {
        device: Device,
    },
    SoloPressed {
        device: Device,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum Device {
    ToGui,
    ToBackend,
}
