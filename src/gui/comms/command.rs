use crate::states::knob_value_update::KnobValueUpdate;

#[derive(Debug, Copy, Clone)]
pub enum CommsCommand {
    Refresh,
    PadPressed { key: u8 },
    WhiteKeyPressed { key: u8 },
    BlackKeyPressed { key: u8 },
    KnobPercentageChanged { knob: u8, value: KnobValueUpdate },
    LoopPressed,
    ShufflePressed,
    SkipTrackPressed,
    MutePressed,
    PausePressed,
    StopAllPressed,
    SoloPressed,
}
