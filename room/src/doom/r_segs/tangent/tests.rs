use super::{finesine, finetangent, lookup, ANGLETOFINESHIFT};

#[test]
fn ordinary_angles_keep_the_tangent_values() {
    for (index, &value) in finetangent.iter().enumerate() {
        assert_eq!(lookup((index as u32) << ANGLETOFINESHIFT), value);
    }
}

#[test]
fn every_angle_cell_uses_the_historical_table_order() {
    let tables: Vec<_> = finetangent.iter().chain(finesine.iter()).copied().collect();
    let last_index = (u32::MAX >> ANGLETOFINESHIFT) as usize;
    let low_bits = (1 << ANGLETOFINESHIFT) - 1;
    for (index, &value) in tables.iter().enumerate().take(last_index + 1) {
        let angle = (index as u32) << ANGLETOFINESHIFT;
        assert_eq!(lookup(angle), value);
        assert_eq!(lookup(angle | low_bits), value);
    }
}

#[test]
fn half_turn_crosses_into_the_sine_table() {
    let half_turn = 1 << 31;
    assert_eq!(lookup(half_turn - 1), finetangent[finetangent.len() - 1]);
    assert_eq!(lookup(half_turn), finesine[0]);
    assert_eq!(lookup(u32::MAX), finesine[finetangent.len() - 1]);
}

#[test]
fn overflow_does_not_clamp_or_wrap_the_tangent_index() {
    let half_turn = 1 << 31;
    assert_ne!(lookup(half_turn), finetangent[0]);
    assert_ne!(lookup(half_turn), finetangent[finetangent.len() - 1]);
    assert_eq!(lookup(8189 << ANGLETOFINESHIFT), finesine[4093]);
}

#[test]
fn the_original_lookup_reproduces_the_reported_panic() {
    let angle = std::hint::black_box(1_u32 << 31);
    let index = (angle >> ANGLETOFINESHIFT) as usize & 0x1fff;
    assert_eq!(index, finetangent.len());
    assert!(std::panic::catch_unwind(|| finetangent[index]).is_err());
    assert_eq!(lookup(angle), finesine[0]);
}
