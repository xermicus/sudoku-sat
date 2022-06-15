use std::{env, fs::File, rc::Rc};

use boolector::{
    option::{BtorOption, ModelGen},
    Btor, BV,
};

/// Read sudoku puzzle from a JSON file. Spec:
/// The JSON file should contain the raw known numbers of the sudoku (9x9) as a 2D array.
/// Unkown cells should be represented as 0. Whitespace is allowed.
fn from_file(path: &str) -> [[u32; 9]; 9] {
    let rdr = File::open(path).expect("can not open input file");
    serde_json::from_reader(rdr).unwrap()
}

/// Return a list of coordinate pairs of all cells which must NOT be equal to this one.
fn must_be_different_to(x: usize, y: usize) -> Vec<(usize, usize)> {
    let x_sub = x - x % 3;
    let y_sub = y - y % 3;
    let mut result = vec![];
    let in_block = |i, j| (i >= x_sub && i < x_sub + 3) && (j >= y_sub && j < y_sub + 3);
    for i in 0..9 {
        for j in 0..9 {
            if x == i && y == j {
                continue;
            } else if in_block(i, j) || x == i || y == j {
                result.push((i, j))
            }
        }
    }
    result
}

struct SudokuSolver {
    btor: Rc<Btor>,
    cells: Vec<BV<Rc<Btor>>>,
}

impl SudokuSolver {
    /// Construct a new instance.
    fn new(sudoku: &[[u32; 9]; 9]) -> Self {
        let btor = Rc::new(Btor::new());
        btor.set_opt(BtorOption::ModelGen(ModelGen::All));
        let mut cells = vec![];
        let one = BV::from_u32(btor.clone(), 1, 8);
        let nine = BV::from_u32(btor.clone(), 9, 8);
        for row in sudoku {
            for digit in row {
                let value = if *digit == 0 {
                    BV::new(btor.clone(), 8, None)
                } else {
                    BV::from_u32(btor.clone(), *digit, 8)
                };
                value.ugte(&one).assert(); // cells can only be between 1..
                value.ulte(&nine).assert(); // .. and 9
                cells.push(value);
            }
        }
        SudokuSolver { btor, cells }
    }

    /// Apply constraints to solve the sudoku.
    fn constrain(&self) {
        let idx = |x, y| x * 9 + y;
        for x in 0..9 {
            for y in 0..9 {
                let cell = &self.cells[idx(x, y)];
                for (x, y) in must_be_different_to(x, y) {
                    cell._ne(&self.cells[idx(x, y)]).assert(); // The rules of the sudoku ^_^
                }
            }
        }
    }

    /// Print out whether the sudoku has a solution or not.
    /// Print the model if it is.
    fn sat(&self) {
        match self.btor.sat() {
            boolector::SolverResult::Sat => {
                println!("hooray! This sudoku is satisfiable (SAT). Model: ");
                for (i, cell) in self.cells.iter().enumerate() {
                    if i % (3 * 9) == 0 {
                        println!()
                    }
                    if i % 9 == 0 {
                        println!()
                    } else if i % 3 == 0 {
                        print!("\t")
                    }
                    print!("{} ", cell.get_a_solution().as_u64().unwrap())
                }
            }
            _ => println!("oops, can't solve this one"),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No input file!\nUsage: {} <sudoku.json>", args[0]);
        return;
    }
    let sudoku = from_file(&args[1]);
    let solver = SudokuSolver::new(&sudoku);
    solver.constrain();
    solver.sat();
}
