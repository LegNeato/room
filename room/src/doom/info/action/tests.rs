use super::*;
use crate::doom::info::{self, State, NUMSTATES, S_BFGFLASH2, S_BLOOD1, S_NULL};
use crate::doom::{p_enemy, p_pspr};
use std::cell::Cell;
use std::mem::{align_of, offset_of, size_of, MaybeUninit};

thread_local! {
    static CALLED: Cell<[usize; 3]> = const { Cell::new([0; 3]) };
}

unsafe extern "C" fn actor_probe(actor: *mut mobj_t) {
    CALLED.with(|called| called.set([1, actor as usize, 0]));
}

unsafe extern "C" fn weapon_probe(player: *mut PlayerT, psprite: *mut PspdefT) {
    CALLED.with(|called| called.set([2, player as usize, psprite as usize]));
}

#[test]
fn actor_callback_receives_its_actor_pointer() {
    let mut actor = MaybeUninit::<mobj_t>::uninit();
    let pointer = actor.as_mut_ptr();
    let action = Action::for_actor(actor_probe);
    // The probe records the pointer and deliberately does not dereference it.
    unsafe { action.actor().unwrap()(pointer) };
    assert_eq!(CALLED.with(Cell::get), [1, pointer as usize, 0]);
}

#[test]
fn weapon_callback_receives_both_distinct_pointers() {
    let mut player = MaybeUninit::<PlayerT>::uninit();
    let mut psprite = MaybeUninit::<PspdefT>::uninit();
    let player = player.as_mut_ptr();
    let psprite = psprite.as_mut_ptr();
    let action = Action::for_weapon(weapon_probe);
    unsafe { action.weapon().unwrap()(player, psprite) };
    assert_eq!(
        CALLED.with(Cell::get),
        [2, player as usize, psprite as usize]
    );
}

#[test]
fn absent_action_is_null_for_both_call_sites() {
    assert!(unsafe { Action::NONE.actor() }.is_none());
    assert!(unsafe { Action::NONE.weapon() }.is_none());
    assert_eq!(storage(Action::NONE), [0; 8]);
}

#[test]
fn action_preserves_the_c_state_storage_layout() {
    assert_eq!(size_of::<Action>(), 8);
    assert_eq!(align_of::<Action>(), 8);
    assert_eq!(size_of::<State>(), 40);
    assert_eq!(offset_of!(State, action), 16);
    assert_eq!(offset_of!(State, nextstate), 24);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    None,
    Actor,
    Weapon,
}

