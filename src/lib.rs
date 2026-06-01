//! Aetherfall is an original falling-sand sandbox core written in Rust.
//!
//! The crate intentionally implements its own compact cellular-automata model
//! rather than copying code from any upstream game or engine. It is suitable as
//! a seed for a larger survival/sandbox game: a deterministic world grid,
//! simple materials, gravity, liquid flow, fire spread, and ASCII rendering.

use std::fmt;

/// A simulation material occupying one world cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Material {
    Air,
    Stone,
    Sand,
    Water,
    Wood,
    Fire,
}

impl Material {
    /// Returns true if another moving material may displace this cell.
    pub const fn is_passable(self) -> bool {
        matches!(self, Self::Air | Self::Water | Self::Fire)
    }

    /// Returns true if this material can catch fire.
    pub const fn is_flammable(self) -> bool {
        matches!(self, Self::Wood)
    }

    /// Stable single-character rendering for terminal prototypes and tests.
    pub const fn glyph(self) -> char {
        match self {
            Self::Air => ' ',
            Self::Stone => '#',
            Self::Sand => '.',
            Self::Water => '~',
            Self::Wood => 'W',
            Self::Fire => '*',
        }
    }
}

/// Error returned when constructing a world with invalid dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    EmptyDimensions { width: usize, height: usize },
    TooLarge { width: usize, height: usize },
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDimensions { width, height } => {
                write!(f, "world dimensions must be non-zero, got {width}x{height}")
            }
            Self::TooLarge { width, height } => {
                write!(
                    f,
                    "world dimensions overflow address space: {width}x{height}"
                )
            }
        }
    }
}

impl std::error::Error for WorldError {}

/// Deterministic two-dimensional sandbox world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    width: usize,
    height: usize,
    cells: Vec<Material>,
    tick: u64,
}

impl World {
    /// Creates a new empty world.
    pub fn new(width: usize, height: usize) -> Result<Self, WorldError> {
        if width == 0 || height == 0 {
            return Err(WorldError::EmptyDimensions { width, height });
        }

        let cell_count = width
            .checked_mul(height)
            .ok_or(WorldError::TooLarge { width, height })?;

        Ok(Self {
            width,
            height,
            cells: vec![Material::Air; cell_count],
            tick: 0,
        })
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Places a material if the coordinate is inside the world.
    pub fn set(&mut self, x: usize, y: usize, material: Material) -> bool {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = material;
            true
        } else {
            false
        }
    }

    /// Reads a material if the coordinate is inside the world.
    pub fn get(&self, x: usize, y: usize) -> Option<Material> {
        self.index(x, y).map(|index| self.cells[index])
    }

