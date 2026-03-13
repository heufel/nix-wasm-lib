use std::str::FromStr;

use proc_macro::{Delimiter, Group, Ident, TokenStream, TokenTree};

macro_rules! force {
    ($f:expr, $emsg:literal $(,)?) => {
        match $f {
            Ok(ok) => ok,
            Err(_) => {
                return TokenStream::from_str(&format!(r#"compile_error!("{}");"#, $emsg)).unwrap()
            }
        }
    };
}

#[derive(Debug)]
enum ParseError {
    NonMatch,
    NoneLeft,
}

type It = std::iter::Peekable<proc_macro::token_stream::IntoIter>;

#[proc_macro_attribute]
pub fn export_nix(_args: TokenStream, input: TokenStream) -> TokenStream {
    let tt = &mut input.into_iter().peekable();
    force!(
        parse_lit(tt, "pub"),
        "A function must be public to be exported to Nix."
    );
    force!(parse_lit(tt, "fn"), "Expected fn");
    let function_name = force!(parse_ident(tt), "Expected function name.").to_string();
    let args_ts = tt.peek().ok_or(ParseError::NonMatch).unwrap().to_string();
    let args = force!(parse_args(tt), "Expected arguments.");
    force!(
        parse_punct(tt, "-"),
        "A function must return a value to be exported to Nix."
    );
    force!(parse_punct(tt, ">"), "Expected ->");
    let ret = force!(
        parse_until_group(tt),
        "A function must return a value to be exported to Nix."
    )
    .to_string();
    let body = force!(parse_function(tt), "Expected function body.").to_string();
    let value = "nix_types::compat::Value";
    let nix_value = "nix_types::NixValue";
    let argstr = args
        .iter()
        .map(|(arg, _)| format!("{}: {value}", arg.to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    let args_pass = args
        .iter()
        .map(|(arg, _)| format!("{nix_value}::from({}).into()", arg))
        .collect::<Vec<_>>()
        .join(", ");

    TokenStream::from_str(&format!(
        r#"
            #[unsafe(no_mangle)]
            pub extern "C" fn {function_name}({argstr}) -> {value} {{
                use {value} as _;
                use {nix_value} as _;
                fn _internal {args_ts} -> {ret} {body}
                {value}::from({nix_value}::from(_internal ({args_pass})))
            }}
        "#
    ))
    .expect("export_nix generated an invalid TokenStream. This is a bug.")
}

fn parse_ident(tt: &mut It) -> Result<Ident, ParseError> {
    let TokenTree::Ident(id) = tt.next().ok_or(ParseError::NoneLeft)? else {
        return Err(ParseError::NonMatch);
    };
    Ok(id)
}

fn parse_lit(tt: &mut It, lit: &str) -> Result<(), ParseError> {
    let id = parse_ident(tt)?;
    if id.to_string() == lit {
        Ok(())
    } else {
        Err(ParseError::NonMatch)
    }
}

fn parse_group(tt: &mut It) -> Result<Group, ParseError> {
    let TokenTree::Group(grp) = tt.next().ok_or(ParseError::NoneLeft)? else {
        return Err(ParseError::NonMatch);
    };
    Ok(grp)
}

fn parse_function(tt: &mut It) -> Result<Group, ParseError> {
    let grp = parse_group(tt)?;
    if grp.delimiter() == Delimiter::Brace {
        Ok(grp)
    } else {
        Err(ParseError::NonMatch)
    }
}

fn parse_punct(tt: &mut It, punct: &str) -> Result<(), ParseError> {
    let TokenTree::Punct(p) = tt.next().ok_or(ParseError::NoneLeft)? else {
        return Err(ParseError::NonMatch);
    };
    if p.to_string() == punct {
        Ok(())
    } else {
        Err(ParseError::NonMatch)
    }
}

fn check_punct(tt: &mut It, punct: &str) -> Result<bool, ParseError> {
    let Some(pk) = tt.peek() else {
        return Err(ParseError::NoneLeft);
    };
    let TokenTree::Punct(p) = pk else {
        return Ok(false);
    };
    if p.to_string() == punct {
        return Ok(true);
    } else {
        return Ok(false);
    }
}
fn check_group(tt: &mut It) -> Result<bool, ParseError> {
    let Some(pk) = tt.peek() else {
        return Err(ParseError::NoneLeft);
    };
    let TokenTree::Group(_) = pk else {
        return Ok(false);
    };
    return Ok(true);
}

fn parse_until_group(tt: &mut It) -> Result<TokenStream, ParseError> {
    let mut ty = TokenStream::new();
    while let Ok(false) = check_group(tt) {
        ty.extend(Some(tt.next().ok_or(ParseError::NonMatch)?));
    }
    Ok(ty)
}

type Arg = (Ident, TokenStream);

fn parse_arg(tt: &mut It) -> Result<Arg, ParseError> {
    let name = parse_ident(tt)?;
    parse_punct(tt, ":")?;

    let mut ty = TokenStream::new();
    while let Ok(false) = check_punct(tt, ",") {
        ty.extend(Some(tt.next().ok_or(ParseError::NonMatch)?));
    }
    Ok((name, ty))
}

fn parse_args(tt: &mut It) -> Result<Vec<Arg>, ParseError> {
    let raw_args = &mut parse_group(tt)?.stream().into_iter().peekable();
    let mut args: Vec<Arg> = Vec::new();
    while let Ok(next) = parse_arg(raw_args) {
        args.push(next);
        if let Ok(true) = check_punct(raw_args, ",") {
            parse_punct(raw_args, ",")?;
        } else if raw_args.next().is_none() {
            break;
        } else {
            return Err(ParseError::NonMatch);
        };
    }

    Ok(args)
}
