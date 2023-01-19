
use std::time::Instant;

use clap::{Parser, Subcommand, ValueEnum};
use sudoku::Sudoku;

use crate::sudoku::solve::{dfs, naive_dfs};

mod sudoku;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Debug, Subcommand)]
enum Mode {
    /// Test the sudoku solver on a default 9x9 puzzle
    Test {
        /// The solver strategy to use
        #[arg(value_enum, default_value_t)]
        solver: SudokuSolver,
    },
}

#[derive(Debug, Default, ValueEnum, Clone)]
enum SudokuSolver {
    /// A naive recursive DFS, doesn't implement any smart strategies
    NaiveDFS,
    #[default]
    DFS,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.mode {
        Mode::Test { solver } => {
            #[cfg(debug_assertions)]
            println!("[WARN] Running test in debug mode, it will take very long to complete");

            let puzzle: Sudoku =
                ".......1.4.........2...........5.4.7..8...3....1.9....3..4..2...5.1........8.6..."
                    .parse()
                    .expect("valid 9x9 Sudoku");

            println!("Testing {solver:?} on:\n{puzzle}");

            let start = Instant::now();
            let solution = match solver {
                SudokuSolver::NaiveDFS => naive_dfs(puzzle),
                SudokuSolver::DFS => dfs(puzzle),
            };

            println!("Took {:?}", start.elapsed());

            if let Ok(puzzle) = solution {
                println!("Solution:\n{puzzle}")
            } else {
                println!("No solution found for sudoku")
            }
        }
    }
    Ok(())
}
