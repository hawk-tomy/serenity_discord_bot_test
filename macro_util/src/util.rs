use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Error, Parse, ParseStream, Result as SynResult},
    punctuated::Punctuated,
    token::{Comma, Mut},
    Attribute, Ident, Lit, Path, PathSegment, Type,
};

// line 275
pub fn rename_attributes(attributes: &mut Vec<Attribute>, name: &str, target: &str) {
    for attr in attributes {
        if attr.path.is_ident(name) {
            attr.path = Path::from(PathSegment::from(Ident::new(target, Span::call_site())));
        }
    }
}

// maybe parse comma separate token
#[derive(Debug)] // line 119
pub struct Parenthesised<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesised<T> {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let content;
        parenthesized!(content in input);

        Ok(Parenthesised(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)] // line 157
pub struct Argument {
    pub mutable: Option<Mut>,
    pub name: Ident,
    pub kind: Type,
}

impl ToTokens for Argument {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Argument {
            mutable,
            name,
            kind,
        } = self;

        stream.extend(quote! {
            #mutable #name: #kind
        });
    }
}

// line 25
pub trait LitExt {
    fn to_str(&self) -> String;
    fn to_bool(&self) -> bool;
    fn to_ident(&self) -> Ident;
}

impl LitExt for Lit {
    fn to_str(&self) -> String {
        match self {
            Lit::Str(s) => s.value(),
            Lit::ByteStr(s) => unsafe { String::from_utf8_unchecked(s.value()) },
            Lit::Char(c) => c.value().to_string(),
            Lit::Byte(b) => (b.value() as char).to_string(),
            _ => panic!("values must be a (byte)string or a char"),
        }
    }

    fn to_bool(&self) -> bool {
        if let Lit::Bool(b) = self {
            b.value
        } else {
            self.to_str()
                .parse()
                .unwrap_or_else(|_| panic!("expected bool from {:?}", self))
        }
    }

    #[inline]
    fn to_ident(&self) -> Ident {
        Ident::new(&self.to_str(), self.span())
    }
}

// line 56
pub trait IdentExt2: Sized {
    fn to_string_non_raw(&self) -> String;
    fn to_uppercase(&self) -> Self;
    fn with_suffix(&self, suf: &str) -> Ident;
}

impl IdentExt2 for Ident {
    #[inline]
    fn to_string_non_raw(&self) -> String {
        let ident_string = self.to_string();
        ident_string.trim_start_matches("r#").into()
    }

    #[inline]
    fn to_uppercase(&self) -> Self {
        // This should be valid because keywords are lowercase.
        format_ident!("{}", self.to_string_non_raw().to_uppercase())
    }

    #[inline]
    fn with_suffix(&self, suffix: &str) -> Ident {
        format_ident!("{}_{}", self.to_uppercase(), suffix)
    }
}

// line 131
#[derive(Debug)]
pub struct AsOption<T>(pub Option<T>);

impl<T> AsOption<T> {
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> AsOption<U> {
        AsOption(self.0.map(f))
    }
}

impl<T: ToTokens> ToTokens for AsOption<T> {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        match &self.0 {
            Some(o) => stream.extend(quote!(Some(#o))),
            None => stream.extend(quote!(None)),
        }
    }
}

impl<T> Default for AsOption<T> {
    #[inline]
    fn default() -> Self {
        AsOption(None)
    }
}

// line 86
macro_rules! propagate_err {
    ($res:expr) => {{
        match $res {
            Ok(v) => v,
            Err(e) => return $crate::util::into_stream(e),
        }
    }};
}

// line 81
#[inline]
pub fn into_stream(e: Error) -> TokenStream {
    e.to_compile_error().into()
}

// line 275
pub fn append_line(desc: &mut AsOption<String>, mut line: String) {
    if line.starts_with(' ') {
        line.remove(0);
    }

    let desc = desc.0.get_or_insert_with(String::default);

    match line.rfind("\\$") {
        Some(i) => {
            desc.push_str(line[..i].trim_end());
            desc.push(' ');
        }
        None => {
            desc.push_str(&line);
            desc.push('\n');
        }
    }
}
