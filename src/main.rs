use std::{collections::BTreeSet, fmt::Display};

use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use rand::seq::IteratorRandom;
use rand::{random_bool, Rng};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize)]
struct Tango {
    grid: Grid<TangoTile>,
    restrictions: Vec<TangoRestriction>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
enum TangoTile {
    #[default]
    Empty,
    Red,
    Blue,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
enum TangoRestriction {
    Same((usize, usize), (usize, usize)),
    Different((usize, usize), (usize, usize)),
}

#[derive(Debug, Clone, Serialize)]
struct Grid<T> {
    width: usize,
    height: usize,
    tiles: Vec<T>,
}

impl Display for TangoTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TangoTile::Empty => write!(f, " "),
            TangoTile::Red => write!(f, "R"),
            TangoTile::Blue => write!(f, "B"),
        }
    }
}

impl Display for Tango {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tango Puzzle: {}x{}", self.grid.width, self.grid.height)?;
        writeln!(f, "Restrictions:")?;
        for restriction in &self.restrictions {
            match restriction {
                TangoRestriction::Same((x1, y1), (x2, y2)) => {
                    writeln!(f, "Same: ({}, {}) <-> ({}, {})", x1, y1, x2, y2)?;
                }
                TangoRestriction::Different((x1, y1), (x2, y2)) => {
                    writeln!(
                        f,
                        "Different: ({}, {}) <-> ({}, {})",
                        x1, y1, x2, y2
                    )?;
                }
            }
        }
        let h = "─"; // U+2500
        let v = "│"; // U+2502

        // Corners
        let tl = "┌"; // U+250C
        let tr = "┐"; // U+2510
        let bl = "└"; // U+2514
        let br = "┘"; // U+2518
        let t_down = "┬"; // U+252C
        let t_up = "┴"; // U+2534
        let t_right = "├"; // U+251C
        let t_left = "┤"; // U+2524
        let cross = "┼"; // U+253C
        writeln!(f, "{}{}{}", tl, h.repeat(self.grid.width * 2 - 1), tr)?;
        for y in 0..self.grid.height {
            write!(f, "{}", v)?;
            for x in 0..self.grid.width {
                if let Some(tile) = self.grid.get(x, y) {
                    write!(f, "{}", tile)?;
                } else {
                    write!(f, "",)?; // Empty space for out-of-bounds
                }
                // Add = or x between tiles with Same or Different restrictions
                if x < self.grid.width - 1 {
                    match self.get_restriction((x, y), (x + 1, y)) {
                        Some(TangoRestriction::Same(_, _)) => write!(f, "=")?,
                        Some(TangoRestriction::Different(_, _)) => {
                            write!(f, "x")?
                        }
                        None => write!(f, "{}", v)?,
                    }
                } else {
                    write!(f, "{}", v)?
                }
            }
            writeln!(f)?;
            if y < self.grid.height - 1 {
                write!(f, "{}", v)?;
                for x in 0..self.grid.width {
                    // Add = or x between tiles with Same or Different restrictions
                    match self.get_restriction((x, y), (x, y + 1)) {
                        Some(TangoRestriction::Same(_, _)) => write!(f, "=")?,
                        Some(TangoRestriction::Different(_, _)) => {
                            write!(f, "x")?
                        }
                        None => write!(f, "{}", h)?,
                    }
                    if x < self.grid.width - 1 {
                        write!(f, "{}", cross)?; // Space between tiles
                    }
                }
                writeln!(f, "{}", v)?;
            }
        }
        writeln!(f, "{}{}{}", bl, h.repeat(self.grid.width * 2 - 1), br)?;
        Ok(())
    }
}

impl<T: Default + Clone> Grid<T> {
    fn new(width: usize, height: usize) -> Self {
        let tiles = vec![T::default(); width * height];
        Grid {
            width,
            height,
            tiles,
        }
    }
    fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            Some(&self.tiles[y * self.width + x])
        } else {
            None
        }
    }
    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width && y < self.height {
            Some(self.tiles.get_mut(y * self.width + x).unwrap())
        } else {
            None
        }
    }
}

