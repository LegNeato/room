use super::*;

fn scalar(line: Line, point: [i32; 2], trace: bool) -> i32 {
    let [x, y] = point;
    let [origin_x, origin_y] = line.origin;
    let [line_x, line_y] = line.direction;
    if line_x == 0 {
        return if x <= origin_x {
            i32::from(line_y > 0)
        } else {
            i32::from(line_y < 0)
        };
    }
    if line_y == 0 {
        return if y <= origin_y {
            i32::from(line_x < 0)
        } else {
            i32::from(line_x > 0)
        };
    }
    let dx = x.wrapping_sub(origin_x);
    let dy = y.wrapping_sub(origin_y);
    let multiply = |a: i32, b: i32| ((i64::from(a) * i64::from(b)) >> 16) as i32;
    let (left, right) = if trace {
        if (line_y ^ line_x ^ dx ^ dy) < 0 {
            return i32::from((line_y ^ dx) < 0);
        }
        (
            multiply(line_y >> 8, dx >> 8),
            multiply(dy >> 8, line_x >> 8),
        )
    } else {
        (multiply(line_y >> 16, dx), multiply(dy, line_x >> 16))
    };
    i32::from(right >= left)
}

fn compare(line: Line, points: [[i32; 2]; 2]) {
    assert_eq!(
        line.sides(points),
        points.map(|point| scalar(line, point, false))
    );
    assert_eq!(
        line.trace_sides(points),
        points.map(|point| scalar(line, point, true))
    );
}

#[test]
fn axial_and_degenerate_lines_keep_boundary_rules() {
    for direction in [[0, 0], [0, 65536], [0, -65536], [65536, 0], [-65536, 0]] {
        for x in [-8, -7, -6] {
            for y in [10, 11, 12] {
                compare(
                    Line {
                        origin: [-7, 11],
                        direction,
                    },
                    [[x, y], [-x, -y]],
                );
            }
        }
    }
}

#[test]
fn diagonal_lines_keep_fixed_point_rounding() {
    let edge = [
        i32::MIN,
        i32::MIN + 1,
        -65537,
        -65536,
        -1,
        0,
        1,
        65535,
        65536,
        i32::MAX,
    ];
    for dx in edge {
        for dy in edge {
            for x in edge {
                compare(
                    Line {
                        origin: [17, -29],
                        direction: [dx, dy],
                    },
                    [[x, x], [x, x.wrapping_neg()]],
                );
            }
        }
    }
}

#[test]
fn paired_points_match_scalar_over_full_integer_range() {
    let mut state = 0x67ab_018d_u32;
    let mut next = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state as i32
    };
    for _ in 0..10000 {
        compare(
            Line {
                origin: [next(), next()],
                direction: [next(), next()],
            },
            [[next(), next()], [next(), next()]],
        );
    }
}
