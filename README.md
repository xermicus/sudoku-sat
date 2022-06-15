# Sudoku SAT

Simple sudoku solver using SAT (uses boolector as the SMT prover).

This serves as an example SAT/SMT application.

Usage:
```
cargo run -- sudoku.json
```

The input file is simply a sudoku in a 2D JSON array (see e.g. `sudoku.json` or `sudoku2.json` files).