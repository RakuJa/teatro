use bitflags::bitflags;
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::soft_keys::SoftKey;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ToggleStates: u16 {
        const CLIP_STOP = 1 << 0;
        const SOLO      = 1 << 1;
        const MUTE      = 1 << 2; // mutes sound music
        const REC_ARM   = 1 << 3;
        const SELECT    = 1 << 4; // used to shuffle playlist
        const STOP_ALL  = 1 << 5; // stops music and
        const VOLUME    = 1 << 6;
        const PAN       = 1 << 7;
        const SEND      = 1 << 8;
        const DEVICE    = 1 << 9;
        const SHIFT     = 1 << 10;
        const FILTER    = 1 << 11;
        const START     = 1 << 12;
    }
}

impl From<SoftKey> for ToggleStates {
    fn from(value: SoftKey) -> Self {
        match value {
            SoftKey::ClipStop => Self::CLIP_STOP,
            SoftKey::Solo => Self::SOLO,
            SoftKey::Mute => Self::MUTE,
            SoftKey::RecArm => Self::REC_ARM,
            SoftKey::Select => Self::SELECT,
        }
    }
}

impl From<KnobCtrlKey> for ToggleStates {
    fn from(value: KnobCtrlKey) -> Self {
        match value {
            KnobCtrlKey::Volume => Self::VOLUME,
            KnobCtrlKey::Pan => Self::PAN,
            KnobCtrlKey::Send => Self::SEND,
            KnobCtrlKey::Device => Self::DEVICE,
        }
    }
}
