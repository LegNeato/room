use super::*;

#[test]
fn boxes_require_overlap_on_every_axis() {
    let a = [10, -10, -10, 10];
    assert!(boxes_overlap(a, [5, -5, -5, 5]));
    assert!(boxes_overlap(a, a));
    for b in [
        [20, 10, -5, 5],
        [-10, -20, -5, 5],
        [5, -5, 10, 20],
        [5, -5, -20, -10],
    ] {
        assert!(!boxes_overlap(a, b));
        assert!(!boxes_overlap(b, a));
    }
}

#[test]
fn things_require_overlap_on_both_axes() {
    assert!(things_overlap([0, 0], [9, -9], 10));
    for point in [[10, 0], [-10, 0], [0, 10], [0, -10], [20, 20]] {
        assert!(!things_overlap([0, 0], point, 10));
    }
    assert!(!things_overlap([0, 0], [0, 0], 0));
}

#[test]
fn comparisons_match_scalar_including_wrapping_coordinates() {
    let edge = [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX];
    for x in edge {
        for y in edge {
            for distance in edge {
                let expected = x.wrapping_sub(17).wrapping_abs() < distance
                    && y.wrapping_sub(-29).wrapping_abs() < distance;
                assert_eq!(things_overlap([x, y], [17, -29], distance), expected);
            }
            let a = [x, y, y, x];
            let b = [y, x, x, y];
            let expected = a[3] > b[2] && a[2] < b[3] && a[0] > b[1] && a[1] < b[0];
            assert_eq!(boxes_overlap(a, b), expected);
        }
    }
}
