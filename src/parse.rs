use {
    crate::{grammar, ParseResult},
    quote::quote,
    syn::{
        braced,
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Attribute, Ident, Item, Token, Visibility,
    },
};

pub(crate) struct PragmaInput {
    pub(crate) items: Punctuated<PragmaItem, Token![;]>,
}

impl Parse for PragmaInput {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut items = Punctuated::new();
        while !input.is_empty() {
            let itm = input.parse::<PragmaItem>()?;
            items.push(itm);
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(PragmaInput { items })
    }
}

pub(crate) enum PragmaItemContent {
    Normal(Item),
    Mod { ident: Ident, content: PragmaInput },
}

pub(crate) struct PragmaItem {
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) visibility: Visibility,
    pub(crate) condition: Option<grammar::ConditionExpr>,
    pub(crate) content: PragmaItemContent,
}

impl Parse for PragmaItem {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // parse attributes
        let attrs = input.call(syn::Attribute::parse_outer)?;
        // parse visibility
        let visibility: Visibility = input.parse()?;

        // check if we have `(if ...)`
        let condition = if input.peek(syn::token::Paren) {
            let content;
            let _paren = syn::parenthesized!(content in input);
            content.parse::<Token![if]>()?;
            let cond_expr = grammar::parse_condition(&&content)?;
            Some(cond_expr)
        } else {
            None
        };

        if input.peek(Token![mod]) {
            // parse a module
            input.parse::<Token![mod]>()?;
            let ident: Ident = input.parse()?;
            let content_stream;
            let _brace = braced!(content_stream in input);

            let mut items = Punctuated::new();
            while !content_stream.is_empty() {
                let itm = content_stream.parse::<PragmaItem>()?;
                items.push(itm);
                if content_stream.peek(Token![;]) {
                    content_stream.parse::<Token![;]>()?;
                }
            }

            let inner_input = PragmaInput { items };
            Ok(PragmaItem {
                attrs,
                visibility,
                condition,
                content: PragmaItemContent::Mod {
                    ident,
                    content: inner_input,
                },
            })
        } else {
            // normal item
            let item: Item = input.parse()?;
            Ok(PragmaItem {
                attrs,
                visibility,
                condition,
                content: PragmaItemContent::Normal(item),
            })
        }
    }
}

pub(crate) fn process_pragma_input(input: PragmaInput) -> proc_macro2::TokenStream {
    let tokens = input.items.into_iter().map(|item| {
        let PragmaItem {
            attrs,
            visibility,
            condition,
            content,
        } = item;

        match content {
            PragmaItemContent::Normal(item) => {
                if let Some(cond) = condition {
                    let main_condition = grammar::condition_to_cfg(&cond);
                    let inverse_condition = quote! { not(#main_condition) };

                    match &visibility {
                        Visibility::Inherited => {
                            // single version for (if condition) no visibility
                            quote! {
                                #[cfg(#main_condition)]
                                #(#attrs)*
                                #item
                            }
                        }
                        _ => {
                            // two versions for pub (if condition)
                            let public_item = quote! {
                                #[cfg(#main_condition)]
                                #(#attrs)*
                                #visibility #item
                            };
                            let private_item = quote! {
                                #[cfg(#inverse_condition)]
                                #(#attrs)*
                                #item
                            };
                            quote! {
                                #public_item
                                #private_item
                            }
                        }
                    }
                } else {
                    // unconditional item
                    quote! {
                        #(#attrs)*
                        #visibility #item
                    }
                }
            }
            PragmaItemContent::Mod {
                ident,
                content: inner_input,
            } => {
                let inner_tokens = process_pragma_input(inner_input);
                if let Some(cond) = condition {
                    let main_condition = grammar::condition_to_cfg(&cond);
                    let inverse_condition = quote! { not(#main_condition) };

                    match &visibility {
                        Visibility::Inherited => {
                            quote! {
                                #[cfg(#main_condition)]
                                #(#attrs)*
                                mod #ident {
                                    #inner_tokens
                                }
                            }
                        }
                        _ => {
                            let public_item = quote! {
                                #[cfg(#main_condition)]
                                #(#attrs)*
                                #visibility mod #ident {
                                    #inner_tokens
                                }
                            };
                            let private_item = quote! {
                                #[cfg(#inverse_condition)]
                                #(#attrs)*
                                mod #ident {
                                    #inner_tokens
                                }
                            };
                            quote! {
                                #public_item
                                #private_item
                            }
                        }
                    }
                } else {
                    // unconditional mod
                    quote! {
                        #(#attrs)*
                        #visibility mod #ident {
                            #inner_tokens
                        }
                    }
                }
            }
        }
    });

    quote! {
        #(#tokens)*
    }
}
