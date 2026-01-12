//! Multi-channel audio configuration.
//!
//! This module provides types for describing channel layouts and how blocks
//! should handle multi-channel audio.

/// Describes the channel layout for audio processing.
///
/// Standard layouts include mono, stereo, surround formats, and ambisonics.
/// Use `Custom(n)` for arbitrary channel counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChannelLayout {
    /// Single channel (mono).
    Mono,

    /// Two channels (left, right).
    #[default]
    Stereo,

    /// 5.1 surround: L, R, C, LFE, Ls, Rs (6 channels).
    Surround51,

    /// 7.1 surround: L, R, C, LFE, Ls, Rs, Lrs, Rrs (8 channels).
    Surround71,

    /// First-order ambisonics (4 channels: W, Y, Z, X).
    AmbisonicFoa,

    /// Second-order ambisonics (9 channels).
    AmbisonicSoa,

    /// Third-order ambisonics (16 channels).
    AmbisonicToa,

    /// Custom channel count for non-standard configurations.
    Custom(usize),
}

impl ChannelLayout {
    /// Returns the number of channels for this layout.
    #[inline]
    pub const fn channel_count(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
            Self::AmbisonicFoa => 4,
            Self::AmbisonicSoa => 9,
            Self::AmbisonicToa => 16,
            Self::Custom(n) => *n,
        }
    }

    /// Returns `true` if this is an ambisonic layout.
    #[inline]
    pub const fn is_ambisonic(&self) -> bool {
        matches!(self, Self::AmbisonicFoa | Self::AmbisonicSoa | Self::AmbisonicToa)
    }

    /// Returns the ambisonic order if this is an ambisonic layout.
    ///
    /// Returns `None` for non-ambisonic layouts.
    #[inline]
    pub const fn ambisonic_order(&self) -> Option<usize> {
        match self {
            Self::AmbisonicFoa => Some(1),
            Self::AmbisonicSoa => Some(2),
            Self::AmbisonicToa => Some(3),
            _ => None,
        }
    }

    /// Creates an ambisonic layout from an order (1-3).
    ///
    /// Returns `None` if the order is out of range.
    #[inline]
    pub const fn from_ambisonic_order(order: usize) -> Option<Self> {
        match order {
            1 => Some(Self::AmbisonicFoa),
            2 => Some(Self::AmbisonicSoa),
            3 => Some(Self::AmbisonicToa),
            _ => None,
        }
    }
}

/// Describes how a block handles multi-channel audio.
///
/// Most blocks process channels independently (Parallel), while some blocks
/// like panners and mixers need explicit control over channel routing (Explicit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelConfig {
    /// Process each channel independently.
    ///
    /// The block receives the same number of inputs and outputs, and each
    /// channel is processed through the same algorithm. This is the default
    /// for most effect blocks (filters, gain, distortion).
    #[default]
    Parallel,

    /// Block handles channel routing internally.
    ///
    /// The block may have different input and output channel counts and
    /// implements its own routing logic. Used for panners, mixers, and
    /// channel splitters/mergers.
    Explicit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_layout_channel_count() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
        assert_eq!(ChannelLayout::Surround51.channel_count(), 6);
        assert_eq!(ChannelLayout::Surround71.channel_count(), 8);
        assert_eq!(ChannelLayout::AmbisonicFoa.channel_count(), 4);
        assert_eq!(ChannelLayout::AmbisonicSoa.channel_count(), 9);
        assert_eq!(ChannelLayout::AmbisonicToa.channel_count(), 16);
        assert_eq!(ChannelLayout::Custom(12).channel_count(), 12);
    }

    #[test]
    fn channel_layout_is_ambisonic() {
        assert!(!ChannelLayout::Mono.is_ambisonic());
        assert!(!ChannelLayout::Stereo.is_ambisonic());
        assert!(!ChannelLayout::Surround51.is_ambisonic());
        assert!(!ChannelLayout::Surround71.is_ambisonic());
        assert!(ChannelLayout::AmbisonicFoa.is_ambisonic());
        assert!(ChannelLayout::AmbisonicSoa.is_ambisonic());
        assert!(ChannelLayout::AmbisonicToa.is_ambisonic());
        assert!(!ChannelLayout::Custom(4).is_ambisonic());
    }

    #[test]
    fn channel_layout_ambisonic_order() {
        assert_eq!(ChannelLayout::Mono.ambisonic_order(), None);
        assert_eq!(ChannelLayout::Stereo.ambisonic_order(), None);
        assert_eq!(ChannelLayout::AmbisonicFoa.ambisonic_order(), Some(1));
        assert_eq!(ChannelLayout::AmbisonicSoa.ambisonic_order(), Some(2));
        assert_eq!(ChannelLayout::AmbisonicToa.ambisonic_order(), Some(3));
    }

    #[test]
    fn channel_layout_from_ambisonic_order() {
        assert_eq!(ChannelLayout::from_ambisonic_order(0), None);
        assert_eq!(
            ChannelLayout::from_ambisonic_order(1),
            Some(ChannelLayout::AmbisonicFoa)
        );
        assert_eq!(
            ChannelLayout::from_ambisonic_order(2),
            Some(ChannelLayout::AmbisonicSoa)
        );
        assert_eq!(
            ChannelLayout::from_ambisonic_order(3),
            Some(ChannelLayout::AmbisonicToa)
        );
        assert_eq!(ChannelLayout::from_ambisonic_order(4), None);
    }

    #[test]
    fn channel_layout_default() {
        assert_eq!(ChannelLayout::default(), ChannelLayout::Stereo);
    }

    #[test]
    fn channel_config_default() {
        assert_eq!(ChannelConfig::default(), ChannelConfig::Parallel);
    }
}
