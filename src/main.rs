use std::{
    num::ParseIntError,
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
        /// The string represeentation of a Sudoku
        #[arg(short, long)]
        sudoku: Option<Sudoku>,
    },
}

#[derive(Debug, Default, ValueEnum, Clone)]
enum SudokuSolver {
    /// [EXTREMELY SLOW] A naive recursive DFS, doesn't implement any smart strategies
    NaiveDfs,
    /// [EXTREMELY SLOW] Prunes possibilities when fixing a value
    Dfs,
    /// Sorts possibilities by ammount
    #[default]
    SortedDfs,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.mode {
        Mode::Test {
            solver,
            file,
            sudoku,
        } => {
            #[cfg(debug_assertions)]
            println!("[WARN] Running test in debug mode, it will take very long to complete");

            if file.is_some() && sudoku.is_some() {
                println!("[WARN] Both a file and a sample sudoku provided, ignoring sudoku");
            }

            if let Some(file) = file {
                println!("Reading Sudokus from file: {}", file.display());
                let start = Instant::now();
                let puzzles = String::from_utf8(std::fs::read(&file)?)?
                    .lines()
                    .enumerate()
                    .map(|(ix, s)| Ok((ix, Sudoku::from_str(s)?)))
                    .collect::<Result<Vec<(usize, Sudoku)>, ParseIntError>>()?;
                println!("Took {:?} to parse puzzles", start.elapsed());

                println!("Testing {solver:?}:");
                let num_puzzles = puzzles.len();
                let start = Instant::now();
                let (ix, longest) = puzzles
                    .into_par_iter()
                    .progress_with_style(
                        ProgressStyle::default_bar()
                            .template("[{pos:>5}/{len}] {per_sec:>10} {wide_bar} {eta_precise}/{duration_precise}")
                            .expect("valid template"),
                    )
                    .fold(
                        || (0, Duration::from_secs(0)),
                        |(ix, longest), (iy, puzzle)| {
                            let start = Instant::now();
                            let solution = match solver {
                                SudokuSolver::Dfs => dfs(puzzle),
                                SudokuSolver::NaiveDfs => naive_dfs(puzzle),
                                SudokuSolver::SortedDfs => sorted_dfs(puzzle),
                            };
                            let end = start.elapsed();

                            if let Err(puzzle) = solution {
                                panic!("Failed to solve {puzzle}");
                            }

                            if end > longest {
                                (iy, end)
                            } else {
                                (ix, longest)
                            }
                        },
                    )
                    .reduce(|| (0, Duration::from_secs(0)), |(ix, tx), (iy, ty)| if ty > tx { (iy, ty)} else {(ix, tx)});
                let end = start.elapsed();
                let cpu_time = end * num_cpus::get() as u32;
                let per_puzzle = cpu_time / num_puzzles as u32;
                println!("Took {end:?} [{per_puzzle:?}/sudoku]");
                println!(
                    "The longest solve was puzzle #{} and took {longest:?}",
                    ix + 1
                );
            } else {
                let puzzle: Sudoku = if let Some(s) = sudoku {
                    s
                } else {
                    ".......1.4.........2...........5.4.7..8...3....1.9....3..4..2...5.1........8.6..."
                    .parse()
                    .expect("valid 9x9 Sudoku")
                };

                println!("Testing {solver:?} on:\n{puzzle}");
                let start = Instant::now();
                let solution = match solver {
                    SudokuSolver::NaiveDfs => naive_dfs(puzzle),
                    SudokuSolver::Dfs => dfs(puzzle),
                    SudokuSolver::SortedDfs => sorted_dfs(puzzle),
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
