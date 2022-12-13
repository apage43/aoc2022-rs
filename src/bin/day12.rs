use std::{
    collections::{HashSet, VecDeque},
    fmt::{Display, Write},
    io::Read,
};

use color_eyre::{eyre::ContextCompat, Result};

struct Heightmap {
    heights: Vec<u8>,
    stride: usize,
    start: (usize, usize),
    end: (usize, usize),
}

impl Display for Heightmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let aschars: Vec<char> = self
            .heights
            .iter()
            .map(|h| (*h + b'a') as char)
            .collect();
        for chunk in aschars.chunks(self.stride) {
            for ch in chunk {
                f.write_char(*ch)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn raw_pos_to_pos(stride: usize, rawp: usize) -> (usize, usize) {
    (rawp as usize % stride, rawp as usize / stride)
}

impl Heightmap {
    fn from_str(input: &str) -> Result<Heightmap> {
        let stride = input.find('\n').context("first newline")?;
        let mut heights = Vec::new();
        let mut idx = 0;
        let mut start = None;
        let mut end = None;
        for ch in input.chars() {
            let height = match ch {
                ch if ('a'..='z').contains(&ch) => Some(ch as u8 - b'a'),
                'S' => {
                    start = Some(raw_pos_to_pos(stride, idx));
                    Some(0)
                }
                'E' => {
                    end = Some(raw_pos_to_pos(stride, idx));
                    Some(25)
                }
                _ => None,
            };
            if let Some(height) = height {
                heights.push(height);
                idx += 1;
            }
        }
        let start = start.context("start marker")?;
        let end = end.context("end marker")?;
        Ok(Heightmap {
            heights,
            stride,
            start,
            end,
        })
    }
    fn height_at(&self, (x, y): (usize, usize)) -> u8 {
        let rawp = x + y * self.stride;
        self.heights[rawp]
    }
    fn up(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        Some((x, y.checked_sub(1)?))
    }
    fn left(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        Some((x.checked_sub(1)?, y))
    }
    fn down(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        (y < (self.heights.len() / self.stride) - 1).then_some((x, y + 1))
    }
    fn right(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        (x < self.stride - 1).then_some((x + 1, y))
    }
    fn adjacents(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let adj = [
            self.up(pos),
            self.right(pos),
            self.down(pos),
            self.left(pos),
        ];
        adj.into_iter().flatten().collect()
    }
    fn climbable(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let curheight = self.height_at(pos);
        self.adjacents(pos)
            .into_iter()
            .flat_map(|apos| match apos {
                pos if self.height_at(pos) <= curheight + 1 => Some(pos),
                _ => None,
            })
            .collect()
    }
    fn climbable_from(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let curheight = self.height_at(pos);
        self.adjacents(pos)
            .into_iter()
            .flat_map(|apos| match apos {
                pos if curheight - 1 <= self.height_at(pos) => Some(pos),
                _ => None,
            })
            .collect()
    }
    fn is_end(&self, pos: (usize, usize)) -> bool {
        pos == self.end
    }
    fn is_zero_elevation(&self, pos: (usize, usize)) -> bool {
        self.height_at(pos) == 0
    }
    fn search(
        &self,
        begin: (usize, usize),
        gen: impl Fn(&Heightmap, (usize, usize)) -> Vec<(usize, usize)>,
        goal: impl Fn(&Heightmap, (usize, usize)) -> bool,
    ) -> Option<usize> {
        struct PathEl {
            pos: (usize, usize),
            step: usize,
        }
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(PathEl {
            pos: begin,
            step: 0,
        });

        while let Some(pel) = queue.pop_front() {
            if visited.contains(&pel.pos) {
                continue;
            }
            if goal(self, pel.pos) {
                return Some(pel.step);
            }
            visited.insert(pel.pos);
            for n in gen(self, pel.pos) {
                if visited.contains(&n) {
                    continue;
                }
                queue.push_back(PathEl {
                    pos: n,
                    step: pel.step + 1,
                })
            }
        }
        None
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let hm = Heightmap::from_str(&buf)?;
    println!("map:\n{}", hm);
    let part1 = hm.search(hm.start, Heightmap::climbable, Heightmap::is_end);
    println!("Part 1 (shortest path S->E): {:?}", part1);
    let part2 = hm.search(
        hm.end,
        Heightmap::climbable_from,
        Heightmap::is_zero_elevation,
    );
    println!("Part 2 (shortest path E->zero height): {:?}", part2);
    Ok(())
}
