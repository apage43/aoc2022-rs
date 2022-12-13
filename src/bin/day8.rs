use color_eyre::{eyre::ContextCompat, Result};

struct TreeGrid {
    heights: Vec<u8>,
    stride: usize,
}

impl TreeGrid {
    fn width(&self) -> usize {
        self.stride
    }
    fn height(&self) -> usize {
        self.heights.len() / self.stride
    }
    fn height_at(&self, x: usize, y: usize) -> Option<u8> {
        self.heights.get(y * self.stride + x).copied()
    }
    fn print_visibility(&self, vis: &[Visibility]) {
        let digstr: Vec<char> = vis
            .iter()
            .map(|x| match x {
                Visibility::Occluded => '#',
                Visibility::Visible => 'T',
            })
            .collect();
        let lines: Vec<String> = digstr.chunks(self.stride).map(String::from_iter).collect();
        println!("{}", lines.join("\n"));
    }
    fn scenic_score(&self, x: usize, y: usize) -> u32 {
        let mut paths: Vec<Vec<(usize, usize)>> = Vec::new();
        let our_height = self.height_at(x, y).unwrap();

        // up
        paths.push((0..y).rev().map(|y| (x, y)).collect());
        // left
        paths.push((0..x).rev().map(|x| (x, y)).collect());
        // right
        paths.push((x..self.width()).map(|x| (x, y)).skip(1).collect());
        // down
        paths.push((y..self.height()).map(|y| (x, y)).skip(1).collect());

        let mut dists = Vec::new();
        for path in paths {
            let mut dist = 0;
            for (x, y) in path {
                let th = self.height_at(x, y).unwrap();
                dist += 1;
                if th >= our_height {
                    break;
                }
            }
            dists.push(dist);
        }
        dists.iter().product()
    }
    fn edge_visibility(&self) -> Vec<Visibility> {
        let mut visibility = vec![Visibility::Occluded; self.heights.len()];
        // left to right passes
        for y in 0..self.height() {
            let mut max_height = None;
            for x in 0..self.width() {
                let height = self.height_at(x, y).unwrap();
                if max_height.is_none() || height > max_height.unwrap() {
                    visibility[self.stride * y + x] = Visibility::Visible;
                    max_height = Some(height);
                }
            }
        }
        // right to left passes
        for y in 0..self.height() {
            let mut max_height = None;
            for x in (0..self.width()).rev() {
                let height = self.height_at(x, y).unwrap();
                if max_height.is_none() || height > max_height.unwrap() {
                    visibility[self.stride * y + x] = Visibility::Visible;
                    max_height = Some(height);
                }
            }
        }
        // top to bottom passes
        for x in 0..self.width() {
            let mut max_height = None;
            for y in 0..self.height() {
                let height = self.height_at(x, y).unwrap();
                if max_height.is_none() || height > max_height.unwrap() {
                    visibility[self.stride * y + x] = Visibility::Visible;
                    max_height = Some(height);
                }
            }
        }
        // bottom to top passes
        for x in 0..self.width() {
            let mut max_height = None;
            for y in (0..self.height()).rev() {
                let height = self.height_at(x, y).unwrap();
                if max_height.is_none() || height > max_height.unwrap() {
                    visibility[self.stride * y + x] = Visibility::Visible;
                    max_height = Some(height);
                }
            }
        }
        visibility
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Visibility {
    Occluded,
    Visible,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut heights: Vec<u8> = Vec::new();
    let mut grid_stride = None;
    for line in std::io::stdin().lines() {
        let line = line?;
        let digits: Option<Vec<_>> = line
            .chars()
            .map(|c| c.to_digit(10).map(|d| d as u8))
            .collect();
        let mut digits = digits.context("digit parsing")?;
        grid_stride = Some(digits.len());
        heights.append(&mut digits);
    }
    let treegrid = TreeGrid {
        heights,
        stride: grid_stride.context("stride")?,
    };
    let edgevis = treegrid.edge_visibility();
    treegrid.print_visibility(&edgevis);
    let total_edge_visible = edgevis
        .iter()
        .filter(|x| **x == Visibility::Visible)
        .count();
    println!("Trees visible from edges: {}", total_edge_visible);
    let mut best_loc = None;
    let mut best_score = None;
    for x in 0..treegrid.width() {
        for y in 0..treegrid.height() {
            let score = treegrid.scenic_score(x, y);
            if best_score.is_none() || score > best_score.unwrap() {
                best_score = Some(score);
                best_loc = Some((x, y));
            }
        }
    }
    println!(
        "Best treehouse score {:?} at {:?}",
        best_score.unwrap(),
        best_loc.unwrap()
    );
    Ok(())
}
