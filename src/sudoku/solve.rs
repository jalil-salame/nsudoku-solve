use std::{collections::HashSet, fmt::Display, num::NonZeroU8, ops::ControlFlow};

use ndarray::Array2;

use super::SudokuValue;

macro_rules! propagate_ok {
    ($x:expr) => {
        match $x {
            Ok(value) => return Ok(value),
            Err(err) => err,
        }
    };
}

pub type SudokuResult = Result<super::Sudoku, super::Sudoku>;

type InternalResult = ControlFlow<super::Sudoku, ()>;

pub fn naive_dfs(mut sudoku: super::Sudoku) -> SudokuResult {
    let order = sudoku.order();
    let Some((ix, _)) = sudoku.0.indexed_iter().find(|(_, value)| value.is_none())
    else {
        if sudoku.solved() {
            return Ok(sudoku);
        } else {
            return Err(sudoku);
        }
    };

    for value in 1..=order as u8 {
        *sudoku.0.get_mut(ix).unwrap() = SudokuValue(Some(value.try_into().unwrap()));

        if sudoku.valid() {
            sudoku = propagate_ok!(naive_dfs(sudoku));
        }
    }

    *sudoku.0.get_mut(ix).unwrap() = SudokuValue(None);

    Err(sudoku)
}

pub fn dfs(sudoku: super::Sudoku) -> SudokuResult {
    let orig = sudoku.clone();
    let mut sudoku: AugmentedSudoku = sudoku.into();

    sudoku.prune_possible();

    match dfs_impl(sudoku) {
        ControlFlow::Continue(_) => Err(orig),
        ControlFlow::Break(solved) => Ok(solved),
    }
}

fn dfs_impl(sudoku: AugmentedSudoku) -> InternalResult {
    let Some((ix, possible)) =
            sudoku
                .data
                .indexed_iter()
                .find(|(_, value)| !value.is_fixed())
        else {
            return ControlFlow::Break(sudoku.into());
        };

    // println!("{sudoku}");

    let possible = if let AugmentedValue::Possible(possible) = possible {
        possible.clone()
    } else {
        unreachable!()
    };

    for value in possible {
        dfs_impl(sudoku.fix_value(ix, value))?;
    }

    ControlFlow::Continue(())
}

pub fn sorted_dfs(sudoku: super::Sudoku) -> SudokuResult {
    let mut sudoku: AugmentedSudoku = sudoku.into();

    sudoku.prune_possible();

    match sorted_dfs_impl(&mut sudoku) {
        ControlFlow::Continue(_) => Err(sudoku.into()),
        ControlFlow::Break(solved) => Ok(solved),
    }
}

fn sorted_dfs_impl(sudoku: &mut AugmentedSudoku) -> InternalResult {
    let Some((ix, possible)) = sudoku.data.indexed_iter().min_by_key(|(_, x)| {
        if let AugmentedValue::Possible(x) = x {
            x.len()
        } else {
            usize::MAX
        }
    }) else {
        return ControlFlow::Break(sudoku.clone().into());
    };

    let possible = if let AugmentedValue::Possible(possible) = possible {
        possible.clone()
    } else {
        return ControlFlow::Break(sudoku.clone().into());
    };

    // If it's the only possiblitiy then just fix it
    if possible.len() == 1 {
        let value = possible.into_iter().next().unwrap();
        sudoku.fix_value_inplace(ix, value);
        return sorted_dfs_impl(sudoku);
    }

    // Clone for each possible value otherwise
    for value in possible {
        sorted_dfs_impl(&mut sudoku.fix_value(ix, value))?;
    }

    ControlFlow::Continue(())
}

/// Augmented Sudoku Value
#[derive(Debug, Clone)]
enum AugmentedValue {
    Fixed(NonZeroU8),
    Possible(HashSet<NonZeroU8>),
}

impl AugmentedValue {
    fn is_fixed(&self) -> bool {
        match self {
            AugmentedValue::Fixed(_) => true,
            AugmentedValue::Possible(_) => false,
        }
    }

    fn remove(&mut self, value: NonZeroU8) -> bool {
        match self {
            AugmentedValue::Fixed(_) => false,
            AugmentedValue::Possible(possible) => possible.remove(&value),
        }
    }
}

/// Augmented Sudoku
#[derive(Debug, Clone)]
#[allow(unused)]
struct AugmentedSudoku {
    cell_size: usize,
    order: usize,
    data: Array2<AugmentedValue>,
}

