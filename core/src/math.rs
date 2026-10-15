//! Self-contained arithmetic expression evaluator.
//!
//! Public entry point is [`eval`]. The implementation is a classic
//! tokenizer + recursive-descent parser with the following grammar:
//!
//! ```text
//! expr    := term (('+' | '-') term)*
//! term    := unary (('*' | '/') unary)*
//! unary   := ('+' | '-') unary | power
//! power   := primary ('^' unary)?          // '^' is RIGHT-associative
//! primary := NUMBER | IDENT | IDENT '(' expr ')' | '(' expr ')'
//! ```
//!
//! Notes on the non-obvious bits:
//! * `power` recurses through `unary` on the right of `^`. This makes `^`
//!   right-associative (`2^3^2 == 2^(3^2) == 512`) *and* lets a unary sign
//!   appear on the exponent (`2^-2 == 0.25`).
//! * Because `unary` calls `power`, a leading `-` binds looser than `^`, so
//!   `-2^2` is `-(2^2) == -4`, matching normal mathematical convention.
//! * The tokenizer only consumes an `e`/`E` as part of a number when it is
//!   actually followed by an optional sign and at least one digit, so `e`
//!   can still be recognised as the constant `e`.
//!
//! Nothing here uses `unsafe`, `unwrap`, or `panic!`; malformed input always
//! comes back as `Err(String)`. Division by zero is *not* an error: it yields
//! `inf`/`NaN` via ordinary `f64` arithmetic and the caller decides what to do.

/// Evaluate an arithmetic expression.
///
/// Returns the computed value, or a human-readable error message describing
/// where the input failed to parse.
pub fn eval(input: &str) -> Result<f64, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    let value = parser.expr()?;
    match parser.peek() {
        Token::Eof => Ok(value),
        other => Err(format!("unexpected trailing token {}", describe(&other))),
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Eof,
}

impl Token {
    fn describe(&self) -> String {
        match self {
            Token::Number(n) => format!("number `{n}`"),
            Token::Ident(s) => format!("identifier `{s}`"),
            Token::Plus => "`+`".into(),
            Token::Minus => "`-`".into(),
            Token::Star => "`*`".into(),
            Token::Slash => "`/`".into(),
            Token::Caret => "`^`".into(),
            Token::LParen => "`(`".into(),
            Token::RParen => "`)`".into(),
            Token::Eof => "end of input".into(),
        }
    }
}

/// Small helper so callers can format tokens uniformly.
fn describe(token: &Token) -> String {
    token.describe()
}

/// Turn the raw input into a flat token stream, appending a sentinel [`Token::Eof`].
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            // Whitespace is ignored anywhere (including Unicode spaces, which
            // are common when pasting from web pages or spreadsheets).
            _ if c.is_whitespace() => i += 1,
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            _ if c.is_ascii_digit() || c == '.' => {
                let start = i;
                // Integer part.
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                // Fractional part.
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                // Optional scientific-notation exponent, but only when it is
                // actually well-formed (`e`/`E`, optional sign, >= 1 digit).
                // Otherwise we leave the `e` alone so it can be the constant.
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    let mut j = i + 1;
                    if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                        j += 1;
                    }
                    if j < chars.len() && chars[j].is_ascii_digit() {
                        i = j;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                let text: String = chars[start..i].iter().collect();
                let value = text
                    .parse::<f64>()
                    .map_err(|_| format!("invalid number `{text}`"))?;
                tokens.push(Token::Number(value));
            }
            _ if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(text));
            }
            _ => return Err(format!("unexpected character `{c}`")),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}

/// Maximum nesting depth (parentheses, unary chains, or nested calls) before we
/// bail out. Guards against a stack overflow / process abort from pathological
/// input such as thousands of `(` characters.
const MAX_DEPTH: usize = 128;

impl Parser {
    /// Peek at the current token. Returns `Eof` past the end so we never index
    /// out of bounds (the tokenizer always terminates with `Eof` anyway).
    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    /// `expr := term (('+' | '-') term)*`
    fn expr(&mut self) -> Result<f64, String> {
        let mut lhs = self.term()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    lhs += self.term()?;
                }
                Token::Minus => {
                    self.advance();
                    lhs -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    /// `term := unary (('*' | '/') unary)*`
    fn term(&mut self) -> Result<f64, String> {
        let mut lhs = self.unary()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    lhs *= self.unary()?;
                }
                Token::Slash => {
                    self.advance();
                    // Deliberately not checked: `x / 0.0` yields inf/NaN.
                    lhs /= self.unary()?;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    /// `unary := ('+' | '-') unary | power`
    ///
    /// Every recursive path in the grammar funnels through here, so this is
    /// where the depth limit is enforced: runaway nesting becomes an `Err`
    /// instead of a stack overflow (which would abort the whole process).
    fn unary(&mut self) -> Result<f64, String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err("expression nested too deeply".to_string());
        }
        let result = match self.peek() {
            Token::Minus => {
                self.advance();
                Ok(-self.unary()?)
            }
            Token::Plus => {
                self.advance();
                self.unary()
            }
            _ => self.power(),
        };
        self.depth -= 1;
        result
    }

