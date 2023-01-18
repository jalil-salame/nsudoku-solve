use std::error::Error;

use sudoku::Sudoku;

use crate::sudoku::solve::naive_dfs;

mod sudoku;

fn main() -> Result<(), Box<dyn Error>> {
    let mut puzzle: Sudoku =
        ".......1.4.........2...........5.4.7..8...3....1.9....3..4..2...5.1........8.6..."
            .parse()?;
    println!("{puzzle}");
    if naive_dfs(&mut puzzle) {
        println!("Solved:\n{puzzle}")
    } else {
        println!("No solution found for sudoku")
    }
    Ok(())
}
