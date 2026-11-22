use crate::audio::output::engine::Engine;
use crate::audio::output::selector::Selector;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub mod engine;
pub mod selector;
pub mod tap;

pub const OUT_RATE: u32 = 48_000;
pub const OUT_CHANNELS: u32 = 2;
/// Samples pulled from the mix per lock (10 ms of stereo).
const CHUNK: usize = 960;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum OutputKind {
    #[default]
    Local = 0,
    Discord = 1,
}

pub trait OutputBackend: Send + Sync {
    fn kind(&self) -> OutputKind;
    fn select(&self);
    fn release(&self);
}

pub struct Output {
    selector: Selector,
    backends: Mutex<Vec<Box<dyn OutputBackend>>>,
}

impl Output {
    pub fn new(engine: &Engine) -> Self {
        Self {
            selector: engine.selector(),
            backends: Mutex::new(Vec::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Box<dyn OutputBackend>>> {
        self.backends.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn with(self, backend: impl OutputBackend + 'static) -> Self {
        self.add(backend);
        self
    }

    pub fn add(&self, backend: impl OutputBackend + 'static) {
        self.lock().push(Box::new(backend));
    }

    pub fn take(&self, kind: OutputKind) -> Option<Box<dyn OutputBackend>> {
        let mut backends = self.lock();
        let idx = backends.iter().position(|b| b.kind() == kind)?;
        let removed = backends.remove(idx);

        if kind != OutputKind::Local {
            self.selector.want(OutputKind::Local);
            for b in backends.iter().filter(|b| b.kind() == OutputKind::Local) {
                b.select();
            }
        }
        removed.release();
        Some(removed)
    }

    pub fn is_available(&self, kind: OutputKind) -> bool {
        self.lock().iter().any(|b| b.kind() == kind)
    }

    pub fn active(&self) -> OutputKind {
        self.selector.active()
    }

    pub fn use_output(&self, kind: OutputKind) -> bool {
        let backends = self.lock();
        if !backends.iter().any(|b| b.kind() == kind) {
            return false;
        }
        self.selector.want(kind);
        for b in backends.iter().filter(|b| b.kind() == kind) {
            b.select();
        }
        for b in backends.iter().filter(|b| b.kind() != kind) {
            b.release();
        }
        true
    }
}
