use std::num::NonZeroU8;

use super::SudokuValue;

pub fn naive_dfs(sudoku: &mut super::Sudoku) -> bool {
    let order = sudoku.order();
    let Some((ix, _)) = sudoku.0.indexed_iter().find(|(_, value)| value.is_none()) else { return sudoku.solved(); };

    for value in 1..order as u8 + 1 {
        *sudoku.0.get_mut(ix).expect("valid index") =
            SudokuValue(Some(NonZeroU8::new(value).expect("NonZeroU8")));

        // println!("{sudoku}");

        if sudoku.valid() && naive_dfs(sudoku) {
            return true;
        }
    }

    *sudoku.0.get_mut(ix).expect("valid index") = SudokuValue(None);

    false
}
