#[derive(Debug, Copy, Clone)]
pub enum CommsCommand {
    Refresh,
    PadPressed {
        key: u8,
    },
    WhiteKeyPressed {
        key: u8,
    },
    BlackKeyPressed {
        key: u8,
    },
    KnobTurned {
        knob: u8,
        steps: i32,
    },
    #[expect(dead_code, reason = "no loop toggle in the GUI yet")]
    LoopPressed,
    ShufflePressed,
    SkipTrackPressed,
    MutePressed,
    PausePressed,
    StopAllPressed,
    SoloPressed,
    PlayTrackPressed {
        index: usize,
    },
    SeekTo {
        target: f32,
    },
}
