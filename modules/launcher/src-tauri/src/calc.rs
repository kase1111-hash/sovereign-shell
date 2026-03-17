//! Calculator mode for the Sovereign Launcher.
//!
//! When the search input starts with `=`, the expression is evaluated as math.
//! Uses a simple recursive descent parser — no external dependencies.
//!
//! Supported: +, -, *, /, %, ^ (power), parentheses, and common functions
//! (sqrt, sin, cos, tan, abs, ln, log, ceil, floor, round).
//! Constants: pi, e, tau.

/// Evaluate a math expression string. Returns the result or an error message.
pub fn evaluate(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err("Unexpected token after expression".to_string());
    }
    Ok(result)
}

/// Format a result for display — integers show without decimal, floats show up to 10 digits.
pub fn format_result(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if value == value.trunc() && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        let s = format!("{:.10}", value);
        // Trim trailing zeros
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => i += 1,
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '^' => { tokens.push(Token::Caret); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num = num_str.parse::<f64>()
                    .map_err(|_| format!("Invalid number: {}", num_str))?;
                tokens.push(Token::Number(num));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let ident: String = chars[start..i].iter().collect();
                // Resolve constants immediately
                match ident.to_lowercase().as_str() {
                    "pi" => tokens.push(Token::Number(std::f64::consts::PI)),
                    "e" => tokens.push(Token::Number(std::f64::consts::E)),
                    "tau" => tokens.push(Token::Number(std::f64::consts::TAU)),
                    _ => tokens.push(Token::Ident(ident.to_lowercase())),
                }
            }
            c => return Err(format!("Unexpected character: '{}'", c)),
        }
    }

    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    // expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => { self.advance(); left += self.parse_term()?; }
                Some(Token::Minus) => { self.advance(); left -= self.parse_term()?; }
                _ => break,
            }
        }
        Ok(left)
    }

    // term = power (('*' | '/' | '%') power)*
    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_power()?;
        loop {
            match self.peek() {
                Some(Token::Star) => { self.advance(); left *= self.parse_power()?; }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 { return Err("Division by zero".to_string()); }
                    left /= right;
                }
                Some(Token::Percent) => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 { return Err("Modulo by zero".to_string()); }
                    left %= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // power = unary ('^' power)?  (right-associative)
    fn parse_power(&mut self) -> Result<f64, String> {
        let base = self.parse_unary()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let exp = self.parse_power()?; // right-associative
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    // unary = '-' unary | '+' unary | primary
    fn parse_unary(&mut self) -> Result<f64, String> {
        match self.peek() {
            Some(Token::Minus) => { self.advance(); Ok(-self.parse_unary()?) }
            Some(Token::Plus) => { self.advance(); self.parse_unary() }
            _ => self.parse_primary(),
        }
    }

    // primary = number | ident '(' args ')' | '(' expr ')'
    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.advance().cloned() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::Ident(name)) => {
                // Must be followed by '('
                match self.peek() {
                    Some(Token::LParen) => {
                        self.advance(); // consume '('
                        let args = self.parse_args()?;
                        self.expect_rparen()?;
                        apply_function(&name, &args)
                    }
                    _ => Err(format!("Unknown identifier: {}", name)),
                }
            }
            Some(Token::LParen) => {
                let val = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(val)
            }
            Some(tok) => Err(format!("Unexpected token: {:?}", tok)),
            None => Err("Unexpected end of expression".to_string()),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<f64>, String> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.advance() {
            Some(Token::RParen) => Ok(()),
            _ => Err("Expected closing parenthesis".to_string()),
        }
    }
}

fn apply_function(name: &str, args: &[f64]) -> Result<f64, String> {
    match (name, args.len()) {
        ("sqrt", 1) => Ok(args[0].sqrt()),
        ("abs", 1) => Ok(args[0].abs()),
        ("sin", 1) => Ok(args[0].sin()),
        ("cos", 1) => Ok(args[0].cos()),
        ("tan", 1) => Ok(args[0].tan()),
        ("asin", 1) => Ok(args[0].asin()),
        ("acos", 1) => Ok(args[0].acos()),
        ("atan", 1) => Ok(args[0].atan()),
        ("ln", 1) => Ok(args[0].ln()),
        ("log", 1) => Ok(args[0].log10()),
        ("log", 2) => Ok(args[0].log(args[1])),
        ("ceil", 1) => Ok(args[0].ceil()),
        ("floor", 1) => Ok(args[0].floor()),
        ("round", 1) => Ok(args[0].round()),
        ("min", 2) => Ok(args[0].min(args[1])),
        ("max", 2) => Ok(args[0].max(args[1])),
        ("pow", 2) => Ok(args[0].powf(args[1])),
        (name, n) => Err(format!("Unknown function: {}({} args)", name, n)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(s: &str) -> f64 {
        evaluate(s).unwrap()
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(eval("2 + 3"), 5.0);
        assert_eq!(eval("10 - 3"), 7.0);
        assert_eq!(eval("4 * 5"), 20.0);
        assert_eq!(eval("15 / 3"), 5.0);
        assert_eq!(eval("7 % 3"), 1.0);
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(eval("2 + 3 * 4"), 14.0);
        assert_eq!(eval("(2 + 3) * 4"), 20.0);
        assert_eq!(eval("2 ^ 3 ^ 2"), 512.0); // right-associative: 2^(3^2)
    }

    #[test]
    fn unary_minus() {
        assert_eq!(eval("-5"), -5.0);
        assert_eq!(eval("-(3 + 2)"), -5.0);
        assert_eq!(eval("--5"), 5.0);
    }

    #[test]
    fn functions() {
        assert!((eval("sqrt(9)") - 3.0).abs() < 1e-10);
        assert!((eval("abs(-42)") - 42.0).abs() < 1e-10);
        assert!((eval("sin(0)")).abs() < 1e-10);
        assert!((eval("cos(0)") - 1.0).abs() < 1e-10);
        assert!((eval("log(100)") - 2.0).abs() < 1e-10);
        assert!((eval("pow(2, 10)") - 1024.0).abs() < 1e-10);
    }

    #[test]
    fn constants() {
        assert!((eval("pi") - std::f64::consts::PI).abs() < 1e-10);
        assert!((eval("e") - std::f64::consts::E).abs() < 1e-10);
        assert!((eval("2 * pi") - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn format() {
        assert_eq!(format_result(42.0), "42");
        assert_eq!(format_result(3.14), "3.14");
        assert_eq!(format_result(0.1 + 0.2), "0.3");
    }

    #[test]
    fn errors() {
        assert!(evaluate("1 / 0").is_err());
        assert!(evaluate("foo(1)").is_err());
        assert!(evaluate("1 +").is_err());
    }
}
