use super::{super::w_wad, Error};

const MAP_THING_BYTES: usize = 10;
const MAX_THINGS_BYTES: usize = 1024 * 1024;

fn starts(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_THINGS_BYTES || !bytes.len().is_multiple_of(MAP_THING_BYTES) {
        return Err(Error::Map);
    }
    let mut players = [false; 2];
    let mut deathmatch = 0;
    for thing in bytes.as_chunks::<MAP_THING_BYTES>().0 {
        match u16::from_le_bytes([thing[6], thing[7]]) {
            1 => players[0] = true,
            2 => players[1] = true,
            11 => deathmatch += 1,
            _ => {}
        }
    }
    if players == [true; 2] && deathmatch >= 4 {
        Ok(())
    } else {
        Err(Error::Map)
    }
}

pub(super) unsafe fn validate() -> Result<(), Error> {
    let marker = w_wad::W_CheckNumForName(c"E1M1".as_ptr());
    let things = u32::try_from(marker).ok().and_then(|v| v.checked_add(1));
    let things = things.filter(|v| *v < w_wad::numlumps).ok_or(Error::Map)?;
    let info = &*w_wad::lumpinfo.add(things as usize);
    if !info
        .name
        .iter()
        .zip(b"THINGS\0\0")
        .all(|(found, wanted)| i32::from(*found) == i32::from(*wanted))
    {
        return Err(Error::Map);
    }
    let length = usize::try_from(w_wad::W_LumpLength(things)).map_err(|_| Error::Map)?;
    if length > MAX_THINGS_BYTES {
        return Err(Error::Map);
    }
    let mut bytes = vec![0; length];
    w_wad::W_ReadLump(things, bytes.as_mut_ptr().cast());
    starts(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn things(types: &[u16]) -> Vec<u8> {
        types
            .iter()
            .flat_map(|kind| {
                let mut thing = [0; MAP_THING_BYTES];
                thing[6..8].copy_from_slice(&kind.to_le_bytes());
                thing
            })
            .collect()
    }

    #[test]
    fn validates_spawn_points_before_loading_the_level() {
        assert_eq!(starts(&things(&[1, 2, 11, 11, 11, 11])), Ok(()));
        assert_eq!(starts(&things(&[1, 11, 11, 11, 11])), Err(Error::Map));
        assert_eq!(starts(&things(&[1, 2, 11, 11, 11])), Err(Error::Map));
        assert_eq!(starts(&[0; 9]), Err(Error::Map));
    }
}
