use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::visualizer::AkaiData;
use flume::Sender;
use std::sync::{Arc, Mutex};

pub struct SoundState {
    pub data: AkaiData,
    pub audio_sinks: Arc<Mutex<AudioSinks>>,
    pub sound_filter: Arc<Mutex<FilterData>>,
    pub tx_data: Sender<AkaiData>,
}
