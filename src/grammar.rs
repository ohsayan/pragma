use {
    super::ParseResult,
    quote::quote,
    syn::{parse::ParseStream, Ident, LitStr, Token},
};

/// Condition expression AST
pub(crate) enum ConditionExpr {
    All(Vec<ConditionExpr>),
    Any(Vec<ConditionExpr>),
    Not(Box<ConditionExpr>),
    KeyVal(Ident, LitStr),
    Key(Ident),
}

/// parse condition expressions
///
/// Grammar:
/// ```text
/// Condition := OrExpr
/// OrExpr    := AndExpr ('or' AndExpr)*
/// AndExpr   := Primary ('and' Primary)*
/// Primary   := KeyVal | Key | Paren | NotExpr
///
/// KeyVal    := Ident '=' LitStr
/// Key       := Ident
/// Paren     := '(' Condition ')'
/// NotExpr   := 'not' '(' Condition ')'
/// ```
pub(crate) fn parse_condition(input: &ParseStream) -> ParseResult<ConditionExpr> {
    parse_or_expr(input)
}

pub(crate) fn parse_or_expr(input: &ParseStream) -> ParseResult<ConditionExpr> {
    let mut expr = parse_and_expr(input)?;
    loop {
        // look ahead to see if the next ident is "or"
        if input.peek(Ident) {
            let ident_peek = input.fork().parse::<Ident>()?;
            if ident_peek == "or" {
                // consume `or` and parse the next AndExpr
                input.parse::<Ident>()?; // actually consume "or"
                let rhs = parse_and_expr(input)?;
                expr = match expr {
                    ConditionExpr::Any(mut v) => {
                        v.push(rhs);
                        ConditionExpr::Any(v)
                    }
                    _ => ConditionExpr::Any(vec![expr, rhs]),
                };
            } else {
                // not "or", so we're done with OrExpr parsing
                break;
            }
        } else {
            break;
        }
    }
    Ok(expr)
}

pub(crate) fn parse_and_expr(input: &ParseStream) -> ParseResult<ConditionExpr> {
    let mut expr = parse_primary(input)?;
    loop {
        // look ahead to see if the next ident is "and"
        if input.peek(Ident) {
            let ident_peek = input.fork().parse::<Ident>()?;
            if ident_peek == "and" {
                // consume `and` and parse the next Primary
                input.parse::<Ident>()?; // consume "and"
                let rhs = parse_primary(input)?;
                expr = match expr {
                    ConditionExpr::All(mut v) => {
                        v.push(rhs);
                        ConditionExpr::All(v)
                    }
                    _ => ConditionExpr::All(vec![expr, rhs]),
                };
            } else {
                // not "and", so we're done with AndExpr parsing.
                // this could be "or" or something else that belongs to a higher level.
                break;
            }
        } else {
            break;
        }
    }
    Ok(expr)
}

pub(crate) fn parse_primary(input: &ParseStream) -> ParseResult<ConditionExpr> {
    if input.peek(Ident) {
        // check if it's `not(...)` or a key/key=val
        let ident: Ident = input.parse()?;
        if ident == "not" {
            // parse 'not(...)'
            let content;
            let _paren = syn::parenthesized!(content in input);
            let inner = parse_condition(&&content)?;
            return Ok(ConditionExpr::Not(Box::new(inner)));
        } else {
            // it's a key or key=val
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let val: LitStr = input.parse()?;
                return Ok(ConditionExpr::KeyVal(ident, val));
            } else {
                return Ok(ConditionExpr::Key(ident));
            }
        }
    }

    if input.peek(syn::token::Paren) {
        // parse '(...)'
        let content;
        let _paren = syn::parenthesized!(content in input);
        let inner = parse_condition(&&content)?;
        return Ok(inner);
    }

    Err(syn::Error::new(
        input.span(),
        "expected condition (key, key=val, not(...), or (...))",
    ))
}

pub(crate) fn condition_to_cfg(expr: &ConditionExpr) -> proc_macro2::TokenStream {
    match expr {
        ConditionExpr::All(exprs) => {
            let inner = exprs.iter().map(condition_to_cfg);
            quote! { all(#(#inner),*) }
        }
        ConditionExpr::Any(exprs) => {
            let inner = exprs.iter().map(condition_to_cfg);
            quote! { any(#(#inner),*) }
        }
        ConditionExpr::Not(e) => {
            let inner = condition_to_cfg(e);
            quote! { not(#inner) }
        }
        ConditionExpr::KeyVal(ident, val) => {
            quote! { #ident = #val }
        }
        ConditionExpr::Key(ident) => {
            quote! { #ident }
        }
    }
}
