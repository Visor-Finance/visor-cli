//! Minimal CSV writing (RFC 4180) for spreadsheet exports.

/// Quotes a field when it holds a comma, a quote, or a line break.
#[must_use]
pub fn field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

/// A text field a spreadsheet must not run as a formula: a leading `=`, `+`,
/// `-`, `@`, tab, or carriage return gets a `'` (OWASP CSV injection).
#[must_use]
pub fn text(value: &str) -> String {
    if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        field(&format!("'{value}"))
    } else {
        field(value)
    }
}

/// One CSV line, with CRLF as RFC 4180 asks.
#[must_use]
pub fn row(fields: &[String]) -> String {
    format!("{}\r", fields.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_only_when_needed() {
        assert_eq!(field("Padaria"), "Padaria");
        assert_eq!(field("Pão, leite"), "\"Pão, leite\"");
        assert_eq!(field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(field("two\nlines"), "\"two\nlines\"");
    }

    #[test]
    fn defuses_formulas_in_text() {
        assert_eq!(text("=HYPERLINK(\"x\")"), "\"'=HYPERLINK(\"\"x\"\")\"");
        assert_eq!(text("+55 11"), "'+55 11");
        assert_eq!(text("Mercado"), "Mercado");
    }
}
