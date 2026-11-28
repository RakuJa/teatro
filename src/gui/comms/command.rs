#[derive(Debug, Copy, Clone)]
pub enum CommsCommand {
    Refresh,
    PadPressed { key: u8 },
    WhiteKeyPressed { key: u8 },
    BlackKeyPressed { key: u8 },
    KnobTurned { knob: u8, steps: i32 },
    LoopPressed,
    ShufflePressed,
    SkipTrackPressed,
    MutePressed,
    PausePressed,
    StopAllPressed,
    SoloPressed,
    PlayTrackPressed { index: usize },
    SeekTo { target: f32 },
}
