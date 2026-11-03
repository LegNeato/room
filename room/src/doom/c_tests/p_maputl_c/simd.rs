use super::*;

#[test]
fn consumed_records_never_call_a_traverser_with_null() {
    let _guard = C_GLOBAL_LOCK.lock().unwrap();
    unsafe {
        let saved_record = p_maputl::intercepts[0];
        let saved_cursor = p_maputl::intercept_p;
        p_maputl::intercepts[0].frac = i32::MAX;
        p_maputl::intercept_p = core::ptr::addr_of_mut!(p_maputl::intercepts)
            .cast::<c_ffi::intercept_t>()
            .add(1);
        assert_eq!(p_maputl::P_TraverseIntercepts(None, i32::MAX), 1);
        p_maputl::intercepts[0] = saved_record;
        p_maputl::intercept_p = saved_cursor;
    }
}
