use super::{info, scoped, StartupLog};

#[test]
fn startup_output_is_scoped_and_restored_after_unwind() {
    // This test owns the only active startup scope in its process. Keeping all
    // stateful cases together avoids concurrent access to ordinary host globals.
    assert_eq!(StartupLog::default(), StartupLog::Normal);
    let mut messages = Vec::new();
    unsafe {
        info(|| messages.push("default"));
        scoped(StartupLog::Quiet, || {
            info(|| panic!("quiet startup must not evaluate output"));
            scoped(StartupLog::Normal, || {
                info(|| messages.push("nested normal"))
            });
            info(|| panic!("the outer quiet scope must be restored"));
        });
        info(|| messages.push("after startup"));
        scoped(StartupLog::Normal, || info(|| messages.push("normal")));
    }
    assert_eq!(
        messages,
        ["default", "nested normal", "after startup", "normal"]
    );
    let failed = std::panic::catch_unwind(|| unsafe {
        scoped(StartupLog::Quiet, || panic!("initialization failed"));
    });
    assert!(failed.is_err());
    unsafe { info(|| messages.push("after failure")) };
    assert_eq!(messages.last(), Some(&"after failure"));
}
