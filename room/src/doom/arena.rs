//! A local authoritative deathmatch world driven by two commands per tic.
//!
//! This module uses the same global world as the original Doom loop. Callers
//! must serialize access or provide a private static image for each arena.

mod map;
mod types;
mod viewport;

pub use types::{Buttons, Command, Config, Error, Player, PlayerState, Snapshot};
pub use viewport::Viewport;

use super::{d_loop, d_main, d_net, d_player, g_game, m_random, p_tick};

static mut CONFIG: Option<Config> = None;
static mut VIEWPORT: Viewport = Viewport::NATIVE;
static mut NATIVE_CLIP_ANGLE: u32 = 0;

/// The current raster in the native-stride framebuffer.
///
/// # Safety
/// The caller holds the active arena's exclusive static image.
pub unsafe fn viewport() -> Viewport {
    VIEWPORT
}

/// Local driver state saved with the matching world and player graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub config: Config,
    pub consistency: [[u8; d_loop::BACKUPTICS]; d_player::MAXPLAYERS],
}

/// Reads the configuration selected for this engine image.
///
/// # Safety
/// The caller must hold exclusive access to the engine's static image.
pub unsafe fn config() -> Option<Config> {
    std::ptr::addr_of!(CONFIG).read()
}

pub(super) unsafe fn clear() {
    CONFIG = None;
}

pub(super) unsafe fn seed_random() {
    if let Some(config) = CONFIG {
        m_random::rndindex = i32::from(config.seed());
        m_random::prndindex = i32::from(config.seed());
    }
}

/// Starts or resets the two-player world after the WAD and graphics initialize.
///
/// # Safety
/// The caller must hold exclusive engine access. The trusted WAD must remain
/// loaded, allocations must succeed, and no borrowed world or frame may remain.
pub unsafe fn start(config: Config) -> Result<Snapshot, Error> {
    map::validate()?;
    CONFIG = Some(config);
    g_game::demoplayback = 0;
    g_game::demorecording = 0;
    g_game::netdemo = 0;
    g_game::netgame = 1;
    g_game::deathmatch = 1;
    g_game::playeringame = [1, 1, 0, 0];
    g_game::consoleplayer = 0;
    g_game::displayplayer = 0;
    d_main::nomonsters = 0;
    d_main::respawnparm = 0;
    d_main::fastparm = 0;
    g_game::consistancy = [[0; d_loop::BACKUPTICS]; d_player::MAXPLAYERS];
    g_game::bodyque = [std::ptr::null_mut(); 32];
    // Reset native per-player counters before G_PlayerReborn preserves them.
    for index in 0..2 {
        let player = &mut *std::ptr::addr_of_mut!(g_game::players)
            .cast::<d_player::PlayerT>()
            .add(index);
        player.frags = [0; 4];
        player.killcount = 0;
        player.itemcount = 0;
        player.secretcount = 0;
    }
    d_loop::singletics = 1;
    d_loop::ticdup = 1;
    d_loop::checkpoint::Boundary {
        tick: 0,
        entered_at: 0,
    }
    .restore();
    g_game::G_InitNew(i32::from(config.skill()), 1, 1);
    g_game::gameaction = 0;
    g_game::oldgamestate = g_game::gamestate;
    d_main::wipegamestate = g_game::gamestate;
    super::r_main::R_SetViewSize(11, 0);
    super::r_main::R_ExecuteSetViewSize();
    VIEWPORT = Viewport::NATIVE;
    NATIVE_CLIP_ANGLE = super::r_main::clipangle;
    snapshot()
}

unsafe fn boundary() -> Result<d_loop::checkpoint::Boundary, Error> {
    if config().is_none()
        || g_game::netgame != 1
        || g_game::deathmatch != 1
        || g_game::playeringame != [1, 1, 0, 0]
        || g_game::consoleplayer != 0
        || g_game::gamestate != 0
        || g_game::gameaction != 0
        || g_game::demoplayback != 0
        || g_game::demorecording != 0
        || g_game::paused != 0
    {
        return Err(Error::State);
    }
    d_loop::checkpoint::Boundary::capture().ok_or(Error::State)
}

struct Commands(*mut d_player::TiccmdT);

impl Drop for Commands {
    fn drop(&mut self) {
        // The command array outlives G_Ticker but must not remain in a global.
        unsafe { d_net::netcmds = self.0 };
    }
}

