use crate::audio::output::engine::Engine;
use crate::audio::output::selector::Selector;
use crate::audio::output::{OutputBackend, OutputKind};
use rodio::mixer::Mixer;

pub struct LocalBackend {
    selector: Selector,
}

impl LocalBackend {
    pub fn new(engine: &Engine, device_mixer: &Mixer) -> Self {
        device_mixer.add(engine.tap(OutputKind::Local));
        Self {
            selector: engine.selector(),
        }
    }
}

impl OutputBackend for LocalBackend {
    fn kind(&self) -> OutputKind {
        OutputKind::Local
    }
    fn select(&self) {
        self.selector.confirm(OutputKind::Local);
    }
    fn release(&self) {}
}
