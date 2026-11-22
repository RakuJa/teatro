use biquad::Type;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

pub const MIN_FILTER_PERCENTAGE: f32 = 1.;
pub const MAX_FILTER_PERCENTAGE: f32 = 50.;

#[derive(Clone, Copy, Debug)]
pub struct FilterParams {
    /// Cutoff as a percentage of the sample rate (50% == Nyquist).
    pub percentage: f32,
    pub filter_type: Type<f32>,
}

struct Shared {
    params: Mutex<FilterParams>,
    version: AtomicU64,
}

#[derive(Clone)]
pub struct FilterData {
    shared: Arc<Shared>,
}

impl FilterData {
    pub fn new(percentage: f32, filter_type: Type<f32>) -> Self {
        Self {
            shared: Arc::new(Shared {
                params: Mutex::new(FilterParams {
                    percentage: percentage.clamp(MIN_FILTER_PERCENTAGE, MAX_FILTER_PERCENTAGE),
                    filter_type,
                }),
                version: AtomicU64::new(0),
            }),
        }
    }

    pub fn params(&self) -> FilterParams {
        *self.lock()
    }

    /// Moves the cutoff by `delta` percentage points (clamped to `[1, 50]`) and sets the type.
    pub fn adjust(&self, delta: f32, filter_type: Type<f32>) {
        {
            let mut p = self.lock();
            p.percentage =
                (p.percentage + delta).clamp(MIN_FILTER_PERCENTAGE, MAX_FILTER_PERCENTAGE);
            p.filter_type = filter_type;
        }
        self.shared.version.fetch_add(1, Ordering::Release);
    }

    pub(crate) fn version(&self) -> u64 {
        self.shared.version.load(Ordering::Acquire)
    }

    fn lock(&self) -> MutexGuard<'_, FilterParams> {
        self.shared
            .params
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
