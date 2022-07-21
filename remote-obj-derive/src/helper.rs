use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, token, bracketed};

extern crate proc_macro2;

#[derive(Debug)]
enum IdentOrIndex {
    Field(Ident),
    Variant(Ident),
    Index(Expr),
}

pub(crate) struct Setter {
    path: Vec<IdentOrIndex>,
    base_type: Ident,
    expr: Option<Expr>
}

impl Parse for Setter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base_type = input.parse::<Ident>()?;
        let mut path = Vec::new();
        let mut expr = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Token![.]) {
                input.parse::<syn::Token![.]>()?;
                path.push(IdentOrIndex::Field(input.parse::<Ident>()?));
            } else if lookahead.peek(syn::Token![::]) {
                input.parse::<syn::Token![::]>()?;
                path.push(IdentOrIndex::Variant(input.parse::<Ident>()?));
            } else if lookahead.peek(token::Bracket) {
                let content;
                bracketed!(content in input);
                path.push(IdentOrIndex::Index(content.parse::<Expr>()?));
            } else if lookahead.peek(syn::Token![=]) {
                input.parse::<syn::Token![=]>()?;
                expr = Some(input.parse::<Expr>()?);
                break;
            } else {
                return Err(lookahead.error())
            }
        }

        if expr.is_none() {
            match path.last().unwrap() {
                IdentOrIndex::Variant(_) => {},
                _ => return Err(input.error("expected `=`"))
            }
        }

        Ok(Setter {
            path,
            base_type,
            expr
        })
    }
}

impl ToTokens for Setter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let base_setter_type = format_ident!("{}Setter", self.base_type);
        let expr = self.expr.clone();

        let mut partial;
        match expr {
            None => {
                partial = quote!{()};
            }
            Some(expr) => {
                partial = quote!{#expr};
            }
        }

        for i in self.path.iter().rev() {
            match i {
                IdentOrIndex::Field(i) | IdentOrIndex::Variant(i) => {
                    let i = format_ident!("make_{}", i);
                    partial = quote! {
                        x.#i(|x| #partial)
                    };
                }
                IdentOrIndex::Index(i) => {
                    partial = quote! {
                        x.arr_set(#i, |x| #partial)
                    };
                }
            }

        }

        tokens.extend(quote! {
            {
                let x = #base_setter_type::default();
                #partial
            }
        })
    }
}

pub(crate) struct Getter {
    path: Vec<IdentOrIndex>,
    base_type: Ident,
}

impl Parse for Getter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base_type = input.parse::<Ident>()?;
        let mut path = Vec::new();

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Token![.]) {
                input.parse::<syn::Token![.]>()?;
                path.push(IdentOrIndex::Field(input.parse::<Ident>()?));
            } else if lookahead.peek(syn::Token![::]) {
                input.parse::<syn::Token![::]>()?;
                path.push(IdentOrIndex::Variant(input.parse::<Ident>()?));
            } else if lookahead.peek(token::Bracket) {
                let content;
                bracketed!(content in input);
                path.push(IdentOrIndex::Index(content.parse::<Expr>()?));
            } else {
                return Err(lookahead.error())
            }
        }

        Ok(Getter {
            path,
            base_type,
        })
    }
}

impl ToTokens for Getter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let base_getter_type = format_ident!("{}Getter", self.base_type);

        let mut partial = quote!{()};
        for i in self.path.iter().rev() {
            match i {
                IdentOrIndex::Field(i) | IdentOrIndex::Variant(i) => {
                    let i = format_ident!("make_{}", i);
                    partial = quote! {
                        x.#i(|x| #partial)
                    };
                }
                IdentOrIndex::Index(i) => {
                    partial = quote! {
                        x.arr_get(#i, |x| #partial)
                    };
                }
            }

        }

        tokens.extend(quote! {
            {
                let x = #base_getter_type::default();
                #partial
            }
        })
    }
}