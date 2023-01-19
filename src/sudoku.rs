use std::{fmt::Display, num::NonZeroU8, ops::Deref, str::FromStr};

use ndarray::{Array2, ArrayView, Dimension};

pub mod solve;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SudokuValue(Option<NonZeroU8>);

#[derive(Debug, Clone)]
pub struct Sudoku(Array2<SudokuValue>);

impl Sudoku {
    /// Create a new empty 9x9 Sudoku
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::from_order(9)
    }

    /// Create a new empty Sudoku of size order * order
    #[allow(dead_code)]
    fn from_order(order: usize) -> Self {
        Self::from_order_vec(order, vec![SudokuValue::default(); order * order])
    }

    /// Create a new Sudoku with size order * order and select values
    fn from_order_vec(order: usize, values: Vec<SudokuValue>) -> Self {
        assert!(
            (2..16).map(|x| x * x).any(|x| x == order),
            "The maximum supported Sudoku size is 225"
        );
        assert_eq!(
            values.len(),
            order * order,
            "Values is not of size order * order"
        );

        Self(Array2::from_shape_vec((order, order), values).expect("valid values size"))
    }

    pub fn solved(&self) -> bool {
        self.filled() && self.valid()
    }

    fn filled(&self) -> bool {
        self.0.iter().all(|value| value.is_some())
    }

    fn valid(&self) -> bool {
        let cell_size = self.cell_size();
        Self::valid_set(self.0.rows())
            && Self::valid_set(self.0.columns())
            && Self::valid_set(self.0.exact_chunks((cell_size, cell_size)))
    }

    fn order(&self) -> usize {
        self.0.dim().0
    }

    fn cell_size(&self) -> usize {
        (self.order() as f64).sqrt() as usize
    }

    fn valid_set<'a, D: Dimension>(
        set: impl IntoIterator<Item = ArrayView<'a, SudokuValue, D>>,
    ) -> bool {
        set.into_iter().all(|subset| {
            subset.iter().enumerate().all(|(ix, value)| {
                value.is_none() || !subset.iter().skip(ix + 1).any(|val| val == value)
            })
        })
    }

    fn required_padding(&self) -> usize {
        let order = self.order();

        if order < 10 {
            3
        } else if order < 100 {
            4
        } else {
            5
        }
    }
}

impl FromStr for Sudoku {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert!(
            s.len() == 4 * 4 || s.len() == 9 * 9,
            "Only works with 4x4 and 9x9 Sudoku puzzles"
        );
        let vals: Result<Vec<_>, _> = s
            .chars()
            .map(|c| {
                let mut buffer = [0; 4];
                c.encode_utf8(&mut buffer).parse::<SudokuValue>()
            })
            .collect();

        let vals = vals?;
        Ok(Self::from_order_vec(
            match vals.len() {
                16 => 4,
                81 => 9,
                _ => unreachable!(),
            },
            vals,
        ))
    }
}

impl FromStr for SudokuValue {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert_eq!(s.len(), 1, "Only works with 4x4 and 9x9 Sudoku puzzles");
        if s == "." {
            Ok(SudokuValue(None))
        } else {
            s.parse::<NonZeroU8>().map(|val| SudokuValue(Some(val)))
        }
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell_size = self.cell_size();
        let padding = self.required_padding() - 1;

        let horizontal_line = format!(
            "{}+",
            format!("+{}", "-".repeat(cell_size * padding + 1)).repeat(cell_size)
        );

        for (ix, row) in self.0.rows().into_iter().enumerate() {
            if (ix) % cell_size == 0 {
                writeln!(f, "{horizontal_line}")?;
            }

            write!(f, "|")?;
            for (ix, ele) in row.into_iter().enumerate() {
                write!(f, "{:>padding$}", format!("{ele}"))?;
                if ix % cell_size == cell_size - 1 {
                    write!(f, " |")?;
                }
            }
            writeln!(f)?;
        }

        write!(f, "{horizontal_line}")
    }
}

impl Deref for SudokuValue {
    type Target = Option<NonZeroU8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SudokuValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(num) = self.0 {
            write!(f, "{num}")
        } else {
            write!(f, ".")
        }
    }
}
