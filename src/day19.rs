#[allow(unused_imports)]
use super::prelude::*;
type Input = Vec<Vec<[i16; 3]>>;

pub fn input_generator(input: &str) -> Input {
    input
        .split("\n\n")
        .map(|beacons| {
            beacons
                .lines()
                .skip(1)
                .map(|line| {
                    let (x, rest) = line.split_once(',').unwrap();
                    let (y, z) = rest.split_once(',').unwrap();
                    [x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()]
                })
                .collect()
        })
        .collect()
}

type Rot = [[i16; 3]; 3];

const fn combine_rots(prev: Rot, new: Rot) -> Rot {
    let mut output = [[0; 3]; 3];
    let mut ijk = 0;
    while ijk < 27 {
        let (i, j, k) = (ijk / 9, (ijk / 3) % 3, ijk % 3);
        output[i][j] += new[i][k] * prev[k][j];
        ijk += 1;
    }
    output
}

const NOP: Rot = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
const ROLL: Rot = [[0, 0, 1], [0, 1, 0], [-1, 0, 0]];
const TURN_CW: Rot = [[1, 0, 0], [0, 0, -1], [0, 1, 0]];
const TURN_CCW: Rot = [[1, 0, 0], [0, 0, 1], [0, -1, 0]];
const TURNS: [Rot; 2] = [TURN_CW, TURN_CCW];

const ROTS: [Rot; 24] = {
    let mut m = NOP;
    let mut rots = [NOP; 24];
    let mut ri = 0;
    while ri < 6 {
        m = combine_rots(m, ROLL);
        rots[ri * 4] = m;
        let mut ti = 0;
        while ti < 3 {
            m = combine_rots(m, TURNS[ri % 2]);
            rots[ri * 4 + ti + 1] = m;
            ti += 1;
        }
        ri += 1;
    }
    rots
};

type ResolvedPositions = Vec<([i16; 3], Vec<[i16; 3]>)>;

fn resolve_positions(input: &Input) -> ResolvedPositions {
    let mut num_found = 1;
    let mut found = vec![None; input.len()];
    found[0] = Some(([0; 3], input[0].clone()));

    let mut visited = Grid {
        vec: vec![false; input.len() * input.len()],
        width: input.len(),
    };

    let distances = input
        .iter()
        .map(|points| {
            (0..points.len())
                .flat_map(|i| (i + 1..points.len()).map(move |j| (points[i], points[j])))
                .map(|(p1, p2)| {
                    p1.iter()
                        .zip(p2)
                        .map(|(&d1, d2)| (d1 - d2).abs())
                        .sum::<i16>()
                })
                .collect::<FxHashSet<_>>()
        })
        .collect::<Vec<_>>();

    let rotated_points = input
        .iter()
        .map(|points| {
            ROTS.map(|rot| {
                points
                    .iter()
                    .map(|&p| rot.map(|l| l.into_iter().zip(p).map(|(m, v)| m * v).sum::<i16>()))
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    while num_found < input.len() {
        'i: for i in 0..input.len() {
            if found[i].is_some() {
                continue 'i;
            }
            'j: for j in 0..input.len() {
                if found[j].is_none() || replace(&mut visited[(i, j)], true) {
                    continue 'j;
                }

                let common_distances = distances[i].intersection(&distances[j]).count();
                if common_distances < 12 * 11 / 2 {
                    continue 'j;
                }

                found[i] = (11..rotated_points[i][0].len())
                    .into_par_iter()
                    .flat_map(|point_idx| rotated_points[i].par_iter().map(move |points| (points, points[point_idx])))
                    .find_map_any(|(points, point)| {
                        let candidates = &found[j].as_ref().unwrap().1;
                        for &cand in candidates.iter().take(candidates.len() - 11) {
                            let root = [cand[0] - point[0], cand[1] - point[1], cand[2] - point[2]];
                            let mapped_points = points
                                .iter()
                                .map(|&p| [p[0] + root[0], p[1] + root[1], p[2] + root[2]]);
                            let num_intersections = mapped_points
                                .clone()
                                .filter(|p| candidates.contains(p))
                                .count();
                            if num_intersections >= 12 {
                                return Some((root, mapped_points.collect()));
                            }
                        }
                        None
                    });
                found[i].is_some().then(|| num_found += 1);
                continue 'i;
            }
        }
    }

    found.into_iter().map(Option::unwrap).collect()
}

static PERSISTANT: Lazy<Mutex<Option<ResolvedPositions>>> = Lazy::new(|| Mutex::new(None));

pub fn part1(input: &Input) -> usize {
    let mut resolved_positions = PERSISTANT.lock();
    let resolved_positions = resolved_positions.insert(resolve_positions(input));
    resolved_positions
        .iter()
        .flat_map(|(_, beacons)| beacons.iter().copied())
        .unique()
        .count()
}

pub fn part2(input: &Input) -> i16 {
    let resolved_positions = PERSISTANT.lock();
    let found = resolved_positions.as_ref().expect("Part 1 didn't run");
    (0..input.len())
        .flat_map(|i| (i + 1..input.len()).map(move |j| (i, j)))
        .map(|(i, j)| (found[i].0, found[j].0))
        .map(|(p1, p2)| (p1[0] - p2[0]).abs() + (p1[1] - p2[1]).abs() + (p1[2] - p2[2]).abs())
        .max()
        .unwrap()
}
