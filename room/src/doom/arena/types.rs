use std::{fmt, ops::BitOr};

/// One of the two players in a local arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Player {
    One = 0,
    Two = 1,
}

impl Player {
    pub const fn index(self) -> usize {
        self as usize
    }
}

impl TryFrom<u8> for Player {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::One),
            1 => Ok(Self::Two),
            _ => Err(Error::Player),
        }
    }
}

/// Gameplay buttons. Network pause, save, and chat commands are excluded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Buttons(u8);

impl Buttons {
    pub const NONE: Self = Self(0);
    pub const FIRE: Self = Self(1);
    pub const USE: Self = Self(2);

    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Buttons {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        if value & !3 == 0 {
            Ok(Self(value))
        } else {
            Err(Error::Buttons)
        }
    }
}

impl BitOr for Buttons {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// One player's input for one 35 Hz simulation tic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Command {
    forward: i8,
    strafe: i8,
    turn: i16,
    buttons: Buttons,
}

impl Command {
    pub fn new(forward: i8, strafe: i8, turn: i16, buttons: Buttons) -> Result<Self, Error> {
        if !(-50..=50).contains(&forward) || !(-50..=50).contains(&strafe) {
            return Err(Error::Movement);
        }
        Ok(Self {
            forward,
            strafe,
            turn,
            buttons,
        })
    }

    pub(super) fn raw(self) -> super::super::d_player::TiccmdT {
        // TiccmdT contains only integers. Zero also initializes its ABI padding.
        let mut command: super::super::d_player::TiccmdT = unsafe { std::mem::zeroed() };
        command.forwardmove = self.forward;
        command.sidemove = self.strafe;
        command.angleturn = self.turn;
        command.buttons = self.buttons.bits();
        command
    }
}

/// E1M1 deathmatch with monsters. Nightmare's implicit fast/respawn mode is excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    skill: u8,
    seed: u8,
}

impl Config {
    pub fn new(skill: u8, seed: u8) -> Result<Self, Error> {
        if skill > 3 {
            return Err(Error::Skill);
        }
        Ok(Self { skill, seed })
    }

    pub const fn skill(self) -> u8 {
        self.skill
    }

    pub const fn seed(self) -> u8 {
        self.seed
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { skill: 2, seed: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerState {
    pub health: i32,
    pub armor: i32,
    pub ammo: [i32; 4],
    /// Opponent frags minus self frags, as in Doom's deathmatch scoreboard.
    pub frags: i64,
    pub monster_kills: i32,
    pub alive: bool,
    /// Position in Doom's signed 16.16 map coordinates.
    pub x: i32,
    pub y: i32,
    /// Unsigned angle: a full turn spans all 32 bits.
    pub angle: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub tick: u64,
    pub players: [PlayerState; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Player,
    Buttons,
    Movement,
    Skill,
    Map,
    State,
    TickLimit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Player => "the arena has only players zero and one",
            Self::Buttons => "the command contains a non-gameplay button",
            Self::Movement => "movement must be between -50 and 50",
            Self::Skill => "arena skill must be between zero and three",
            Self::Map => "E1M1 needs two player starts and at least four deathmatch starts",
            Self::State => "the arena is not at a completed local tic",
            Self::TickLimit => "the arena reached Doom's tic counter limit",
        })
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_reject_control_messages_and_turbo_movement() {
        for value in 0..=u8::MAX {
            assert_eq!(Buttons::try_from(value).is_ok(), value <= 3);
        }
        assert!(Command::new(-50, 50, i16::MIN, Buttons::FIRE | Buttons::USE).is_ok());
        assert!(Command::new(-51, 0, 0, Buttons::NONE).is_err());
        assert!(Command::new(0, 51, 0, Buttons::NONE).is_err());
    }

    #[test]
    fn players_and_skills_are_bounded() {
        assert_eq!(Player::try_from(1), Ok(Player::Two));
        assert!(Player::try_from(2).is_err());
        assert!(Config::new(3, 255).is_ok());
        assert!(Config::new(4, 0).is_err());
    }
}
