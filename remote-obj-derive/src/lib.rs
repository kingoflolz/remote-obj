extern crate proc_macro;

use darling::FromDeriveInput;
use syn::{parse_macro_input, DeriveInput};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

mod setter;
mod getter;
mod helper;

#[proc_macro_derive(RemoteSetter, attributes(remote))]
pub fn derive_setter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setter::Receiver::from_derive_input(&parse_macro_input!(input as DeriveInput))
        .map(|receiver| quote!(#receiver))
        .unwrap_or_else(|err| err.write_errors())
        .into()
}

#[proc_macro_derive(RemoteGetter, attributes(remote))]
pub fn derive_getter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    getter::Receiver::from_derive_input(&parse_macro_input!(input as DeriveInput))
        .map(|receiver| quote!(#receiver))
        .unwrap_or_else(|err| err.write_errors())
        .into()
}

#[proc_macro]
pub fn setter(token: TokenStream) -> TokenStream {
    let v = parse_macro_input!(token as helper::Setter);
    v.to_token_stream().into()
}

#[proc_macro]
pub fn getter(token: TokenStream) -> TokenStream {
    let v = parse_macro_input!(token as helper::Getter);
    v.to_token_stream().into()
}

