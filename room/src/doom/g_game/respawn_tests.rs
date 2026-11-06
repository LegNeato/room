use super::*;
use crate::doom::{c_ffi, info, m_fixed::FRACUNIT, p_setup, p_tick, z_zone};

const CASE: &str = "ROOM_RESPAWN_FOG_CASE";

#[test]
fn respawn_spawns_teleport_fog_for_every_start_angle() {
    run_case("respawn");
}

#[test]
fn blocked_respawn_does_not_spawn_fog_or_evict_a_corpse() {
    run_case("blocked");
}

#[test]
fn initial_spawn_does_not_spawn_fog() {
    run_case("initial");
}

#[test]
fn occupied_initial_spawn_is_rejected_without_fog() {
    run_case("occupied_initial");
}

fn run_case(case: &str) {
    // The simulation uses process globals; each fixture gets a fresh process.
    let module = module_path!().split_once("::").unwrap().1;
    let test = format!("{module}::respawn_process");
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", &test, "--nocapture"])
        .env(CASE, case)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{case}: {}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    );
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("1 passed; 0 failed"),
        "the isolated respawn fixture did not execute: {}",
        String::from_utf8_lossy(&result.stdout),
    );
}

#[test]
#[ignore = "runs in an isolated process because the simulation owns process globals"]
fn respawn_process() {
    let case = std::env::var(CASE).unwrap();
    let mut sector: c_ffi::sector_t = unsafe { std::mem::zeroed() };
    sector.floorheight = 104 * FRACUNIT;
    sector.ceilingheight = 232 * FRACUNIT;
    let mut subsector: c_ffi::subsector_t = unsafe { std::mem::zeroed() };
    subsector.sector = (&mut sector as *mut c_ffi::sector_t).cast();
    let mut corpse: mobj_t = unsafe { std::mem::zeroed() };
    corpse.x = 64 * FRACUNIT;
    corpse.y = 64 * FRACUNIT;
    corpse.radius = 16 * FRACUNIT;
    let mut blocker: mobj_t = unsafe { std::mem::zeroed() };
    blocker.x = corpse.x;
    blocker.y = corpse.y;
    blocker.radius = corpse.radius;
    blocker.flags = info::MF_SOLID;
    let mut blocklinks = [ptr::null_mut()];
    let mut blockmap = [0_i16];
    let mut lines = [-1_i16];
    let mut start = mapthing_t {
        x: 64,
        y: 64,
        angle: 0,
        r#type: 1,
        options: 0,
    };

    unsafe {
        z_zone::Z_Init();
        p_tick::P_InitThinkers();
        p_setup::numsubsectors = 1;
        p_setup::subsectors = &mut subsector;
        p_setup::numnodes = 0;
        p_setup::bmapwidth = 1;
        p_setup::bmapheight = 1;
        p_setup::blocklinks = blocklinks.as_mut_ptr();
        p_setup::blockmap = blockmap.as_mut_ptr();
        p_setup::blockmaplump = lines.as_mut_ptr();
        players[0].mo = (&mut corpse as *mut mobj_t).cast();
        // G_CheckSpot's normal initial-frame sentinel suppresses sound; the
        // fixture exercises real collision, allocation and thinker linking.
        players[0].viewz = 1;
        consoleplayer = 0;
        let cap = &raw mut p_tick::thinkercap;

        match case.as_str() {
            "respawn" => {
                // These direction pairs are the original Doom table samples
                // at the eight start angles, including its tangent overflow
                // emulation. Widen first, then truncate to model C's 32-bit
                // result independently of the production wrapping operations.
                for (angle, direction) in [
                    (0, [65_535, 25]),
                    (45, [46_323, 46_358]),
                    (90, [-25, 65_535]),
                    (135, [-46_358, 46_323]),
                    (180, [25, -170_910_304]),
                    (225, [65_586, -65_485]),
                    (270, [25, 25]),
                    (315, [46_358, 65_586]),
                ] {
                    start.angle = angle;
                    assert_eq!(G_CheckSpot(0, &mut start), 1);
                    let spawned = (*cap).prev.cast::<mobj_t>();
                    assert_ne!(spawned.cast(), cap);
                    assert_ne!(
                        (*spawned).mobjtype,
                        info::MT_SPAWNSHOT,
                        "respawn must not create a boss cube",
                    );
                    assert_eq!((*spawned).mobjtype, info::MT_TFOG);
                    assert_eq!((*spawned).sprite, info::SPR_TFOG);
                    let expected = direction
                        .map(|value| (64 * i64::from(FRACUNIT) + 20 * i64::from(value)) as i32);
                    assert_eq!([(*spawned).x, (*spawned).y], expected);
                    assert_eq!(
                        (*spawned).state.cast(),
                        &raw mut info::states[info::S_TFOG as usize]
                    );
                    assert_eq!((*spawned).z, sector.floorheight);
                    let expected_tics = info::states[info::S_TFOG as usize].tics;
                    assert_eq!((*spawned).tics, expected_tics);
                }
                let mut count = 0;
                let mut thinker = (*cap).next;
                while thinker != cap {
                    assert!(count < 8, "unexpected thinker or broken list");
                    assert_eq!((*thinker.cast::<mobj_t>()).mobjtype, info::MT_TFOG);
                    count += 1;
                    thinker = (*thinker).next;
                }
                assert_eq!(count, 8);
                let queued = bodyqueslot;
                assert_eq!(queued, 8);
            }
            "blocked" => {
                p_setup::blocklinks.write((&mut blocker as *mut mobj_t).cast());
                assert_eq!(G_CheckSpot(0, &mut start), 0);
                assert_eq!((*cap).next, cap);
                let queued = bodyqueslot;
                assert_eq!(queued, 0);
            }
            "initial" => {
                players[0].mo = ptr::null_mut();
                assert_eq!(G_CheckSpot(0, &mut start), 1);
                assert_eq!((*cap).next, cap);
            }
            "occupied_initial" => {
                assert_eq!(G_CheckSpot(1, &mut start), 0);
                assert_eq!((*cap).next, cap);
            }
            _ => panic!("unknown respawn fixture"),
        }
        z_zone::Z_CheckHeap();
    }
}
