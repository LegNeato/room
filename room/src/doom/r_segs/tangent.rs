use super::super::tables::{finesine, finetangent, ANGLETOFINESHIFT};

/// Resolves a wall angle using the original tangent-then-sine table layout.
/// Some visible wall angles exceed the tangent table. The C renderer reads
/// into the following sine table. Use explicit indexing to preserve that
/// behavior without relying on an out-of-bounds read or linker placement.
#[inline]
pub(super) fn lookup(angle: u32) -> i32 {
    let index = (angle >> ANGLETOFINESHIFT) as usize;
    if index < finetangent.len() {
        finetangent[index]
    } else {
        finesine[index - finetangent.len()]
    }
}

const _: () =
    assert!(((u32::MAX >> ANGLETOFINESHIFT) as usize) < finetangent.len() + finesine.len());

#[cfg(test)]
mod tests;
