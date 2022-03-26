use std::fmt::{self, Write};

use proc_macro2::Span;
use syn::{
    parse::{Error, Result},
    spanned::Spanned,
    Attribute, Ident, Lit, LitStr, Meta, NestedMeta, Path,
};

use crate::util::LitExt;

// line 73
pub fn parse_values(attr: &Attribute) -> Result<Values> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Path(path) => {
            let name = to_ident(path)?;

            Ok(Values::new(name, ValueKind::Name, Vec::new(), attr.span()))
        }
        Meta::List(meta) => {
            let name = to_ident(meta.path)?;
            let nested = meta.nested;

            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

            let mut lits = Vec::with_capacity(nested.len());

            for meta in nested {
                match meta {
                    NestedMeta::Lit(l) => lits.push(l),
                    NestedMeta::Meta(m) => match m {
                        Meta::Path(path) => {
                            let i = to_ident(path)?;
                            lits.push(Lit::Str(LitStr::new(&i.to_string(), i.span())))
                        }
                        Meta::List(_) | Meta::NameValue(_) => {
                            return Err(Error::new(attr.span(), "cannot nest a list; only accept literals and identifiers at this level"))
                        }
                    },
                }
            }

            let kind = if lits.len() == 1 {
                ValueKind::SingleList
            } else {
                ValueKind::List
            };

            Ok(Values::new(name, kind, lits, attr.span()))
        }
        Meta::NameValue(meta) => {
            let name = to_ident(meta.path)?;
            let lit = meta.lit;
            Ok(Values::new(name, ValueKind::Equals, vec![lit], attr.span()))
        }
    }
}

// line 53
#[derive(Debug)]
pub struct Values {
    pub name: Ident,
    pub literals: Vec<Lit>,
    pub kind: ValueKind,
    pub span: Span,
}

impl Values {
    #[inline]
    pub fn new(name: Ident, kind: ValueKind, literals: Vec<Lit>, span: Span) -> Self {
        Values {
            name,
            literals,
            kind,
            span,
        }
    }
}

// line 11
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueKind {
    // #[<name>]
    Name,

    // #[<name> = <value>]
    Equals,

    // #[<name>([<value>, <value>, <value>, ...])]
    List,

    // #[<name>(<value>)]
    SingleList,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Name => f.pad("`#[<name>]`"),
            ValueKind::Equals => f.pad("`#[<name> = <value>]`"),
            ValueKind::List => f.pad("`#[<name>([<value>, <value>, <value>, ...])]"),
            ValueKind::SingleList => f.pad("`#[<name>(<value>)]`"),
        }
    }
}

// line 37
fn to_ident(p: Path) -> Result<Ident> {
    if p.segments.is_empty() {
        return Err(Error::new(
            p.span(),
            "cannot convert an empty path to an identifier",
        ));
    }

    if p.segments.len() > 1 {
        return Err(Error::new(
            p.span(),
            "the path must not have more than one segment",
        ));
    }

    if !p.segments[0].arguments.is_empty() {
        return Err(Error::new(
            p.span(),
            "the singular path segment must not have any arguments",
        ));
    }

    Ok(p.segments[0].ident.clone())
}

// line 120
#[derive(Debug, Clone)]
struct DisplaySlice<'a, T>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter().enumerate();

        match iter.next() {
            None => f.write_str("nothing")?,
            Some((idx, elem)) => {
                write!(f, "{}: {}", idx, elem)?;

                for (idx, elem) in iter {
                    f.write_char('\n')?;
                    write!(f, "{}: {}", idx, elem)?;
                }
            }
        }

        Ok(())
    }
}

// line 143
#[inline]
fn is_form_acceptable(expect: &[ValueKind], kind: ValueKind) -> bool {
    if expect.contains(&ValueKind::List) && kind == ValueKind::SingleList {
        true
    } else {
        expect.contains(&kind)
    }
}

// line 152
#[inline]
fn validate(values: &Values, forms: &[ValueKind]) -> Result<()> {
    if !is_form_acceptable(forms, values.kind) {
        return Err(Error::new(
            values.span,
            format_args!(
                "the attribute must be in of these forms:\n{}",
                DisplaySlice(forms)
            ),
        ));
    }

    Ok(())
}

// line 165
#[inline]
pub fn parse<T: AttributeOption>(values: Values) -> Result<T> {
    T::parse(values)
}

pub trait AttributeOption: Sized {
    fn parse(values: Values) -> Result<Self>;
}

// line 182
impl AttributeOption for String {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Equals, ValueKind::SingleList])?;

        Ok(values.literals[0].to_str())
    }
}
