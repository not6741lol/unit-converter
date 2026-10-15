# Convert

A small desktop unit-converter built in Rust with [`eframe`](https://crates.io/crates/eframe)/`egui`.

Pick a category, choose the two units, type a value, and the result updates live as you
type. It's a compact, single-file learning project — nothing more, nothing less.

## Features

- Three categories: **Length**, **Weight**, and **Temperature**.
- **Length:** meter, kilometer, centimeter, millimeter, mile, yard, foot, inch.
- **Weight:** gram, kilogram, milligram, pound, ounce, tonne.
- **Temperature:** Celsius, Fahrenheit, Kelvin.
- Category picker plus two unit dropdowns.
- A swap button (`⇄`) to flip the "from" and "to" units.
- Live conversion — the result recomputes on every keystroke.
- The value box accepts **arithmetic expressions**, not just plain numbers:
  `2+3*4`, `(100-32)*5/9`, `sqrt(2)`, `pi*2`, `2^10`. Supports `+ - * / ^`,
  parentheses, unary minus, the functions `sqrt abs sin cos tan ln log exp floor ceil round`,
  and the constants `pi` and `e`.
- Results are trimmed of trailing zeros (e.g. `1000.000000` shows as `1000`).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed via [rustup](https://rustup.rs/).
  That's it — the toolchain and `cargo` come with it.

## How to run it

Development build (compile and run in one step):

```sh
cargo run
```

Release build:

```sh
cargo build --release
```

The release binary is written to:

```
target/release/convert.exe
```

## How to run the tests

```sh
cargo test
```

The tests cover length, weight, and temperature conversions, the identity case
(converting a unit to itself), the number formatting helper, and the full arithmetic
expression evaluator (precedence, associativity, functions, constants, and malformed
input).

## Project layout

- `src/main.rs` — the GUI (eframe/egui), the unit definitions, and the conversion logic.
- `src/math.rs` — a small self-contained arithmetic expression evaluator (no dependencies).

## How it works

Most categories use a **convert-via-base-unit** approach. Every unit is defined by how
large it is relative to the category's base unit — the metre for length and the gram for
weight (see the `factor` function). Converting then reduces to arithmetic:

```
result = value * factor(from) / factor(to)
```

Temperature is the **special case**. It has offsets as well as scale, so the factor trick
doesn't work. Instead, temperature conversions funnel through Celsius: the input is
converted to Celsius (`to_celsius`), then out to the target unit (`from_celsius`).

The `convert` function in `src/main.rs` picks between these two strategies based on the
selected category.

Whatever you type into the value box is first run through the expression evaluator in
`src/math.rs` (`math::eval`). A plain number like `42` is just a trivial expression, so
this adds calculator powers without changing how normal input works.

## Adding a new unit

For **length** or **weight**, add the unit name to the category's list in
`Category::units`, then give it a size relative to the base unit in `factor`. For example,
a length unit that is 0.1 metres would be `0.1`. No other code changes are needed.

For **temperature**, add the name to the list and extend both `to_celsius` and
`from_celsius` with the forward and inverse formulas.

## License

MIT — see [LICENSE](LICENSE).
