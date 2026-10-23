use std::path::Path;

use super::SOURCE_ROOT;

#[test]
fn source_root_locates_the_engine_modules() {
    let root = Path::new(SOURCE_ROOT);
    assert!(root.is_absolute());
    for module in ["doom.rs", "headless.rs", "types.rs"] {
        assert!(root.join("room/src").join(module).is_file(), "{module}");
    }
    assert!(root.join("room/src/doom/r_segs/tangent.rs").is_file());
}
