//! Pure-Rust Media & Audio Inspector
//!
//! Provides zero-dependency native media decoding and metadata extraction
//! without requiring external `ffmpeg` binary executables.

use std::fs::File;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{Error, Result};

/// Metadata and inspection result for an audio/media asset.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MediaMetadata {
    pub format_name: String,
    pub codec_name: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_seconds: f64,
}

/// Pure-Rust Media Inspector (FFmpeg binary-independent)
pub struct MediaInspector;

impl MediaInspector {
    /// Inspect an audio file natively from disk or memory stream.
    pub fn inspect_file<P: AsRef<Path>>(path: P) -> Result<MediaMetadata> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .map_err(|e| Error::Decode(format!("Medya dosyası açılamadı: {}", e)))?;

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let fmt_opts = FormatOptions::default();
        let meta_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| Error::Decode(format!("Medya biçimi algılanamadı: {}", e)))?;

        let format = probed.format;

        let track = format
            .default_track()
            .ok_or_else(|| Error::Decode("Varsayılan medya izi (track) bulunamadı".to_string()))?;

        let codec_params = track.codec_params.clone();
        let codec_name = format!("{:?}", codec_params.codec);
        let sample_rate = codec_params.sample_rate.unwrap_or(0);
        let channels = codec_params.channels.map(|c| c.count()).unwrap_or(0);

        let duration_seconds =
            if let (Some(tb), Some(n_frames)) = (codec_params.time_base, codec_params.n_frames) {
                let time = tb.calc_time(n_frames);
                time.seconds as f64 + time.frac
            } else {
                0.0
            };

        Ok(MediaMetadata {
            format_name: "symphonia-native".to_string(),
            codec_name,
            sample_rate,
            channels,
            duration_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_inspector_struct() {
        let meta = MediaMetadata {
            format_name: "symphonia-native".to_string(),
            codec_name: "MP3".to_string(),
            sample_rate: 44100,
            channels: 2,
            duration_seconds: 120.5,
        };
        assert_eq!(meta.sample_rate, 44100);
        assert_eq!(meta.channels, 2);
    }
}
