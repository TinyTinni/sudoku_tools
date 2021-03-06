extern crate array_init;
extern crate varisat;
use varisat::ExtendFormula;

pub struct SudokuSolver<'a> {
    solver: varisat::Solver<'a>,
}

impl<'a> SudokuSolver<'a> {
    pub fn new() -> SudokuSolver<'a> {
        let mut s = varisat::Solver::new();
        SudokuSolver::add_uniquness(&mut s, 9, 9);
        for i in 0..9 {
            SudokuSolver::add_row_rule(&mut s, i);
            SudokuSolver::add_column_rule(&mut s, i);
            SudokuSolver::add_box_rule(&mut s, 3 * (i % 3), 3 * (i / 3));
        }
        SudokuSolver { solver: s }
    }

    /// solves a sudoku.
    /// # Arguments
    /// * `assumptions` - Already filled numbers. Use Grid to generate assumptions from already filled cells.
    pub fn solve(&mut self, assumptions: &[varisat::Lit]) -> Option<Vec<varisat::Lit>> {
        self.solver.assume(assumptions);
        let sol = self.solver.solve();
        match sol {
            Ok(_) => self.solver.model(),
            Err(_) => None,
        }
    }

    /// returns a literal from a sudoku cell \note: value \in [0;8]
    pub fn value_to_lit(v: u8, row: usize, column: usize) -> varisat::Lit {
        let index = row * 9 * 9 + column * 9 + (v as usize) + 1;
        varisat::Lit::from_index(index, true)
    }

    /// returns a sudoku cell with value from a literal "(value, row, column)" \note: value \in [0;8]
    pub fn lit_to_value(i: varisat::Lit) -> (u8, usize, usize) {
        let v = i.index() - 1;
        let row = v / (9 * 9);
        let column = (v - row * 9 * 9) / 9;
        let v = v - row * 9 * 9 - column * 9;
        return (v as u8, row, column);
    }

    fn exactly_one(solver: &mut varisat::Solver, lits: &[varisat::Lit]) {
        // at least one has to be true
        solver.add_clause(lits);

        //make pairs, if one is true, the other has to be false
        let n = lits.len();
        for i in 0..n {
            for j in (i + 1)..n {
                solver.add_clause(&[!lits[i], !lits[j]]);
            }
        }
    }

    fn add_uniquness(solver: &mut varisat::Solver, rows: usize, columns: usize) {
        for r in 0..rows {
            for c in 0..columns {
                let lits: [varisat::Lit; 9] =
                    array_init::array_init(|v| SudokuSolver::value_to_lit(v as u8, r, c));
                SudokuSolver::exactly_one(solver, &lits);
            }
        }
    }

    fn add_column_rule(solver: &mut varisat::Solver, column: usize) {
        for v in 0..9 {
            let lits: [varisat::Lit; 9] =
                array_init::array_init(|r| SudokuSolver::value_to_lit(v, r, column));
            SudokuSolver::exactly_one(solver, &lits);
        }
    }

    fn add_row_rule(solver: &mut varisat::Solver, row: usize) {
        for v in 0..9 {
            let lits: [varisat::Lit; 9] =
                array_init::array_init(|c| SudokuSolver::value_to_lit(v, row, c));
            SudokuSolver::exactly_one(solver, &lits);
        }
    }

    fn add_box_rule(solver: &mut varisat::Solver, row: usize, column: usize) {
        for v in 0..9 {
            let lits: [varisat::Lit; 9] = array_init::array_init(|i| {
                SudokuSolver::value_to_lit(v, row + (i / 3), column + (i % 3))
            });
            SudokuSolver::exactly_one(solver, &lits);
        }
    }
}

/////////////////////////////////////////////////////////////
/// Test
/////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_exactly_one() {
        let mut solver = varisat::Solver::new();
        let (x, y, z) = solver.new_lits();
        SudokuSolver::exactly_one(&mut solver, &[x, y, z]);

        solver.assume(&[!y, !z]);
        assert_eq!(solver.solve().unwrap(), true);
        assert!(solver.model().unwrap().contains(&x));

        solver.assume(&[!x, !z]);
        assert_eq!(solver.solve().unwrap(), true);
        assert!(solver.model().unwrap().contains(&y));

        solver.assume(&[!x, !y, !z]);
        assert_eq!(solver.solve().unwrap(), false);

        solver.assume(&[!x]);
        assert_eq!(solver.solve().unwrap(), true);
        assert!(solver.model().unwrap().contains(&y) || solver.model().unwrap().contains(&z));
    }

    #[test]
    fn test_add_row_rule() {
        let mut solver = varisat::Solver::new();
        SudokuSolver::add_uniquness(&mut solver, 1, 9);
        SudokuSolver::add_row_rule(&mut solver, 0);

        let mut vec = Vec::new();
        for i in 0..8 {
            let v: u8 = i as u8;
            vec.push(SudokuSolver::value_to_lit(v, 0, i));
        }

        solver.assume(&vec);
        assert_eq!(solver.solve().unwrap(), true);

        let proofed_literal = SudokuSolver::value_to_lit(8, 0, 8);
        let m = solver.model().unwrap();
        assert_eq!(m.contains(&proofed_literal), true);
    }

    #[test]
    fn test_value_to_index() {
        let (value, row, column) = SudokuSolver::lit_to_value(varisat::Lit::from_index(432, true));
        assert_eq!(row, 5);
        assert_eq!(column, 2);
        assert_eq!(value, 9);
    }
}
