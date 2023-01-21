use std::{path::PathBuf, str::FromStr, time::Instant};

use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressIterator, ProgressStyle};
use sudoku::Sudoku;

use crate::sudoku::solve::{dfs, naive_dfs, sorted_dfs};

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
        /// A file with one sudoku per line
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

#[derive(Debug, Default, ValueEnum, Clone)]
enum SudokuSolver {
    /// [EXTREMELY SLOW] A naive recursive DFS, doesn't implement any smart strategies
    NaiveDFS,
    /// [EXTREMELY SLOW] Prunes possibilities when fixing a value
    DFS,
    /// Sorts possibilities by ammount
    #[default]
    SortedDFS,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.mode {
        Mode::Test { solver, file } => {
            #[cfg(debug_assertions)]
            println!("[WARN] Running test in debug mode, it will take very long to complete");

            if let Some(file) = file {
                println!("Reading Sudokus from file: {}", file.display());
                let start = Instant::now();
                let puzzles = String::from_utf8(std::fs::read(&file)?)?
                    .lines()
                    .map(Sudoku::from_str)
                    .collect::<Result<Vec<_>, _>>()?;
                println!("Took {:?} to parse puzzles", start.elapsed());

                println!("Testing {solver:?}:");
                let mut longest = None;
                let start = Instant::now();
                for (ix, puzzle) in puzzles
                    .into_iter()
                    .progress_with_style(
                        ProgressStyle::default_bar()
                            .template("[{pos:>5}/{len}] {per_sec:>9} {wide_bar} {eta_precise}")
                            .expect("valid template"),
                    )
                    .enumerate()
                {
                    let start = Instant::now();
                    let solution = match solver {
                        SudokuSolver::NaiveDFS => naive_dfs(puzzle),
                        SudokuSolver::DFS => dfs(puzzle),
                        SudokuSolver::SortedDFS => sorted_dfs(puzzle),
                    };
                    let end = start.elapsed();
                    if let Some((_, elapsed)) = &longest {
                        if elapsed > &end {
                            longest = Some((ix, end));
                        }
                    } else {
                        longest = Some((ix, end));
                    }

                    if let Err(puzzle) = solution {
                        panic!("Failed to solve {puzzle}");
                    }
                }
                println!("Took {:?}", start.elapsed());
                let (ix, end) = longest.unwrap();
                println!("The longest solve took {end:?}, it was puzzle {}", ix + 1);
            } else {
                let puzzle: Sudoku =
                ".......1.4.........2...........5.4.7..8...3....1.9....3..4..2...5.1........8.6..."
                    .parse()
                    .expect("valid 9x9 Sudoku");

                println!("Testing {solver:?} on:\n{puzzle}");
                let start = Instant::now();
                let solution = match solver {
                    SudokuSolver::NaiveDFS => naive_dfs(puzzle),
                    SudokuSolver::DFS => dfs(puzzle),
                    SudokuSolver::SortedDFS => sorted_dfs(puzzle),
                };
                println!("Took {:?}", start.elapsed());

                if let Ok(puzzle) = solution {
                    println!("Solution:\n{puzzle}")
                } else {
                    println!("No solution found for sudoku")
                }
            }
        }
    }
    Ok(())
}
