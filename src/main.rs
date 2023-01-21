use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;
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
                let num_puzzles = puzzles.len();
                let start = Instant::now();
                let longest = puzzles
                    .into_par_iter()
                    .progress_with_style(
                        ProgressStyle::default_bar()
                            .template("[{pos:>5}/{len}] {per_sec:>10} {wide_bar} {eta_precise}/{duration_precise}")
                            .expect("valid template"),
                    )
                    .fold(
                        || Duration::from_secs(0),
                        |longest, puzzle| {
                            let start = Instant::now();
                            let solution = match solver {
                                SudokuSolver::DFS => dfs(puzzle),
                                SudokuSolver::NaiveDFS => naive_dfs(puzzle),
                                SudokuSolver::SortedDFS => sorted_dfs(puzzle),
                            };
                            let end = start.elapsed();

                            if let Err(puzzle) = solution {
                                panic!("Failed to solve {puzzle}");
                            }

                            longest.max(end)
                        },
                    )
                    .reduce(|| Duration::from_secs(0), Duration::max);
                let end = start.elapsed();
                let cpu_time = end * num_cpus::get() as u32;
                let per_puzzle = cpu_time / num_puzzles as u32;
                println!("Took {end:?} [{per_puzzle:?}/sudoku]");
                println!("The longest solve took {longest:?}");
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
