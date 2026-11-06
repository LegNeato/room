use crate::doom::d_player::{PlayerT, PspdefT};
use crate::doom::p_telept::mobj_t;

type Actor = unsafe extern "C" fn(*mut mobj_t);
type Weapon = unsafe extern "C" fn(*mut PlayerT, *mut PspdefT);

/// C `actionf_t` storage with the actual Rust callback signatures.
///
/// This union deliberately has no tag, preserving the C state-table layout.
/// The state graph determines its active field: weapon/flash states
/// `S_LIGHTDONE..=S_BFGFLASH2` use `weapon`; actor states use `actor`.
/// `NONE` is the null function pointer and may be read as either signature.
/// Any state-table patch must preserve that calling-kind invariant.
#[repr(C)]
#[derive(Clone, Copy)]
pub union Action {
    actor: Option<Actor>,
    weapon: Option<Weapon>,
}

impl Action {
    pub(super) const NONE: Self = Self { actor: None };

    pub(super) const fn for_actor(callback: Actor) -> Self {
        Self {
            actor: Some(callback),
        }
    }

    pub(super) const fn for_weapon(callback: Weapon) -> Self {
        Self {
            weapon: Some(callback),
        }
    }

    /// Read an actor state's callback without erasing its parameter type.
    ///
    /// # Safety
    ///
    /// This action must have been constructed with `for_actor` or `NONE`.
    /// Calling the returned function also requires the callback's actor and
    /// engine-state invariants. An untagged C table cannot validate its own kind.
    pub(crate) unsafe fn actor(self) -> Option<Actor> {
        self.actor
    }

    /// Read a weapon/flash state's callback without erasing either parameter.
    ///
    /// # Safety
    ///
    /// This action must have been constructed with `for_weapon` or `NONE`.
    /// Calling the returned function also requires the callback's player,
    /// psprite and engine-state invariants.
    pub(crate) unsafe fn weapon(self) -> Option<Weapon> {
        self.weapon
    }
}

const _: () = assert!(std::mem::size_of::<Action>() == 8);
const _: () = assert!(std::mem::align_of::<Action>() == 8);

#[cfg(test)]
mod tests;
