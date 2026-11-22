use crate::audio::output::OutputKind;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Clone)]
pub struct Selector(Arc<SelectorInner>);

struct SelectorInner {
    wanted: Mutex<OutputKind>,
    active: AtomicU8,
}

impl Selector {
    pub(crate) fn new() -> Self {
        Self(Arc::new(SelectorInner {
            wanted: Mutex::new(OutputKind::Local),
            active: AtomicU8::new(OutputKind::Local as u8),
        }))
    }

    pub fn want(&self, kind: OutputKind) {
        *self.0.wanted.lock().unwrap_or_else(PoisonError::into_inner) = kind;
    }

    pub fn confirm(&self, kind: OutputKind) {
        let wanted = self.0.wanted.lock().unwrap_or_else(PoisonError::into_inner);
        if *wanted == kind {
            self.0.active.store(kind as u8, Ordering::Relaxed);
        }
    }

    pub fn active(&self) -> OutputKind {
        match self.0.active.load(Ordering::Relaxed) {
            0 => OutputKind::Local,
            _ => OutputKind::Discord,
        }
    }
}
