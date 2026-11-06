use super::*;

const CHILD: &str = "ROOM_SPRITE_PROJECTION_CASE";

#[test]
fn missing_sprite_frames_are_skipped_without_range_checks() {
    projection_case("missing");
}

#[test]
fn present_sprite_frames_remain_readable() {
    projection_case("present");
}

fn projection_case(case: &str) {
    // Other renderer tests also use process globals. Keep this fixture's
    // deliberately minimal projection state in its own process.
    let module = module_path!().split_once("::").unwrap().1;
    let test = format!("{module}::project_sprite_process");
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", &test, "--nocapture"])
        .env(CHILD, case)
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
        "the isolated projection test did not execute: {}",
        String::from_utf8_lossy(&result.stdout),
    );
}

#[test]
#[ignore = "runs in a child process because the renderer owns process globals"]
fn project_sprite_process() {
    let case = std::env::var(CHILD).unwrap();
    let present = match case.as_str() {
        "missing" => false,
        "present" => true,
        _ => panic!("unknown sprite projection fixture"),
    };
    let mut frame = spriteframe_t {
        rotate: 0,
        lump: [0; 8],
        flip: [0; 8],
    };
    let mut definition = spritedef_t {
        numframes: i32::from(present),
        spriteframes: if present { &mut frame } else { ptr::null_mut() },
    };
    let mut offset = 0;
    let mut thing: crate::doom::c_ffi::mobj_t = unsafe { std::mem::zeroed() };
    thing.x = 8 * FRACUNIT;
    // Scene09 failed on an absent sprite's fourth frame. The high bit is a
    // lighting flag and must not change the missing-definition decision.
    thing.frame = if present {
        FF_FULLBRIGHT
    } else {
        4 | FF_FULLBRIGHT
    };
    unsafe {
        sprites = (&mut definition as *mut spritedef_t).cast();
        numsprites = 1;
        viewx = 0;
        viewy = 0;
        viewcos = FRACUNIT;
        viewsin = 0;
        projection = FRACUNIT;
        centerxfrac = 0;
        // A present frame takes the ordinary off-screen return after reading
        // its descriptor and offset; no other renderer storage is required.
        viewwidth = -1;
        spriteoffset = &mut offset;
        R_ClearSprites();
        let before = vissprite_p;
        R_ProjectSprite((&mut thing as *mut crate::doom::c_ffi::mobj_t).cast());
        let after = vissprite_p;
        assert_eq!(after, before);
    }
    assert_eq!(thing.x, 8 * FRACUNIT);
    assert_eq!(definition.numframes, i32::from(present));
    assert_eq!(frame.rotate, 0);
}