fn expected(name: &str) -> (Kind, Action) {
    match name {
        "NULL" => (Kind::None, Action::NONE),
        "A_BFGSpray" => (Kind::Actor, Action::for_actor(p_pspr::A_BFGSpray)),
        "A_BFGsound" => (Kind::Weapon, Action::for_weapon(p_pspr::A_BFGsound)),
        "A_BabyMetal" => (Kind::Actor, Action::for_actor(p_enemy::A_BabyMetal)),
        "A_BossDeath" => (Kind::Actor, Action::for_actor(p_enemy::A_BossDeath)),
        "A_BrainAwake" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainAwake)),
        "A_BrainDie" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainDie)),
        "A_BrainExplode" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainExplode)),
        "A_BrainPain" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainPain)),
        "A_BrainScream" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainScream)),
        "A_BrainSpit" => (Kind::Actor, Action::for_actor(p_enemy::A_BrainSpit)),
        "A_BruisAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_BruisAttack)),
        "A_BspiAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_BspiAttack)),
        "A_CPosAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_CPosAttack)),
        "A_CPosRefire" => (Kind::Actor, Action::for_actor(p_enemy::A_CPosRefire)),
        "A_Chase" => (Kind::Actor, Action::for_actor(p_enemy::A_Chase)),
        "A_CheckReload" => (Kind::Weapon, Action::for_weapon(p_pspr::A_CheckReload)),
        "A_CloseShotgun2" => (Kind::Weapon, Action::for_weapon(p_enemy::A_CloseShotgun2)),
        "A_CyberAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_CyberAttack)),
        "A_Explode" => (Kind::Actor, Action::for_actor(p_enemy::A_Explode)),
        "A_FaceTarget" => (Kind::Actor, Action::for_actor(p_enemy::A_FaceTarget)),
        "A_Fall" => (Kind::Actor, Action::for_actor(p_enemy::A_Fall)),
        "A_FatAttack1" => (Kind::Actor, Action::for_actor(p_enemy::A_FatAttack1)),
        "A_FatAttack2" => (Kind::Actor, Action::for_actor(p_enemy::A_FatAttack2)),
        "A_FatAttack3" => (Kind::Actor, Action::for_actor(p_enemy::A_FatAttack3)),
        "A_FatRaise" => (Kind::Actor, Action::for_actor(p_enemy::A_FatRaise)),
        "A_Fire" => (Kind::Actor, Action::for_actor(p_enemy::A_Fire)),
        "A_FireBFG" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FireBFG)),
        "A_FireCGun" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FireCGun)),
        "A_FireCrackle" => (Kind::Actor, Action::for_actor(p_enemy::A_FireCrackle)),
        "A_FireMissile" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FireMissile)),
        "A_FirePistol" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FirePistol)),
        "A_FirePlasma" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FirePlasma)),
        "A_FireShotgun" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FireShotgun)),
        "A_FireShotgun2" => (Kind::Weapon, Action::for_weapon(p_pspr::A_FireShotgun2)),
        "A_GunFlash" => (Kind::Weapon, Action::for_weapon(p_pspr::A_GunFlash)),
        "A_HeadAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_HeadAttack)),
        "A_Hoof" => (Kind::Actor, Action::for_actor(p_enemy::A_Hoof)),
        "A_KeenDie" => (Kind::Actor, Action::for_actor(p_enemy::A_KeenDie)),
        "A_Light0" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Light0)),
        "A_Light1" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Light1)),
        "A_Light2" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Light2)),
        "A_LoadShotgun2" => (Kind::Weapon, Action::for_weapon(p_enemy::A_LoadShotgun2)),
        "A_Look" => (Kind::Actor, Action::for_actor(p_enemy::A_Look)),
        "A_Lower" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Lower)),
        "A_Metal" => (Kind::Actor, Action::for_actor(p_enemy::A_Metal)),
        "A_OpenShotgun2" => (Kind::Weapon, Action::for_weapon(p_enemy::A_OpenShotgun2)),
        "A_Pain" => (Kind::Actor, Action::for_actor(p_enemy::A_Pain)),
        "A_PainAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_PainAttack)),
        "A_PainDie" => (Kind::Actor, Action::for_actor(p_enemy::A_PainDie)),
        "A_PlayerScream" => (Kind::Actor, Action::for_actor(p_enemy::A_PlayerScream)),
        "A_PosAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_PosAttack)),
        "A_Punch" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Punch)),
        "A_Raise" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Raise)),
        "A_ReFire" => (Kind::Weapon, Action::for_weapon(p_pspr::A_ReFire)),
        "A_SPosAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_SPosAttack)),
        "A_SargAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_SargAttack)),
        "A_Saw" => (Kind::Weapon, Action::for_weapon(p_pspr::A_Saw)),
        "A_Scream" => (Kind::Actor, Action::for_actor(p_enemy::A_Scream)),
        "A_SkelFist" => (Kind::Actor, Action::for_actor(p_enemy::A_SkelFist)),
        "A_SkelMissile" => (Kind::Actor, Action::for_actor(p_enemy::A_SkelMissile)),
        "A_SkelWhoosh" => (Kind::Actor, Action::for_actor(p_enemy::A_SkelWhoosh)),
        "A_SkullAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_SkullAttack)),
        "A_SpawnFly" => (Kind::Actor, Action::for_actor(p_enemy::A_SpawnFly)),
        "A_SpawnSound" => (Kind::Actor, Action::for_actor(p_enemy::A_SpawnSound)),
        "A_SpidRefire" => (Kind::Actor, Action::for_actor(p_enemy::A_SpidRefire)),
        "A_StartFire" => (Kind::Actor, Action::for_actor(p_enemy::A_StartFire)),
        "A_Tracer" => (Kind::Actor, Action::for_actor(p_enemy::A_Tracer)),
        "A_TroopAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_TroopAttack)),
        "A_VileAttack" => (Kind::Actor, Action::for_actor(p_enemy::A_VileAttack)),
        "A_VileChase" => (Kind::Actor, Action::for_actor(p_enemy::A_VileChase)),
        "A_VileStart" => (Kind::Actor, Action::for_actor(p_enemy::A_VileStart)),
        "A_VileTarget" => (Kind::Actor, Action::for_actor(p_enemy::A_VileTarget)),
        "A_WeaponReady" => (Kind::Weapon, Action::for_weapon(p_pspr::A_WeaponReady)),
        "A_XScream" => (Kind::Actor, Action::for_actor(p_enemy::A_XScream)),
        name => panic!("unknown C callback {name}"),
    }
}