    /// Advances the simulation by one deterministic tick.
    pub fn step(&mut self) {
        let mut visited = vec![false; self.cells.len()];
        let left_first = self.tick % 2 == 0;

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let index = self.index_unchecked(x, y);
                if visited[index] {
                    continue;
                }

                match self.cells[index] {
                    Material::Sand => self.update_sand(x, y, &mut visited, left_first),
                    Material::Water => self.update_water(x, y, &mut visited, left_first),
                    Material::Fire => self.update_fire(x, y, &mut visited),
                    _ => visited[index] = true,
                }
            }
        }

        self.tick += 1;
    }

    /// Renders the world as newline-separated ASCII rows.
    pub fn render_ascii(&self) -> String {
        let mut output = String::with_capacity((self.width + 1) * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                output.push(self.cells[self.index_unchecked(x, y)].glyph());
            }
            if y + 1 < self.height {
                output.push('\n');
            }
        }
        output
    }

    fn update_sand(&mut self, x: usize, y: usize, visited: &mut [bool], left_first: bool) {
        let destinations = self.gravity_destinations(x, y, left_first);
        for (next_x, next_y) in destinations {
            if self.can_sand_enter(next_x, next_y) {
                self.swap_and_mark(x, y, next_x, next_y, visited);
                return;
            }
        }
        visited[self.index_unchecked(x, y)] = true;
    }

    fn update_water(&mut self, x: usize, y: usize, visited: &mut [bool], left_first: bool) {
        let horizontal = self.horizontal_destinations(x, y, left_first);
        let destinations = [
            self.below(x, y),
            horizontal[0],
            horizontal[1],
            self.diagonal_below(x, y, left_first),
            self.diagonal_below(x, y, !left_first),
        ];

        for destination in destinations.into_iter().flatten() {
            if self.get(destination.0, destination.1) == Some(Material::Air) {
                self.swap_and_mark(x, y, destination.0, destination.1, visited);
                return;
            }
        }
        visited[self.index_unchecked(x, y)] = true;
    }

    fn update_fire(&mut self, x: usize, y: usize, visited: &mut [bool]) {
        let index = self.index_unchecked(x, y);
        let mut spread = false;

        for (neighbor_x, neighbor_y) in self.neighbors4(x, y) {
            let neighbor_index = self.index_unchecked(neighbor_x, neighbor_y);
            if self.cells[neighbor_index].is_flammable() {
                self.cells[neighbor_index] = Material::Fire;
                visited[neighbor_index] = true;
                spread = true;
            }
        }

        self.cells[index] = if spread || (self.tick + x as u64 + y as u64) % 3 == 0 {
            Material::Air
        } else {
            Material::Fire
        };
        visited[index] = true;
    }

    fn can_sand_enter(&self, x: usize, y: usize) -> bool {
        self.get(x, y)
            .is_some_and(|material| material.is_passable() && material != Material::Fire)
    }

    fn gravity_destinations(&self, x: usize, y: usize, left_first: bool) -> Vec<(usize, usize)> {
        let mut destinations = Vec::with_capacity(3);
        if let Some(below) = self.below(x, y) {
            destinations.push(below);
        }
        if let Some(diagonal) = self.diagonal_below(x, y, left_first) {
            destinations.push(diagonal);
        }
        if let Some(diagonal) = self.diagonal_below(x, y, !left_first) {
            destinations.push(diagonal);
        }
        destinations
    }

    fn below(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        (y + 1 < self.height).then_some((x, y + 1))
    }

    fn diagonal_below(&self, x: usize, y: usize, left: bool) -> Option<(usize, usize)> {
        let next_x = if left { x.checked_sub(1)? } else { x + 1 };
        (next_x < self.width && y + 1 < self.height).then_some((next_x, y + 1))
    }

    fn horizontal_destinations(
        &self,
        x: usize,
        y: usize,
        left_first: bool,
    ) -> [Option<(usize, usize)>; 2] {
        let left = x.checked_sub(1).map(|next_x| (next_x, y));
        let right = (x + 1 < self.width).then_some((x + 1, y));
        if left_first {
            [left, right]
        } else {
            [right, left]
        }
    }

    fn neighbors4(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::with_capacity(4);
        if let Some(left) = x.checked_sub(1) {
            neighbors.push((left, y));
        }
        if x + 1 < self.width {
            neighbors.push((x + 1, y));
        }
        if let Some(up) = y.checked_sub(1) {
            neighbors.push((x, up));
        }
        if y + 1 < self.height {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    fn swap_and_mark(
        &mut self,
        x: usize,
        y: usize,
        next_x: usize,
        next_y: usize,
        visited: &mut [bool],
    ) {
        let current = self.index_unchecked(x, y);
        let next = self.index_unchecked(next_x, next_y);
        self.cells.swap(current, next);
        visited[current] = true;
        visited[next] = true;
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.width && y < self.height).then(|| self.index_unchecked(x, y))
    }

    fn index_unchecked(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

#[cfg(test)]
mod tests {
    use super::{Material, World, WorldError};

    #[test]
    fn rejects_empty_worlds() {
        assert_eq!(
            World::new(0, 4).unwrap_err(),
            WorldError::EmptyDimensions {
                width: 0,
                height: 4
            }
        );
    }

    #[test]
    fn sand_falls_into_air() {
        let mut world = World::new(3, 3).unwrap();
        world.set(1, 0, Material::Sand);

        world.step();

        assert_eq!(world.get(1, 1), Some(Material::Sand));
        assert_eq!(world.get(1, 0), Some(Material::Air));
    }

    #[test]
    fn sand_displaces_water() {
        let mut world = World::new(3, 3).unwrap();
        world.set(1, 0, Material::Sand);
        world.set(1, 1, Material::Water);
        world.set(0, 1, Material::Stone);
        world.set(2, 1, Material::Stone);
        world.set(0, 2, Material::Stone);
        world.set(1, 2, Material::Stone);
        world.set(2, 2, Material::Stone);

        world.step();

        assert_eq!(world.get(1, 1), Some(Material::Sand));
        assert_eq!(world.get(1, 0), Some(Material::Water));
    }

    #[test]
    fn water_spreads_sideways_when_blocked() {
        let mut world = World::new(3, 3).unwrap();
        world.set(1, 1, Material::Water);
        world.set(1, 2, Material::Stone);

        world.step();

        assert_eq!(world.get(0, 1), Some(Material::Water));
    }

    #[test]
    fn fire_spreads_to_wood() {
        let mut world = World::new(3, 3).unwrap();
        world.set(1, 1, Material::Fire);
        world.set(2, 1, Material::Wood);

        world.step();

        assert_eq!(world.get(2, 1), Some(Material::Fire));
    }

    #[test]
    fn renders_ascii_snapshot() {
        let mut world = World::new(2, 2).unwrap();
        world.set(0, 0, Material::Stone);
        world.set(1, 1, Material::Water);

        assert_eq!(world.render_ascii(), "# \n ~");
    }
}
