// Sanitized unit tests retain I_GetTime and need its platform clock callback.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DG_GetTicksMs() -> u32 {
    super::virtual_ms()
}

#[test]
fn platform_clock_reads_virtual_time() {
    let before = super::virtual_ms();
    super::bump_virtual_ms(super::TICK_MS);
    assert_eq!(DG_GetTicksMs(), before + super::TICK_MS);
}