/// Applies both players' inputs and advances the shared world once.
///
/// # Safety
/// The initialization and exclusive-access requirements of [`start`] must hold.
pub unsafe fn step(commands: [Command; 2]) -> Result<Snapshot, Error> {
    let mut boundary = boundary()?;
    let next = boundary.tick.checked_add(1).ok_or(Error::TickLimit)?;
    let mut raw = [
        commands[0].raw(),
        commands[1].raw(),
        Command::default().raw(),
        Command::default().raw(),
    ];
    let slot = boundary.tick as usize % d_loop::BACKUPTICS;
    for (index, command) in raw.iter_mut().enumerate().take(2) {
        // Both commands are local. Doom's ring still advances exactly as usual.
        command.consistancy = g_game::consistancy[index][slot];
    }
    let _restore = Commands(d_net::netcmds);
    d_net::netcmds = raw.as_mut_ptr();
    g_game::G_Ticker();
    boundary.tick = next;
    boundary.restore();
    snapshot()
}

/// Captures both players without borrowing engine storage.
///
/// # Safety
/// The caller must hold exclusive initialized engine access between tics.
pub unsafe fn snapshot() -> Result<Snapshot, Error> {
    boundary()?;
    Ok(Snapshot {
        tick: u64::try_from(p_tick::leveltime).map_err(|_| Error::State)?,
        players: [player_state(Player::One)?, player_state(Player::Two)?],
    })
}

fn frag_sum(row: [i32; 4], player: usize) -> i64 {
    row.into_iter()
        .take(2)
        .enumerate()
        .map(|(index, frags)| {
            if index == player {
                -i64::from(frags)
            } else {
                i64::from(frags)
            }
        })
        .sum()
}

unsafe fn player_state(player: Player) -> Result<PlayerState, Error> {
    let index = player.index();
    let player = &*std::ptr::addr_of!(g_game::players)
        .cast::<d_player::PlayerT>()
        .add(index);
    let object = player
        .mo
        .cast::<super::p_telept::mobj_t>()
        .as_ref()
        .ok_or(Error::State)?;
    Ok(PlayerState {
        health: player.health,
        armor: player.armorpoints,
        ammo: player.ammo,
        frags: frag_sum(player.frags, index),
        monster_kills: player.killcount,
        alive: player.playerstate == 0 && player.health > 0,
        x: object.x,
        y: object.y,
        angle: object.angle,
    })
}

/// Draws a selected player's world and weapon without stepping the game.
/// The host supplies score and health UI; Doom's cached single-player HUD is omitted.
///
/// # Safety
/// The caller must hold exclusive initialized engine access with graphics ready.
pub unsafe fn render(player: Player) -> Result<(), Error> {
    render_at(player, Viewport::NATIVE)
}

/// Draws a requested native-aspect raster without changing the world or camera.
///
/// # Safety
/// The caller holds exclusive initialized engine access with graphics ready.
pub unsafe fn render_at(player: Player, viewport: Viewport) -> Result<(), Error> {
    boundary()?;
    let player = std::ptr::addr_of_mut!(g_game::players)
        .cast::<d_player::PlayerT>()
        .add(player.index());
    if (*player).mo.is_null() {
        return Err(Error::State);
    }
    if viewport != VIEWPORT {
        super::r_main::arena_viewport(viewport, NATIVE_CLIP_ANGLE);
        VIEWPORT = viewport;
    }
    super::r_main::R_RenderPlayerView(player);
    let palette = super::w_wad::W_CacheLumpName(c"PLAYPAL".as_ptr(), super::z_zone::PU_CACHE);
    super::i_video::I_SetPalette(palette.cast());
    super::i_video::I_FinishUpdate();
    Ok(())
}

/// Captures driver state to accompany a complete world checkpoint.
///
/// # Safety
/// The caller must hold exclusive initialized engine access between tics.
pub unsafe fn capture() -> Result<State, Error> {
    boundary()?;
    Ok(State {
        config: CONFIG.ok_or(Error::State)?,
        consistency: std::ptr::addr_of!(g_game::consistancy).read(),
    })
}

/// Restores driver state after the matching world and loop counters are restored.
///
/// # Safety
/// The state must be an unchanged trusted capture from this engine revision.
/// Both players and the same map must already be restored under exclusive access.
pub unsafe fn restore(state: &State) {
    CONFIG = Some(state.config);
    g_game::consistancy = state.consistency;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoreboard_subtracts_native_self_frags() {
        assert_eq!(frag_sum([2, 5, 0, 0], 0), 3);
        assert_eq!(frag_sum([1, 3, 0, 0], 1), -2);
        assert_eq!(frag_sum([1, 3, 99, 99], 1), -2);
        assert_eq!(frag_sum([i32::MIN, i32::MAX, 0, 0], 0), 4_294_967_295);
    }
}