impl Tango {
    fn new(
        width: usize,
        height: usize,
        restrictions: Vec<TangoRestriction>,
    ) -> Result<Self, &'static str> {
        if width == 0 || height == 0 {
            return Err("Width and height must be greater than zero.");
        }
        if width % 2 != 0 || height % 2 != 0 {
            return Err("Width and height must be even numbers.");
        }
        Ok(Tango {
            grid: Grid::new(width, height),
            restrictions,
        })
    }

    fn set_tile(&mut self, x: usize, y: usize, tile: TangoTile) -> bool {
        let mut prev_tile = TangoTile::Empty;
        if let Some(existing_tile) = self.grid.get_mut(x, y) {
            prev_tile = *existing_tile;
            *existing_tile = tile;
        }
        if self.is_valid_row(y)
            && self.is_valid_column(x)
            && self.check_restrictions()
        {
            true
        } else {
            if let Some(existing_tile) = self.grid.get_mut(x, y) {
                *existing_tile = prev_tile;
            }
            false
        }
    }

    fn get_tile(&self, x: usize, y: usize) -> Option<TangoTile> {
        self.grid.get(x, y).cloned()
    }

    fn is_valid_row(&self, y: usize) -> bool {
        if y >= self.grid.height {
            return false;
        }
        let mut last_tile = TangoTile::Empty;
        let mut consecuteive_same_count = 0;
        let mut red_count = 0;
        let mut blue_count = 0;
        for x in 0..self.grid.width {
            match self.get_tile(x, y) {
                Some(TangoTile::Red) => red_count += 1,
                Some(TangoTile::Blue) => blue_count += 1,
                _ => {}
            }
            if let Some(tile) = self.get_tile(x, y) {
                if tile != TangoTile::Empty && tile == last_tile {
                    consecuteive_same_count += 1;
                    if consecuteive_same_count > 1 {
                        return false; // More than one consecutive same tile
                    }
                } else {
                    consecuteive_same_count = 0; // Reset count for different tile
                }
                last_tile = tile;
            }
        }
        red_count <= self.grid.width / 2 && blue_count <= self.grid.width / 2
    }
    fn is_valid_column(&self, x: usize) -> bool {
        if x >= self.grid.width {
            return false;
        }
        let mut last_tile = TangoTile::Empty;
        let mut consecuteive_same_count = 0;
        let mut red_count = 0;
        let mut blue_count = 0;
        for y in 0..self.grid.height {
            match self.get_tile(x, y) {
                Some(TangoTile::Red) => red_count += 1,
                Some(TangoTile::Blue) => blue_count += 1,
                _ => {}
            }
            if let Some(tile) = self.get_tile(x, y) {
                if tile != TangoTile::Empty && tile == last_tile {
                    consecuteive_same_count += 1;
                    if consecuteive_same_count > 1 {
                        return false; // More than one consecutive same tile
                    }
                } else {
                    consecuteive_same_count = 0; // Reset count for different tile
                }
                last_tile = tile;
            }
        }
        red_count <= self.grid.height / 2 && blue_count <= self.grid.height / 2
    }
    fn check_restrictions(&self) -> bool {
        for restriction in &self.restrictions {
            match restriction {
                TangoRestriction::Same((x1, y1), (x2, y2)) => {
                    if let (Some(tile1), Some(tile2)) =
                        (self.get_tile(*x1, *y1), self.get_tile(*x2, *y2))
                    {
                        if tile1 == TangoTile::Empty
                            || tile2 == TangoTile::Empty
                        {
                            continue; // Empty tiles can be ignored
                        }
                        if tile1 != tile2 {
                            return false;
                        }
                    }
                }
                TangoRestriction::Different((x1, y1), (x2, y2)) => {
                    if let (Some(tile1), Some(tile2)) =
                        (self.get_tile(*x1, *y1), self.get_tile(*x2, *y2))
                    {
                        if tile1 == TangoTile::Empty
                            || tile2 == TangoTile::Empty
                        {
                            continue; // Empty tiles can be ignored
                        }
                        if tile1 == tile2 {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    fn get_restriction(
        &self,
        a: (usize, usize),
        b: (usize, usize),
    ) -> Option<&TangoRestriction> {
        self.restrictions.iter().find(|r| match r {
            TangoRestriction::Same((x1, y1), (x2, y2)) => {
                (x1 == &a.0 && y1 == &a.1 && x2 == &b.0 && y2 == &b.1)
                    || (x2 == &a.0 && y2 == &a.1 && x1 == &b.0 && y1 == &b.1)
            }
            TangoRestriction::Different((x1, y1), (x2, y2)) => {
                (x1 == &a.0 && y1 == &a.1 && x2 == &b.0 && y2 == &b.1)
                    || (x2 == &a.0 && y2 == &a.1 && x1 == &b.0 && y1 == &b.1)
            }
        })
    }
}

struct RecursiveTangoSolver {
    tango: Tango,
}

impl RecursiveTangoSolver {
    fn new(tango: Tango) -> Self {
        RecursiveTangoSolver { tango }
    }

    fn solve(&mut self, counter_mode: bool) -> usize {
        self.solve_recursive(counter_mode, 0)
    }

    fn solve_recursive(&mut self, counter_mode: bool, mut acc: usize) -> usize {
        for y in 0..self.tango.grid.height {
            for x in 0..self.tango.grid.width {
                if let Some(tile) = self.tango.get_tile(x, y) {
                    if tile == TangoTile::Empty {
                        for &new_tile in &[TangoTile::Red, TangoTile::Blue] {
                            if self.tango.set_tile(x, y, new_tile) {
                                let result =
                                    self.solve_recursive(counter_mode, acc);
                                if result > 0 {
                                    if !counter_mode {
                                        return result; // Return the count
                                    } else {
                                        acc = result;
                                    }
                                }
                            }
                            // Reset the tile if it doesn't lead to a solution
                            self.tango.set_tile(x, y, TangoTile::Empty);
                        }
                        return acc; // No valid tile found
                    }
                }
            }
        }
        // println!("Reached a solution state\n{}", self.tango);
        acc + 1
    }
}

struct TangoGenerator {
    width: usize,
    height: usize,
    neighbor_pairs: Vec<((usize, usize), (usize, usize))>,
}

impl TangoGenerator {
    fn new(width: usize, height: usize) -> Self {
        TangoGenerator {
            width,
            height,
            neighbor_pairs: itertools::iproduct!(0..width - 1, 0..height)
                .zip(itertools::iproduct!(1..width, 0..height))
                .chain(
                    itertools::iproduct!(0..width, 0..height - 1)
                        .zip(itertools::iproduct!(0..width, 1..height)),
                )
                .collect(),
        }
    }

    fn generate(&self) -> Tango {
        // Placeholder for actual generation logic
        let mut tango = Tango::new(self.width, self.height, vec![])
            .expect("Failed to create Tango");

        let mut rng = &mut rand::rng();

        let to_take = rng.random_range(0..=self.neighbor_pairs.len());
        for (a, b) in self
            .neighbor_pairs
            .iter()
            .cloned()
            .choose_multiple(&mut rng, to_take)
        {
            if random_bool(0.5) {
                tango.restrictions.push(TangoRestriction::Same(a, b));
            } else {
                tango.restrictions.push(TangoRestriction::Different(a, b));
            }
        }
        // Randomly fill the grid with tiles
        for y in 0..tango.grid.height {
            for x in 0..tango.grid.width {
                if random_bool(0.1) {
                    if random_bool(0.5) {
                        tango.set_tile(x, y, TangoTile::Red);
                    } else {
                        tango.set_tile(x, y, TangoTile::Blue);
                    }
                }
            }
        }

        tango
    }

    fn generate_one_solution_tango() -> Tango {
        let tango_generator = TangoGenerator::new(6, 6);

        let mut try_count = 0;
        loop {
            try_count += 1;
            println!("Attempt #{:06}", try_count);
            let tango = tango_generator.generate();

            let mut solver = RecursiveTangoSolver::new(tango.clone());
            let solution_count = solver.solve(true);
            if solution_count == 1 {
                solver.solve(false);
                println!(
                    "Solution found!\nTotal solutions: {}",
                    solution_count
                );
                println!("Tango:\n{}", &tango);
                dbg!("Tango:\n{}", &tango);
                return tango;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route(
            "/api/tango-board",
            get(|| async {
                axum::Json(serde_json::json!(
                    TangoGenerator::generate_one_solution_tango()
                ))
            }),
        )
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