    /// `power := primary ('^' unary)?`
    ///
    /// The right operand is parsed with `unary`, which is what gives `^` its
    /// right associativity and also permits `2^-3`.
    fn power(&mut self) -> Result<f64, String> {
        let base = self.primary()?;
        if self.peek() == Token::Caret {
            self.advance();
            let exponent = self.unary()?;
            Ok(base.powf(exponent))
        } else {
            Ok(base)
        }
    }

    /// `primary := NUMBER | IDENT | IDENT '(' expr ')' | '(' expr ')'`
    fn primary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Token::Number(n) => {
                self.advance();
                Ok(n)
            }
            Token::LParen => {
                self.advance();
                let value = self.expr()?;
                match self.peek() {
                    Token::RParen => {
                        self.advance();
                        Ok(value)
                    }
                    other => Err(format!("expected `)`, found {}", describe(&other))),
                }
            }
            Token::Ident(name) => {
                self.advance();
                if self.peek() == Token::LParen {
                    // Function call.
                    self.advance();
                    let arg = self.expr()?;
                    match self.peek() {
                        Token::RParen => {
                            self.advance();
                        }
                        other => {
                            return Err(format!(
                                "expected `)` after argument to `{name}`, found {}",
                                describe(&other)
                            ));
                        }
                    }
                    apply_function(&name, arg)
                } else {
                    // Bare identifier: must be a known constant.
                    constant(&name)
                }
            }
            other => Err(format!("unexpected token {}", describe(&other))),
        }
    }
}

// ---------------------------------------------------------------------------
// Constants and functions
// ---------------------------------------------------------------------------

fn constant(name: &str) -> Result<f64, String> {
    match name {
        "pi" => Ok(std::f64::consts::PI),
        "e" => Ok(std::f64::consts::E),
        _ => Err(format!("unknown identifier `{name}`")),
    }
}

