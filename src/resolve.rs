//! Lets people name things (`--category Mercado`) where the API takes IDs.

use crate::error::{Error, Result};
use crate::input;
use crate::models::{Account, Category, Space, Tag};

pub trait Named {
    fn id(&self) -> &str;
    /// Every name the record answers to, compared case-insensitively.
    fn names(&self) -> Vec<&str>;
}

impl Named for Space {
    fn id(&self) -> &str {
        &self.id
    }
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for Account {
    fn id(&self) -> &str {
        &self.id
    }
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for Tag {
    fn id(&self) -> &str {
        &self.id
    }
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for Category {
    fn id(&self) -> &str {
        &self.id
    }
    /// A system category also answers to its key, with or without the
    /// `category.` prefix, so scripts can use names that never get renamed.
    fn names(&self) -> Vec<&str> {
        let mut names = vec![self.name.as_str()];
        if let Some(key) = &self.key {
            names.push(key);
            names.extend(key.strip_prefix("category."));
        }
        names
    }
}

/// Finds the record `query` names, by ID or by name. `what` is the noun
/// used in messages, such as "category".
pub fn find<'a, T: Named>(items: &'a [T], query: &str, what: &str) -> Result<&'a T> {
    let query = query.trim();
    if input::is_id(query) {
        return items
            .iter()
            .find(|item| item.id().eq_ignore_ascii_case(query))
            .ok_or_else(|| Error::usage(format!("no {what} has the ID {query}")));
    }

    let matches: Vec<&T> = items
        .iter()
        .filter(|item| item.names().iter().any(|name| name.eq_ignore_ascii_case(query)))
        .collect();

    match matches.as_slice() {
        [item] => Ok(item),
        [] => Err(Error::usage(format!(
            "no {what} is named `{query}`; list them with `visor {}`",
            list_command(what)
        ))),
        several => Err(Error::usage(format!(
            "{} {what} entries are named `{query}`; pass one of these IDs instead: {}",
            several.len(),
            several
                .iter()
                .map(|item| item.id())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn list_command(what: &str) -> String {
    match what {
        "category" => "categories list".to_owned(),
        other => format!("{other}s list"),
    }
}

/// Picks the space to use when none is configured: the default personal
/// space, or the only space there is.
pub fn default_space(spaces: &[Space]) -> Result<&Space> {
    spaces
        .iter()
        .find(|space| space.default)
        .or(match spaces {
            [only] => Some(only),
            _ => None,
        })
        .ok_or_else(|| Error::usage("pick a space with `visor spaces use <name>` or pass --space"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(id: &str, name: &str, key: Option<&str>) -> Category {
        Category {
            id: id.into(),
            key: key.map(Into::into),
            name: name.into(),
            parent_id: None,
            kind: Some("expense".into()),
            origin: "system".into(),
            hidden: false,
        }
    }

    fn space(id: &str, name: &str, default: bool) -> Space {
        Space {
            id: id.into(),
            name: name.into(),
            kind: "personal".into(),
            default,
            role: "owner".into(),
        }
    }

    const A: &str = "00000000-0000-4000-8000-00000000000a";
    const B: &str = "00000000-0000-4000-8000-00000000000b";

    #[test]
    fn finds_by_name_key_or_id() {
        let items = vec![
            category(A, "Food and drinks", Some("category.food_and_drinks")),
            category(B, "Mercado", None),
        ];

        assert_eq!(find(&items, "mercado", "category").unwrap().id, B);
        assert_eq!(find(&items, "food_and_drinks", "category").unwrap().id, A);
        assert_eq!(
            find(&items, "category.food_and_drinks", "category").unwrap().id,
            A
        );
        assert_eq!(find(&items, &A.to_uppercase(), "category").unwrap().id, A);
    }

    #[test]
    fn explains_a_missing_or_ambiguous_name() {
        let items = vec![category(A, "Mercado", None), category(B, "mercado", None)];

        let ambiguous = find(&items, "Mercado", "category").unwrap_err().to_string();
        assert!(ambiguous.contains(A) && ambiguous.contains(B), "{ambiguous}");

        let missing = find(&items, "Padaria", "category").unwrap_err().to_string();
        assert!(missing.contains("visor categories list"), "{missing}");
    }

    #[test]
    fn picks_the_default_space() {
        let spaces = vec![space(A, "Casa", false), space(B, "Pessoal", true)];
        assert_eq!(default_space(&spaces).unwrap().id, B);

        let only = vec![space(A, "Empresa", false)];
        assert_eq!(default_space(&only).unwrap().id, A);

        let none = vec![space(A, "Casa", false), space(B, "Empresa", false)];
        assert!(default_space(&none).is_err());
    }
}
