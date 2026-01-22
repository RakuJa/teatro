use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::visualizer::AkaiData;
use flume::Sender;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MusicState {
    pub music_filter: Arc<Mutex<FilterData>>,
    pub sound_filter: Arc<Mutex<FilterData>>,
    pub data: Arc<Mutex<AkaiData>>,
    pub audio_sinks: Arc<Mutex<AudioSinks>>,
    pub tx_data: Sender<AkaiData>,
}
