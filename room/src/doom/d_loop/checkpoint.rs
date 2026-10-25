//! A local loop can stop after its last prepared command has run.

use super::*;

/// Counters for a single-player loop with one command per tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub tick: i32,
    pub entered_at: i32,
}

impl Boundary {
    /// Captures a boundary with no prepared commands left to consume.
    ///
    /// # Safety
    /// The caller must hold exclusive engine access outside `TryRunTics`.
    pub unsafe fn capture() -> Option<Self> {
        if singletics != 1
            || ticdup != 1
            || LOCALPLAYER != 0
            || NEW_SYNC != 0
            || net_client_connected != 0
            || drone != 0
            || MAKETIC != gametic
        {
            return None;
        }
        Some(Self {
            tick: gametic,
            entered_at: OLDENTERTICS,
        })
    }

    /// Restores counters after the matching world and input have been restored.
    ///
    /// The next local command replaces its ring-buffer slot before execution.
    ///
    /// # Safety
    /// The caller must hold exclusive engine access outside `TryRunTics`.
    /// The destination must use the same single-player configuration.
    pub unsafe fn restore(self) {
        gametic = self.tick;
        MAKETIC = self.tick;
        OLDENTERTICS = self.entered_at;
    }
}
