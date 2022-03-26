#[allow(unused_imports)]
use proc_macro2::Span;
#[allow(unused_imports)]
use proc_macro2::TokenStream as TokenStream2;
#[allow(unused_imports)]
use quote::{quote, ToTokens};
#[allow(unused_imports)]
use syn::{
    braced,
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Block, Expr, ExprClosure, FnArg, Ident, Pat, ReturnType, Stmt, Token, Type,
    Visibility,
};

use crate::consts::CHECK;
use crate::util::{rename_attributes, Argument, AsOption, IdentExt2, Parenthesised};

// check, is this a other attribute
fn is_cooked(attr: &Attribute) -> bool {
    const COOKED_ATTRIBUTE_NAMES: &[&str] = &[
        "cfg", "cfg_attr", "derive", "inline", "allow", "warn", "deny", "forbid",
    ];

    COOKED_ATTRIBUTE_NAMES.iter().any(|n| attr.path.is_ident(n))
}

// maybe pase argument and generate Argument Obj
fn parse_argument(arg: FnArg) -> Result<Argument> {
    match arg {
        // typed argument,
        // `#muutable #name: #kind`
        FnArg::Typed(typed) => {
            let pat = typed.pat;
            let kind = typed.ty;

            // pat == pattern
            match *pat {
                // not binding -> not match this arm
                Pat::Ident(id) => {
                    let name = id.ident;
                    let mutable = id.mutability;

                    Ok(Argument {
                        mutable,
                        name,
                        kind: *kind, // maybe remove ref?
                    })
                }
                // this arm mean parse _ variable.
                Pat::Wild(wild) => {
                    // ah, maybe mean, parsed "_" token?
                    // also we need only span info??? lmao
                    let token = wild.underscore_token;

                    // maybe, Wild not have name(because match only "_" case).
                    // so Argument required "name", must create this
                    let name = Ident::new("_", token.spans[0]);

                    Ok(Argument {
                        mutable: None, // not use -> shoud not/ must not mutable
                        name,
                        kind: *kind, // ^ see before match arm.
                    })
                }
                // this mean any other not binding pattern.
                // e.g. `: u32`(type pattern)
                _ => Err(Error::new(
                    pat.span(),
                    format_args!("unsupported pattern: {:?}", pat),
                )),
            }
        }
        // This arm match only untyped self
        // exclude `self: Box<Self>` pattern, match ^ arm
        FnArg::Receiver(_) => {
            // prohibited -> must not use / illegal
            // dont use self argument,,,?
            // maybe can not use command proc macro inner impl
            // TODO: research doc
            Err(Error::new(
                arg.span(),
                format_args!("`self` argument are prohibited: {:?}", arg),
            ))
        }
    }
}

// separate other attribute
fn remove_cooked(attrs: &mut Vec<Attribute>) -> Vec<Attribute> {
    let mut cooked = Vec::new();

    let mut i = 0;
    while i < attrs.len() {
        if !is_cooked(&attrs[i]) {
            i += 1;
            continue;
        }

        cooked.push(attrs.remove(i));
    }

    cooked
}

#[derive(Debug)]
pub struct CommandFun {
    // `#[...]` style attributes
    pub attributes: Vec<Attribute>,
    // Populated cooked attributes.
    // These are attributes outside of the realm of this crate's procedural macros
    // and will appear in generated output.
    // ^ from doc string. also mean not do anything, and put on generate
    pub cooked: Vec<Attribute>,
    // pub keyword
    pub visibility: Visibility,
    // function name
    pub name: Ident,
    // argument vector.
    // see imple Parse fn parse.
    pub args: Vec<Argument>,
    // mean return type. checked not Unit
    pub ret: Type,
    // function's body. `fn name() {/* this */}`
    pub body: Vec<Stmt>,
}

#[allow(unused_mut)]
#[allow(unused_variables)]
impl Parse for CommandFun {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // parse attribute.
        // inculude before fn keyword, [#hoge]'s vector
        let mut attributes = input.call(Attribute::parse_outer)?;

        //TODO: remove
        dbg!(&attributes);

        // doc rename to description
        rename_attributes(&mut attributes, "doc", "description");

