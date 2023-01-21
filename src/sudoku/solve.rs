use std::{collections::HashSet, fmt::Display, num::NonZeroU8};

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

type InternalResult = Result<AugmentedSudoku, AugmentedSudoku>;

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
    let mut sudoku: AugmentedSudoku = sudoku.into();

    sudoku.prune_possible();

    Ok(dfs_impl(sudoku)?.into())
}

fn dfs_impl(sudoku: AugmentedSudoku) -> InternalResult {
    let Some((ix, possible)) =
            sudoku
                .data
                .indexed_iter()
                .find(|(_, value)| !value.is_fixed())
        else {
            return Ok(sudoku);
        };

    // println!("{sudoku}");

    let possible = if let AugmentedValue::Possible(possible) = possible {
        possible.clone()
    } else {
        unreachable!()
    };

    for value in possible {
        propagate_ok!(dfs_impl(sudoku.fix_value(ix, value)));
    }

    Err(sudoku)
}

pub fn sorted_dfs(sudoku: super::Sudoku) -> SudokuResult {
    let mut sudoku: AugmentedSudoku = sudoku.into();

    sudoku.prune_possible();

    Ok(sorted_dfs_impl(sudoku)?.into())
}

fn sorted_dfs_impl(sudoku: AugmentedSudoku) -> InternalResult {
    let Some((ix, possible)) = sudoku.data.indexed_iter().min_by_key(|(_, x)| {
        if let AugmentedValue::Possible(x) = x {
            x.len()
        } else {
            usize::MAX
        }
    }) else {
        return Ok(sudoku);
    };

    let possible = if let AugmentedValue::Possible(possible) = possible {
        possible.clone()
    } else {
        return Ok(sudoku)
    };

    for value in possible {
        propagate_ok!(sorted_dfs_impl(sudoku.fix_value(ix, value)));
    }

    Err(sudoku)
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
            .skip(chunk)
            .next()
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
                            (1..=order as u8)
                                .into_iter()
                                .filter_map(NonZeroU8::new)
                                .collect()
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
