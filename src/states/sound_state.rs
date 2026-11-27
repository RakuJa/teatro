use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::visualizer::RuntimeData;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SoundState {
    pub data: Arc<Mutex<RuntimeData>>,
    pub audio_sinks: Arc<Mutex<AudioSinks>>,
    pub ambience_filter: Arc<Mutex<FilterData>>,
    pub sound_effect_filter: Arc<Mutex<FilterData>>,
}
