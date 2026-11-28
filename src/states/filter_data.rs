use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Knob positions below this are "off".
const BYPASS_BELOW: f32 = 0.0025;

/// Cutoff used by the "muffle" toggle.
pub const MUFFLE_HZ: f32 = 400.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::enum_variant_names, reason = "standard filter names")]
pub enum FilterKind {
    /// 12 dB/oct Butterworth low-pass: "behind a wall / underwater".
    LowPass,
    /// 6 dB/oct one-pole low-pass: gentle darkening.
    SoftLowPass,
    /// 12 dB/oct Butterworth high-pass: "thin / radio / far away".
    HighPass,
}

impl FilterKind {
    const fn sweep(self) -> (f32, f32) {
        match self {
            Self::LowPass => (20_000.0, 200.0),
            Self::SoftLowPass => (20_000.0, 150.0),
            Self::HighPass => (20.0, 5_000.0),
        }
    }

    /// Maps a knob position to a cutoff on a ~logarithmic scale.
    /// Returns `None` when the knob is at zero.
    pub fn cutoff_hz(self, knob: f32) -> Option<f32> {
        if !knob.is_finite() || knob < BYPASS_BELOW {
            return None;
        }
        let (from, to) = self.sweep();
        Some(from * (to / from).powf(knob.min(1.0)))
    }

    /// Inverse of [`Self::cutoff_hz`]: the knob position giving `hz`.
    pub fn knob_for_cutoff(self, hz: f32) -> f32 {
        let (from, to) = self.sweep();
        if !hz.is_finite() || hz <= 0.0 {
            return 0.0;
        }
        (hz / from).log(to / from).clamp(0.0, 1.0)
    }
}

pub const fn knob_filter(knob: u8) -> Option<FilterKind> {
    match knob {
        2 | 6 => Some(FilterKind::LowPass),
        3 | 8 => Some(FilterKind::HighPass),
        4 => Some(FilterKind::SoftLowPass),
        _ => None,
    }
}

pub const MUSIC_FILTER_KNOBS: &[u8] = &[2, 3, 4];
pub const AMBIENCE_FILTER_KNOBS: &[u8] = &[6];
pub const SOUND_EFFECT_FILTER_KNOBS: &[u8] = &[8];

/// Cutoffs (Hz) of each stage; `None` means the stage is bypassed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FilterChain {
    pub high_pass: Option<f32>,
    pub low_pass: Option<f32>,
    pub soft_low_pass: Option<f32>,
}

impl FilterChain {
    pub fn from_knobs(knob_values: &HashMap<u8, f32>, knobs: &[u8]) -> Self {
        let mut chain = Self::default();
        for &knob in knobs {
            let Some(kind) = knob_filter(knob) else {
                continue;
            };
            let cutoff = kind.cutoff_hz(knob_values.get(&knob).copied().unwrap_or(0.0));
            match kind {
                FilterKind::HighPass => chain.high_pass = cutoff,
                FilterKind::LowPass => chain.low_pass = cutoff,
                FilterKind::SoftLowPass => chain.soft_low_pass = cutoff,
            }
        }
        chain
    }

    /// Forces the low-pass down to at least [`MUFFLE_HZ`].
    #[must_use]
    pub fn muffled(mut self) -> Self {
        self.low_pass = Some(self.low_pass.map_or(MUFFLE_HZ, |hz| hz.min(MUFFLE_HZ)));
        self
    }

    /// Active stages in processing order.
    pub fn stages(&self) -> impl Iterator<Item = (FilterKind, f32)> {
        [
            (FilterKind::HighPass, self.high_pass),
            (FilterKind::LowPass, self.low_pass),
            (FilterKind::SoftLowPass, self.soft_low_pass),
        ]
        .into_iter()
        .filter_map(|(kind, hz)| hz.map(|hz| (kind, hz)))
    }
}

struct Shared {
    chain: Mutex<FilterChain>,
    version: AtomicU64,
}