fn matches_graph(index: usize, kind: Kind) -> bool {
    match kind {
        Kind::None => true,
        Kind::Weapon => index > S_NULL as usize && index <= S_BFGFLASH2 as usize,
        Kind::Actor => index >= S_BLOOD1 as usize && index < NUMSTATES,
    }
}

// Inspect initialized union storage rather than reading a possibly wrong
// function signature if a future table edit violates the tested invariant.
fn storage(action: Action) -> [u8; 8] {
    unsafe { std::ptr::addr_of!(action).cast::<[u8; 8]>().read() }
}

#[test]
fn every_callback_matches_the_c_table_and_state_graph_kind() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let source = std::fs::read_to_string(
        std::path::Path::new(&manifest).join("../vendor/doomgeneric/info.c"),
    )
    .unwrap();
    let table = source.split_once("states[NUMSTATES] = {").unwrap().1;
    let table = table.split_once("};").unwrap().0;
    let names: Vec<_> = table
        .lines()
        .filter(|line| line.trim_start().starts_with("{SPR_"))
        .map(|line| {
            line.split_once(",{ ")
                .map_or_else(|| line.split_once(",{").unwrap().1, |(_, rest)| rest)
                .split_once('}')
                .unwrap()
                .0
                .trim()
        })
        .collect();
    assert_eq!(names.len(), NUMSTATES);
    let mut counts = [0; 3];
    let mut unique = std::collections::BTreeSet::new();
    for (index, name) in names.into_iter().enumerate() {
        let (kind, action) = expected(name);
        assert!(
            matches_graph(index, kind),
            "state {index}: {name} has wrong arity"
        );
        let actual =
            unsafe { (*std::ptr::addr_of!(info::states).cast::<State>().add(index)).action };
        assert_eq!(storage(actual), storage(action), "state {index}: {name}");
        counts[match kind {
            Kind::None => 0,
            Kind::Actor => 1,
            Kind::Weapon => 2,
        }] += 1;
        if kind != Kind::None {
            unique.insert(name);
        }
    }
    assert_eq!(counts, [519, 378, 70]);
    assert_eq!(unique.len(), 74);
}

#[test]
fn opposite_state_kinds_are_rejected_by_the_table_contract() {
    assert!(!matches_graph(S_BFGFLASH2 as usize, Kind::Actor));
    assert!(!matches_graph(S_BLOOD1 as usize, Kind::Weapon));
    assert!(!matches_graph(S_NULL as usize, Kind::Weapon));
    assert!(!matches_graph(NUMSTATES, Kind::Actor));
}

#[test]
fn normal_state_links_and_entry_tables_stay_within_their_calling_kind() {
    for index in 1..NUMSTATES {
        let next =
            unsafe { (*std::ptr::addr_of!(info::states).cast::<State>().add(index)).nextstate };
        assert!(next >= 0 && (next as usize) < NUMSTATES);
        if next != S_NULL {
            assert_eq!(
                index <= S_BFGFLASH2 as usize,
                next <= S_BFGFLASH2,
                "state {index}"
            );
        }
    }
    for index in 0..info::NUMMOBJTYPES {
        let info = unsafe {
            &*std::ptr::addr_of!(info::mobjinfo)
                .cast::<info::MobjInfo>()
                .add(index)
        };
        for state in [
            info.spawnstate,
            info.seestate,
            info.painstate,
            info.meleestate,
            info.missilestate,
            info.deathstate,
            info.xdeathstate,
            info.raisestate,
        ] {
            assert!(
                state == S_NULL || matches_graph(state as usize, Kind::Actor),
                "mobj {index}"
            );
        }
    }
    for weapon in &crate::doom::d_items::weaponinfo {
        for state in [
            weapon.upstate,
            weapon.downstate,
            weapon.readystate,
            weapon.atkstate,
            weapon.flashstate,
        ] {
            assert!(state == S_NULL || matches_graph(state as usize, Kind::Weapon));
        }
    }
}