fn apply_function(name: &str, x: f64) -> Result<f64, String> {
    match name {
        "sqrt" => Ok(x.sqrt()),
        "abs" => Ok(x.abs()),
        "sin" => Ok(x.sin()),
        "cos" => Ok(x.cos()),
        "tan" => Ok(x.tan()),
        "ln" => Ok(x.ln()),
        "log" => Ok(x.log10()),
        "exp" => Ok(x.exp()),
        "floor" => Ok(x.floor()),
        "ceil" => Ok(x.ceil()),
        "round" => Ok(x.round()),
        _ => Err(format!("unknown function `{name}`")),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::eval;

    /// Compare floating-point results with a small tolerance.
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn ok(input: &str) -> f64 {
        match eval(input) {
            Ok(v) => v,
            Err(e) => panic!("expected `{input}` to parse, got error: {e}"),
        }
    }

    fn is_err(input: &str) {
        if let Ok(v) = eval(input) {
            panic!("expected `{input}` to fail, but got {v}");
        }
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(ok("1+2"), 3.0);
        assert_eq!(ok("5-8"), -3.0);
        assert_eq!(ok("3*4"), 12.0);
        assert_eq!(ok("9/3"), 3.0);
    }

    #[test]
    fn precedence() {
        assert_eq!(ok("2+3*4"), 14.0);
        assert_eq!(ok("2*3+4"), 10.0);
        assert_eq!(ok("2+3*4-1"), 13.0);
        assert_eq!(ok("2*2+2/2"), 5.0);
    }

    #[test]
    fn left_associativity() {
        assert_eq!(ok("10-3-2"), 5.0);
        assert_eq!(ok("100/10/2"), 5.0);
        assert_eq!(ok("20-5+3"), 18.0);
    }

    #[test]
    fn power_is_right_associative() {
        assert_eq!(ok("2^3^2"), 512.0);
        assert_eq!(ok("2^(3^2)"), 512.0);
        assert_eq!(ok("(2^3)^2"), 64.0);
        assert_eq!(ok("2^2^3"), 256.0);
    }

    #[test]
    fn power_binds_tighter_than_mul_and_add() {
        assert_eq!(ok("2*3^2"), 18.0);
        assert_eq!(ok("1+2^3"), 9.0);
    }

    #[test]
    fn parentheses() {
        assert_eq!(ok("(2+3)*4"), 20.0);
        assert_eq!(ok("2*(3+4)"), 14.0);
        assert_eq!(ok("((((1))))"), 1.0);
        assert_eq!(ok("(1+2)*(3+4)"), 21.0);
    }

    #[test]
    fn unary_minus() {
        assert_eq!(ok("-5"), -5.0);
        assert_eq!(ok("-(2+3)"), -5.0);
        assert_eq!(ok("3 * -2"), -6.0);
        assert_eq!(ok("--5"), 5.0);
        assert_eq!(ok("+7"), 7.0);
        // Unary binds looser than `^`.
        assert_eq!(ok("-2^2"), -4.0);
        assert_eq!(ok("2^-2"), 0.25);
        assert_eq!(ok("-3^-2"), -(3f64.powf(-2.0)));
    }

    #[test]
    fn whitespace_is_ignored() {
        assert_eq!(ok("  1  +  2  *  3  "), 7.0);
        assert_eq!(ok("\t2\n*\n(3 + 4)\r"), 14.0);
        assert_eq!(ok(" sqrt ( 9 ) "), 3.0);
    }

    #[test]
    fn decimals_and_scientific() {
        assert_eq!(ok(".5"), 0.5);
        assert_eq!(ok("0.5"), 0.5);
        assert_eq!(ok("1.25"), 1.25);
        assert_eq!(ok("1e3"), 1000.0);
        assert_eq!(ok("1E3"), 1000.0);
        assert_eq!(ok("1.5e-2"), 0.015);
        assert_eq!(ok("2e2+1"), 201.0);
    }

    #[test]
    fn constants() {
        assert!(approx(ok("pi"), std::f64::consts::PI));
        assert!(approx(ok("e"), std::f64::consts::E));
        assert!(approx(ok("pi*2"), std::f64::consts::PI * 2.0));
        assert!(approx(ok("e^2"), std::f64::consts::E.powi(2)));
    }

    #[test]
    fn functions_all() {
        assert!(approx(ok("sqrt(2)"), 2f64.sqrt()));
        assert_eq!(ok("sqrt(9)"), 3.0);
        assert_eq!(ok("abs(-7)"), 7.0);
        assert_eq!(ok("abs(7)"), 7.0);
        assert!(approx(ok("sin(0)"), 0.0));
        assert!(approx(ok("sin(pi/2)"), 1.0));
        assert!(approx(ok("cos(0)"), 1.0));
        assert!(approx(ok("cos(pi)"), -1.0));
        assert!(approx(ok("tan(0)"), 0.0));
        assert!(approx(ok("ln(e)"), 1.0));
        assert!(approx(ok("log(1000)"), 3.0));
        assert!(approx(ok("log(1)"), 0.0));
        assert!(approx(ok("exp(0)"), 1.0));
        assert!(approx(ok("exp(1)"), std::f64::consts::E));
        assert_eq!(ok("floor(1.9)"), 1.0);
        assert_eq!(ok("floor(-1.1)"), -2.0);
        assert_eq!(ok("ceil(1.1)"), 2.0);
        assert_eq!(ok("ceil(-1.9)"), -1.0);
        assert_eq!(ok("round(2.5)"), 3.0);
        assert_eq!(ok("round(2.4)"), 2.0);
    }

    #[test]
    fn nested_functions() {
        assert_eq!(ok("floor(sqrt(10))"), 3.0);
        assert!(approx(ok("abs(sin(0) - 1)"), 1.0));
        assert_eq!(ok("round(sqrt(16)) + ceil(0.1)"), 5.0);
    }

    #[test]
    fn division_by_zero_yields_inf_or_nan() {
        assert!(ok("1/0").is_infinite() && ok("1/0").is_sign_positive());
        assert!(ok("-1/0").is_infinite() && ok("-1/0").is_sign_negative());
        assert!(ok("0/0").is_nan());
    }

    #[test]
    fn malformed_inputs_are_errors() {
        is_err("1+");
        is_err("(1+2");
        is_err("*3");
        is_err("sqrt(");
        is_err("");
        is_err("1 2");
        is_err("2**3");
        is_err("1+*2");
        is_err("()");
        is_err(")");
        is_err("(1))");
        is_err("foo(3)");
        is_err("nope");
        is_err("1@@2");
        is_err("1..2");
    }

    #[test]
    fn deep_nesting_is_rejected_not_crashed() {
        // Without a depth guard these overflow the stack and abort the process.
        let parens = "(".repeat(10_000) + &")".repeat(10_000);
        assert!(eval(&parens).is_err());
        let signs = format!("{}1", "-".repeat(10_000));
        assert!(eval(&signs).is_err());
        let nested = "sqrt(".repeat(10_000);
        assert!(eval(&nested).is_err());
        // Reasonable nesting still works.
        assert_eq!(eval("((((1+2))))").unwrap(), 3.0);
    }

    #[test]
    fn unicode_whitespace_is_ignored() {
        assert_eq!(eval("1\u{00A0}+\u{2003}2").unwrap(), 3.0);
    }
}