#[derive(Clone)]
pub struct FilterData {
    shared: Arc<Shared>,
}

impl Default for FilterData {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterData {
    /// A fully bypassed filter.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                chain: Mutex::new(FilterChain::default()),
                version: AtomicU64::new(0),
            }),
        }
    }

    pub fn chain(&self) -> FilterChain {
        *self.lock()
    }

    pub fn set(&self, chain: FilterChain) {
        {
            let mut current = self.lock();
            if *current == chain {
                return;
            }
            *current = chain;
        }
        self.shared.version.fetch_add(1, Ordering::Release);
    }

    pub(crate) fn version(&self) -> u64 {
        self.shared.version.load(Ordering::Acquire)
    }

    fn lock(&self) -> MutexGuard<'_, FilterChain> {
        self.shared
            .chain
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_bypass_and_first_tick_is_inaudible() {
        for kind in [
            FilterKind::LowPass,
            FilterKind::SoftLowPass,
            FilterKind::HighPass,
        ] {
            assert_eq!(kind.cutoff_hz(0.0), None);
            assert_eq!(kind.cutoff_hz(1e-7), None, "float noise must stay off");
        }
        assert!(FilterKind::LowPass.cutoff_hz(0.005).unwrap() > 19_000.0);
        assert!(FilterKind::HighPass.cutoff_hz(0.005).unwrap() < 21.0);
    }

    #[test]
    fn more_knob_always_means_more_filtering() {
        let mut last_low_pass = f32::INFINITY;
        let mut last_high_pass = 0.0;
        for tick in 1..=200 {
            let v = tick as f32 * 0.005;
            let lp = FilterKind::LowPass.cutoff_hz(v).unwrap();
            let hp = FilterKind::HighPass.cutoff_hz(v).unwrap();
            assert!(
                lp < last_low_pass && hp > last_high_pass,
                "not monotonic at {v}"
            );
            last_low_pass = lp;
            last_high_pass = hp;
        }
        assert!((last_low_pass - 200.0).abs() < 0.5);
        assert!((last_high_pass - 5_000.0).abs() < 1.0);
    }

    #[test]
    fn scale_is_logarithmic() {
        let mid = FilterKind::LowPass.cutoff_hz(0.5).unwrap();
        assert!((mid - 2_000.0).abs() < 1.0, "got {mid}");
    }

    #[test]
    fn knob_for_cutoff_inverts_cutoff_hz() {
        for kind in [
            FilterKind::LowPass,
            FilterKind::SoftLowPass,
            FilterKind::HighPass,
        ] {
            for v in [0.1, 0.37, 0.8, 1.0] {
                let hz = kind.cutoff_hz(v).unwrap();
                assert!((kind.knob_for_cutoff(hz) - v).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn music_knobs_drive_independent_stages() {
        let knobs = HashMap::from([(2, 0.5), (3, 0.0), (4, 0.25)]);
        let chain = FilterChain::from_knobs(&knobs, MUSIC_FILTER_KNOBS);
        assert_eq!(chain.high_pass, None);
        assert!(chain.low_pass.is_some() && chain.soft_low_pass.is_some());
        assert_eq!(chain.stages().count(), 2);
    }

    #[test]
    fn muffle_only_ever_lowers_the_cutoff() {
        let open = FilterChain::default().muffled();
        assert_eq!(open.low_pass, Some(MUFFLE_HZ));
        let darker = FilterChain {
            low_pass: Some(250.0),
            ..FilterChain::default()
        }
        .muffled();
        assert_eq!(darker.low_pass, Some(250.0));
    }

    #[test]
    fn set_bumps_version_only_on_change() {
        let f = FilterData::new();
        f.set(FilterChain::default());
        assert_eq!(f.version(), 0);
        f.set(FilterChain {
            low_pass: Some(1_000.0),
            ..FilterChain::default()
        });
        assert_eq!(f.version(), 1);
    }
}
