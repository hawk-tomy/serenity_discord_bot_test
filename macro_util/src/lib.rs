use proc_macro::TokenStream;
#[allow(unused_imports)]
use proc_macro2::Span;
#[allow(unused_imports)]
use quote::quote;
#[allow(unused_imports)]
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Lit, Token,
};

pub(crate) mod attributes;
pub(crate) mod consts;
pub(crate) mod structures;

#[macro_use]
pub(crate) mod util;

use attributes::parse_values;
use structures::{CommandFun, Options};
use util::{IdentExt2, LitExt};

// define macro
macro_rules! match_options {
    ($v:expr, $values:ident, $options:ident, $span:expr => [$($name:ident);*]) => {
        match $v {
            $(
                stringify!($name) => $options.$name = propagate_err!($crate::attributes::parse($values)),
            )*
            _ => {
                return Error::new($span, format_args!("invalid attribute: {:?}", $v))
                    .to_compile_error()
                    .into();
            },
        }
    };
}

// define proc macro
#[proc_macro_attribute]
pub fn application_command(attr: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: remove
    dbg!(&input);
    dbg!(&attr);

    // input parse to Command Fun
    let mut fun = parse_macro_input!(input as CommandFun);

    // TODO: remove
    dbg!(&fun);

    // TODO: maybe set name?
    let _name = if !attr.is_empty() {
        parse_macro_input!(attr as Lit).to_str()
    } else {
        fun.name.to_string_non_raw()
    };

    let mut options = Options::new();

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = propagate_err!(parse_values(attribute));

        let name = values.name.to_string();
        let name = &name[..];

        match name {
            // num_args and example are not necessary,
            "description" => {
                let line: String = propagate_err!(attributes::parse(values));
                util::append_line(&mut options.description, line);
            }
            _ => {
                // min_args;
                // max_args;
                match_options!(name, values, options, span => [
                    checks;
                    bucket;
                    aliases;
                    delimiters;
                    usage;
                    required_permissions;
                    allowed_roles;
                    help_available;
                    only_in;
                    owners_only;
                    owner_privilege;
                    sub_commands
                ]);
            }
        }
    }

    let Options {
        checks,
        bucket,
        aliases,
        description,
        delimiters,
        usage,
        examples,
        allowed_roles,
        required_permissions,
        help_available,
        only_in,
        owners_only,
        owner_privilege,
        sub_commands,
    } = options;

    propagate_err!(create_declaration_validations());

    attr
}