impl AugmentedSudoku {
    fn prune_possible(&mut self) {
        let fixed_values = self
            .data
            .indexed_iter()
            .filter_map(|(ix, value)| {
                if let AugmentedValue::Fixed(value) = value {
                    Some((ix, *value))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (ix, value) in fixed_values {
            self.remove_value(ix, value);
        }
    }

    fn remove_value(&mut self, ix: (usize, usize), value: NonZeroU8) {
        let (row, col) = ix;
        let chunk = (row / self.cell_size) * self.cell_size + col / self.cell_size;

        self.data.row_mut(row).map_inplace(|val| {
            val.remove(value);
        });
        self.data.column_mut(col).map_inplace(|val| {
            val.remove(value);
        });
        self.data
            .exact_chunks_mut((self.cell_size, self.cell_size))
            .into_iter()
            .nth(chunk)
            .unwrap()
            .map_inplace(|val| {
                val.remove(value);
            });
    }

    fn fix_value_inplace(&mut self, ix: (usize, usize), value: NonZeroU8) {
        *self.data.get_mut(ix).unwrap() = value.into();

        self.remove_value(ix, value);
    }

    fn fix_value(&self, ix: (usize, usize), value: NonZeroU8) -> Self {
        let mut new = self.clone();
        new.fix_value_inplace(ix, value);
        new
    }
}

impl From<AugmentedSudoku> for super::Sudoku {
    fn from(value: AugmentedSudoku) -> Self {
        Self(value.data.mapv_into_any(|val| val.into()))
    }
}

impl From<AugmentedValue> for SudokuValue {
    fn from(value: AugmentedValue) -> Self {
        Self(match value {
            AugmentedValue::Fixed(val) => Some(val),
            AugmentedValue::Possible(_) => None,
        })
    }
}

impl From<super::Sudoku> for AugmentedSudoku {
    fn from(value: super::Sudoku) -> Self {
        let order = value.order();
        Self {
            cell_size: value.cell_size(),
            order,
            data: Array2::from_shape_vec(
                (order, order),
                value
                    .0
                    .into_iter()
                    .map(|val| {
                        if let Some(val) = val.0 {
                            val.into()
                        } else {
                            (1..=order as u8).filter_map(NonZeroU8::new).collect()
                        }
                    })
                    .collect(),
            )
            .expect("valid sudoku"),
        }
    }
}

impl From<NonZeroU8> for AugmentedValue {
    fn from(value: NonZeroU8) -> Self {
        Self::Fixed(value)
    }
}

impl TryFrom<u8> for AugmentedValue {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self::Fixed(value.try_into()?))
    }
}

impl FromIterator<NonZeroU8> for AugmentedValue {
    fn from_iter<T: IntoIterator<Item = NonZeroU8>>(iter: T) -> Self {
        Self::Possible(iter.into_iter().collect())
    }
}

impl Display for AugmentedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AugmentedValue::Fixed(val) => write!(f, "{val}"),
            AugmentedValue::Possible(possible) => {
                write!(f, "[")?;
                for ele in possible {
                    write!(f, " {ele}")?;
                }
                write!(f, " ]")
            }
        }
    }
}

impl Display for AugmentedSudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.order * 2 + 3;
        let horizontal_line = format!(
            "+{}",
            format!("{}+", "-".repeat(self.cell_size * (width + 1) + 1)).repeat(self.cell_size)
        );

        for blocks in self.data.exact_chunks((self.cell_size, self.order)) {
            writeln!(f, "{horizontal_line}")?;
            for row in blocks.rows() {
                write!(f, "|")?;
                for cell in row.exact_chunks((self.cell_size,)) {
                    for ele in cell {
                        write!(f, " {:^width$}", format!("{ele}"))?;
                    }
                    write!(f, " |")?;
                }
                writeln!(f)?;
            }
        }

        write!(f, "{horizontal_line}")
    }
}

#[cfg(test)]
mod test {
    use crate::sudoku::Sudoku;

    use super::dfs;

    #[test]
    fn puzzle54_solvable() {
        let sudoku: Sudoku =
            ".......16.4...5.......2.......6..43.2...1....3.....5.......37..1..8.......2......"
                .parse()
                .expect("Successful parse");

        assert!(dfs(sudoku).is_ok())
    }

    // extern crate test;
    // use test::Bencher;
    //
    // #[bench]
    // fn sudoku_54(bench: &mut Bencher) {
    //     let sudoku: Sudoku =
    //         ".......16.4...5.......2.......6..43.2...1....3.....5.......37..1..8.......2......"
    //             .parse()
    //             .expect("Successful parse");

    //     bench.iter(|| dfs(sudoku.clone()).is_ok())
    // }
}
