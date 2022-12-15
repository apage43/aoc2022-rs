use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use std::{collections::HashSet, io};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Pos(isize, isize);
impl Pos {
    fn distance(self, other: Pos) -> usize {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sensor {
    location: Pos,
    nearest_beacon: Pos,
}
impl Sensor {
    fn beacon_radius(self) -> usize {
        self.location.distance(self.nearest_beacon)
    }
}

fn beaconsless_cells_on_row(row: isize, sensors: &[Sensor]) -> usize {
    let mut tbl = 0;
    let beacons: HashSet<Pos> = sensors.iter().map(|s| s.nearest_beacon).collect();
    let min_x_sensor = sensors
        .iter()
        .min_by(|&a, &b| a.location.0.cmp(&b.location.0))
        .expect("empty sensor list");
    let max_x_sensor = sensors
        .iter()
        .max_by(|&a, &b| a.location.0.cmp(&b.location.0))
        .expect("empty sensor list");
    let maxrad: usize = sensors
        .iter()
        .map(|s| s.beacon_radius())
        .max()
        .expect("empty sensor list");
    let xrange =
        min_x_sensor.location.0 - maxrad as isize..max_x_sensor.location.0 + maxrad as isize;
    for x in xrange {
        let test_loc = Pos(x, row);
        if beacons.contains(&test_loc) {
            continue;
        };
        if sensors
            .iter()
            .any(|s| s.location.distance(test_loc) <= s.beacon_radius())
        {
            tbl += 1;
        }
    }
    tbl
}

fn beacon_find(sensors: &[Sensor], minpos: Pos, maxpos: Pos) -> Result<Pos> {
    for y in minpos.1..maxpos.1 {
        let mut x = minpos.0;
        while x <= maxpos.0 {
            for sensor in sensors {
                let xdist = sensor.location.0.abs_diff(x);
                let ydist = sensor.location.1.abs_diff(y);
                let radius = sensor.beacon_radius();
                if xdist + ydist < radius {
                    x = sensor.location.0 + (radius - ydist) as isize;
                }
            }
            if x > maxpos.0 {
                continue;
            }
            let pos = Pos(x, y);
            if sensors
                .iter()
                .all(|s| s.location.distance(pos) > s.beacon_radius())
            {
                return Ok(pos);
            }
            x += 1;
        }
    }
    bail!("No beacon in area");
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let input = io::read_to_string(io::stdin())?;
    let sensor_re = regex::Regex::new(
        r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)",
    )?;
    let sensors: Vec<Sensor> = input
        .lines()
        .map(|sline| {
            let caps = sensor_re
                .captures(sline)
                .ok_or_else(|| eyre!("Bad sensor line: {}", sline))?;
            let sens = Sensor {
                location: Pos(caps[1].parse()?, caps[2].parse()?),
                nearest_beacon: Pos(caps[3].parse()?, caps[4].parse()?),
            };
            Ok(sens)
        })
        .collect::<Result<_>>()?;
    let tune = |pos: Pos| pos.0 * 4000000 + pos.1;
    if sensors[0].location.0 == 2 {
        let ex = beaconsless_cells_on_row(10, &sensors);
        println!("Beaconless cells on row 10 (Example input): {}", ex);
        let beacon = beacon_find(&sensors, Pos(0, 0), Pos(20, 20))?;
        println!(
            "(Example) beacon found: {:?}, answer: {}",
            beacon,
            tune(beacon)
        );
    } else {
        let part1 = beaconsless_cells_on_row(2000000, &sensors);
        println!("Beaconless cells on row 2000000 (Part1): {}", part1);
        let beacon = beacon_find(&sensors, Pos(0, 0), Pos(4000000, 4000000))?;
        println!("Beacon found: {:?}, answer: {}", beacon, tune(beacon));
    }
    Ok(())
}