        // separate outside attributes from attributes
        let cooked = remove_cooked(&mut attributes);
        // get visibility. maybe pub keyword
        let visibility = input.parse::<Visibility>()?;

        //check async
        input.parse::<Token![async]>()?;
        // check func
        input.parse::<Token![fn]>()?;
        // get func name
        let name = input.parse::<Ident>()?;

        // (...)
        // pase arguments
        let Parenthesised(args) = input.parse::<Parenthesised<FnArg>>()?;

        /*
           This mean Return Type must be Command or Check Result.
           maybe check not return unit("()")?
        */
        let ret = match input.parse::<ReturnType>()? {
            ReturnType::Type(_, t) => (*t).clone(),
            ReturnType::Default => {
                return Err(input
                    .error("excepeted a result type of either `CommandResult` or `CheckResult`"))
            }
        };

        // { ... }
        // maybe pasing func's main block
        let bcont;
        braced!(bcont in input);
        let body = bcont.call(Block::parse_within)?;

        // create argument Argument vector
        // Argument have {mutable: Option<Mut>, name: Ident, lind: Type} attribute.
        let args = args
            .into_iter()
            .map(parse_argument)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            attributes,
            cooked,
            visibility,
            name,
            args,
            ret,
            body,
        })
    }
}

// Ident vector, has any check???
// TODO
#[derive(Debug, Default)]
pub struct Checks(pub Vec<Ident>);

impl ToTokens for Checks {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let v = self.0.iter().map(|i| i.with_suffix(CHECK));

        // TODO: ???
        stream.extend(quote!(&[#(&#v),*]));
    }
}

#[derive(Debug, Default)]
pub struct Permissions(pub u64);

impl Permissions {
    #[allow(unreachable_code)] //TODO
    pub fn from_str(s: &str) -> Option<Self> {
        Some(Permissions(match s.to_uppercase().as_str() {
            //            "PRESET_GENERAL" => 0b0000_0110_0011_0111_1101_1100_0100_0001,
            _ => return None,
        }))
    }
}

impl ToTokens for Permissions {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let bits = self.0;

        // TODO: replace it
        //        let path = quote!(crate::);
        let path = quote!(serenity::model::permissions::Permissions);

        stream.extend(quote! {
            #path { bits: #bits }
        });
    }
}

#[derive(Debug, PartialEq)]
pub enum OnlyIn {
    Global,
    Guild(u64),
    None,
}

impl OnlyIn {
    #[inline]
    pub fn from_str(s: &str, span: Span) -> Result<Self> {
        if s.len() < 6 {
            return Err(Error::new(span, "invailid restriction type"));
        }
        match &s[0..5] {
            "global" => Ok(OnlyIn::Global),
            "guild(" => {
                if !s.ends_with(")") {
                    return Err(Error::new(span, "invailid restriction type (must end `)`)"));
                };
                let i = s.strip_prefix("guild(").unwrap().strip_suffix(")").unwrap();
                Ok(OnlyIn::Guild(match i.parse::<u64>() {
                    Ok(i) => i,
                    _ => return Err(Error::new(span, "invailid restriction type (must u64)")),
                }))
            }
            _ => return Err(Error::new(span, "invailid restriction type")),
        }
    }
}

impl ToTokens for OnlyIn {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        // TODO: replace it
        //        let path = quote!(crate::);
        let only_in_path = quote!(serenity::framework::standard::OnlyIn);
        match self {
            OnlyIn::Global => stream.extend(quote!(#only_in_path::Global)),
            OnlyIn::Guild(i) => stream.extend(quote!(#only_in_path::Guild(#i))),
            OnlyIn::None => stream.extend(quote!(#only_in_path::None)),
        }
    }
}

impl Default for OnlyIn {
    #[inline]
    fn default() -> Self {
        OnlyIn::None
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub checks: Checks,
    pub bucket: AsOption<String>,
    pub aliases: Vec<String>,
    pub description: AsOption<String>,
    pub delimiters: Vec<String>,
    pub usage: AsOption<String>,
    pub examples: Vec<String>,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub only_in: Vec<OnlyIn>,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub sub_commands: Vec<Ident>,
}

impl Options {
    #[inline]
    pub fn new() -> Self {
        Self {
            help_available: true,
            ..Default::default()
        }
    }
}
