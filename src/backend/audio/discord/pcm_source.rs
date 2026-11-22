use crate::audio::output::tap::Tap;
use songbird::input::core::io::MediaSource;
use std::io;
use std::io::{Read, Seek, SeekFrom};

pub struct PcmSource {
    tap: Tap,
    carry: [u8; 4],
    pos: usize,
}

impl PcmSource {
    pub(crate) const fn new(tap: Tap) -> Self {
        Self {
            tap,
            carry: [0; 4],
            pos: 4,
        }
    }
}

impl Read for PcmSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for byte in buf.iter_mut() {
            if self.pos == 4 {
                self.carry = self.tap.next().unwrap_or(0.).to_le_bytes();
                self.pos = 0;
            }
            *byte = self.carry[self.pos];
            self.pos += 1;
        }
        Ok(buf.len())
    }
}

impl Seek for PcmSource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "live PCM stream is not seekable",
        ))
    }
}

impl MediaSource for PcmSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::output::engine::Engine;
    use crate::audio::output::{OUT_CHANNELS, OUT_RATE, OutputKind};
    use songbird::input::RawAdapter;

    #[test]
    fn adapter_emits_header_then_silence_when_not_active() {
        // Local is the active output by default, so a Discord tap is silent.
        let engine = Engine::new();
        let mut adapter = RawAdapter::new(
            PcmSource::new(engine.tap(OutputKind::Discord)),
            OUT_RATE,
            OUT_CHANNELS,
        );

        let mut header = [0u8; 16];
        adapter.read_exact(&mut header).unwrap();
        assert_eq!(&header[..5], b"Sbird");

        // The stream must stay alive and yield silence.
        let mut pcm = [1u8; 64];
        assert_eq!(adapter.read(&mut pcm).unwrap(), 64);
        assert!(pcm.iter().all(|&b| b == 0));
    }

    #[test]
    fn pcm_source_is_not_seekable() {
        let engine = Engine::new();
        let mut src = PcmSource::new(engine.tap(OutputKind::Discord));
        assert!(!src.is_seekable());
        assert!(src.byte_len().is_none());
        assert!(src.seek(SeekFrom::Start(0)).is_err());
    }

    #[test]
    fn pcm_source_serves_partial_samples_across_reads() {
        // Reads that don't align to 4 bytes must not drop or repeat bytes.
        let engine = Engine::new();
        let mut src = PcmSource::new(engine.tap(OutputKind::Discord));
        let mut total = 0;
        for len in [1, 3, 5, 7, 11] {
            let mut buf = vec![1u8; len];
            assert_eq!(src.read(&mut buf).unwrap(), len);
            assert!(buf.iter().all(|&b| b == 0));
            total += len;
        }
        assert_eq!(total, 27);
    }
}
