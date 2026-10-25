//! Keyboard state needed to continue a local game between ticks.

use super::*;

/// Input state without pointers or platform handles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub keys: [bool; NUMKEYS],
    pub turn_held: i32,
    pub double_click_time: [i32; 2],
    pub double_click_state: [bool; 2],
    pub double_click_count: [i32; 2],
    pub next_weapon: i32,
    pub turn_carry: i16,
}

impl Input {
    /// Captures keyboard input when mouse and joystick state are empty.
    ///
    /// # Safety
    /// The caller must hold exclusive engine access between completed ticks.
    pub unsafe fn capture() -> Option<Self> {
        if std::ptr::addr_of!(MOUSEARRAY)
            .read()
            .into_iter()
            .chain(std::ptr::addr_of!(JOYARRAY).read())
            .any(|value| value != 0)
            || [MOUSEX, MOUSEY, JOYXMOVE, JOYYMOVE, JOYSTRAFEMOVE]
                .iter()
                .any(|value| *value != 0)
        {
            return None;
        }
        Some(Self {
            keys: GAMEKEYDOWN.map(|value| value != 0),
            turn_held: TURNHELD,
            double_click_time: [DCLICKTIME, DCLICKTIME2],
            double_click_state: [DCLICKSTATE != 0, DCLICKSTATE2 != 0],
            double_click_count: [DCLICKS, DCLICKS2],
            next_weapon: NEXT_WEAPON,
            turn_carry: LOWRES_TURN_CARRY,
        })
    }

    /// Restores input together with the matching game and platform state.
    ///
    /// # Safety
    /// The caller must hold exclusive engine access between completed ticks.
    /// Values must come from a checkpoint of the same engine version.
    pub unsafe fn restore(&self) {
        GAMEKEYDOWN = self.keys.map(i32::from);
        TURNHELD = self.turn_held;
        [DCLICKTIME, DCLICKTIME2] = self.double_click_time;
        [DCLICKSTATE, DCLICKSTATE2] = self.double_click_state.map(i32::from);
        [DCLICKS, DCLICKS2] = self.double_click_count;
        NEXT_WEAPON = self.next_weapon;
        LOWRES_TURN_CARRY = self.turn_carry;
        MOUSEARRAY = [0; MAX_MOUSE_BUTTONS + 1];
        JOYARRAY = [0; MAX_JOY_BUTTONS + 1];
        MOUSEX = 0;
        MOUSEY = 0;
        JOYXMOVE = 0;
        JOYYMOVE = 0;
        JOYSTRAFEMOVE = 0;
    }
}
