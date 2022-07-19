use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprMethodCall, ExprField, Member, ExprPath};

extern crate proc_macro2;

pub(crate) struct Setter {
    path: Vec<Ident>,
    base_type: Ident,
    expr: Expr
}

impl Parse for Setter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let assign = input.parse::<ExprMethodCall>()?;

        let mut path = Vec::new();
        let expr;

        let ExprMethodCall{receiver, method, args, ..} = assign;
        path.push(method.clone());

        let mut partial = receiver.clone();
        loop {
            match &*partial {
                Expr::Field(ExprField{base, member: Member::Named(ident), ..}) => {
                    path.push(ident.clone());

                    partial = base.clone();
                }
                Expr::Path(ExprPath{path: p, ..}) => {
                    path.extend(p.segments.iter().rev().map(|s| s.ident.clone()));
                    break
                }
                _ => break
            }
        }
        assert_eq!(args.len(), 1);

        expr = args.first().unwrap().clone();

        let base_type = path.pop().unwrap();
        path.reverse();

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

        let mut partial = quote!{#expr};
        for i in self.path.iter().rev() {
            let i = format_ident!("make_{}", i);
            partial = quote! {
                x.#i(|x| #partial)
            };
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
    path: Vec<Ident>,
    base_type: Ident,
}

impl Parse for Getter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let assign = input.parse::<ExprMethodCall>()?;

        let mut path = Vec::new();

        let ExprMethodCall{receiver, method, ..} = assign;
        path.push(method.clone());

        let mut partial = receiver.clone();
        loop {
            match &*partial {
                Expr::Field(ExprField{base, member: Member::Named(ident), ..}) => {
                    path.push(ident.clone());

                    partial = base.clone();
                }
                Expr::Path(ExprPath{path: p, ..}) => {
                    path.extend(p.segments.iter().rev().map(|s| s.ident.clone()));
                    break
                }
                _ => break
            }
        }

        let base_type = path.pop().unwrap();
        path.reverse();

        Ok(Getter {
            path,
            base_type,
        })
    }
}

impl ToTokens for Getter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let base_setter_type = format_ident!("{}Getter", self.base_type);

        let mut partial = quote!{ () };
        for i in self.path.iter().rev() {
            let i = format_ident!("make_{}", i);
            partial = quote! {
                x.#i(|x| #partial)
            };
        }

        tokens.extend(quote! {
            {
                let x = #base_setter_type::default();
                #partial
            }
        })
    }
}
