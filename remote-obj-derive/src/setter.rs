use darling::{ast::Data, FromDeriveInput, FromField, FromVariant};
use darling::ast::Fields;
use darling::util::PathList;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Ident, Type, Visibility};
use crate::helper::strip_ref;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named, enum_any), forward_attrs(derive), attributes(remote))]
pub(crate) struct Receiver {
    ident: Ident,
    generics: Generics,
    data: Data<ReceiverVariant, ReceiverField>,
    vis: Visibility,
    #[darling(default)]
    derive: PathList,
}

impl ToTokens for Receiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.data {
            Data::Enum(_) => {
                self.to_tokens_enum(tokens);
            }
            Data::Struct(_) => {
                self.to_tokens_struct(tokens);
            }
        }
    }
}

impl Receiver {
    fn setter_fields_to_emit(&self) -> Vec<ReceiverField> {
        self.data
            .as_ref()
            .take_struct()
            .expect("FieldNames only supports named structs")
            .into_iter()
            .filter(|field| {
                !field.skip && !field.read_only
            } )
            .map(|x| (*x).clone())
            .collect()
    }
}

impl Receiver {
    fn to_tokens_struct(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let setter_enum_ident = format_ident!("{}Setter", ident);

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let fields = self.setter_fields_to_emit();

        // panic!("{:?}, {:?}, {:?}", impl_generics, ty_generics, where_clause);

        let types: Vec<_> = fields.iter().map(|field|
            strip_ref(field.ty.clone())
        ).collect();

        let names: Vec<_> = fields.clone().into_iter().map(|field|
            field.ident.unwrap()
        ).collect();

        let method_names: Vec<_> = fields.clone().into_iter().map(|field| {
            format_ident!("make_{}", field.ident.unwrap())
        }).collect();
        let vis = &self.vis;
        let inner_derives = &self.derive;

        let names_string: Vec<String> = fields.clone().into_iter().map(|field| {
            format!(".{}", field.ident.unwrap())
        }).collect();

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Copy, Clone)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #setter_enum_ident {
                #(#names(<#types as RemoteSet>::SetterType),)*
                #[default]
                __None,
            }

            #[allow(non_snake_case)]
            impl #impl_generics #setter_enum_ident {
                #(#vis fn #method_names<F>(&self, func: F) -> Self where F: Fn(<#types as RemoteSet>::SetterType) -> <#types as RemoteSet>::SetterType {
                    #setter_enum_ident::#names(func(<#types as RemoteSet>::SetterType::default()))
                })*
            }

            impl Setter for #setter_enum_ident {
                fn parse_setter<T: Sized>(&self, x: &str, set: T) -> Option<Self> {
                    match &x[..] {
                        #(s if s.starts_with(#names_string) => {
                            return Some(#setter_enum_ident::#names(<#types as RemoteSet>::SetterType::default().parse_setter(&s[#names_string.len()..], set)?));
                        })*,
                        _ => {
                            return None;
                        }
                    };
                }

                fn parse_setter_numeric(&self, x: &str, set: f64) -> Option<Self> {
                    match &x[..] {
                        #(s if s.starts_with(#names_string) => {
                            return Some(#setter_enum_ident::#names(<#types as RemoteSet>::SetterType::default().parse_setter_numeric(&s[#names_string.len()..], set)?));
                        })*,
                        _ => {
                            return None;
                        }
                    };
                }
            }

            impl core::fmt::Display for #setter_enum_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#setter_enum_ident::#names(x) => {
                            write!(f, #names_string)?;
                            write!(f, "{}", x)?;
                        },)*
                        _ => {
                            unreachable!();
                        }
                    }
                    Ok(())
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics RemoteSet for #ident #ty_generics #where_clause {
                type SetterType = #setter_enum_ident;

                fn set(&mut self, x: Self::SetterType) -> Result<(), ()> {
                    match x {
                        #(#setter_enum_ident::#names(x) => self.#names.set(x),)*
                        #setter_enum_ident::__None => { unimplemented!() }
                    }
                }
            }
        })
    }
}

#[derive(FromField, Clone)]
#[darling(attributes(remote))]
#[allow(dead_code)]
struct ReceiverField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    write_only: bool,
    #[darling(default)]
    read_only: bool,
}

impl Receiver {
    fn unit_variants(&self) -> Vec<Ident> {
        self.data
            .as_ref()
            .take_enum()
            .expect("VariantNames only takes enums")
            .into_iter()
            .filter(|v| v.fields.is_unit() && !v.skip && !v.read_only)
            .map(|v| v.ident.clone())
            .collect()
    }

    fn newtype_variants(&self) -> Vec<Ident>{
        self.data
            .as_ref()
            .take_enum()
            .expect("VariantNames only takes enums")
            .into_iter()
            .filter(|v| v.fields.is_newtype() && !v.skip && !v.read_only)
            .map(|v| v.ident.clone())
            .collect()
    }

    fn newtype_types(&self) -> Vec<Type> {
        self.data
            .as_ref()
            .take_enum()
            .expect("VariantNames only takes enums")
            .into_iter()
            .filter(|v| v.fields.is_newtype())
            .map(|v| strip_ref(v.fields.fields.first().unwrap().ty.clone()))
            .collect()
    }

    fn other_varient_names(&self) -> Vec<Ident> {
        self.data
            .as_ref()
            .take_enum()
            .expect("VariantNames only takes enums")
            .into_iter()
            .filter(|v| !v.fields.is_unit() && !v.fields.is_newtype())
            .map(|v| v.ident.clone())
            .collect()
    }
}

impl Receiver {
    fn to_tokens_enum(&self, tokens: &mut TokenStream) {
        let other_varient_names = self.other_varient_names();
        if other_varient_names.len() > 0 {
            panic!("VariantNames only supports enums with no unit or newtype variants, {:?}", other_varient_names);
        }

        let ident = &self.ident;
        let setter_enum_ident = format_ident!("{}Setter", ident);

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let unit_variants = self.unit_variants();
        let unit_variant_method_names: Vec<_> = unit_variants.iter().map(|field| {
            format_ident!("make_{}", field)
        }).collect();

        let newtype_variants = self.newtype_variants();
        let newtype_variant_method_names: Vec<_> = newtype_variants.iter().map(|field| {
            format_ident!("make_{}", field)
        }).collect();

        let newtype_variants_names_string: Vec<String> = newtype_variants.clone().iter().map(|field| {
            format!("::{}", field)
        }).collect();

        let unit_variants_names_string: Vec<String> = unit_variants.clone().iter().map(|field| {
            format!("::{}", field)
        }).collect();

        let newtype_types = self.newtype_types();

        let vis = &self.vis;
        let inner_derives = &self.derive;

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Copy, Clone)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #setter_enum_ident #ty_generics {
                #(#unit_variants,)*
                #(#newtype_variants(<#newtype_types as RemoteSet>::SetterType),)*
                #[default]
                __None,
            }

            #[allow(non_snake_case)]
            impl #impl_generics #setter_enum_ident #ty_generics {
                #(#vis fn #unit_variant_method_names<F>(&self, func: F) -> Self where F: Fn(()) -> () {
                    #setter_enum_ident::#unit_variants
                })*

                #(#vis fn #newtype_variant_method_names<F>(&self, func: F) -> Self
                    where F: Fn(<#newtype_types as RemoteSet>::SetterType) -> <#newtype_types as RemoteSet>::SetterType {
                        #setter_enum_ident::#newtype_variants(func(<#newtype_types as RemoteSet>::SetterType::default()))
                })*
            }

            impl Setter for #setter_enum_ident {
                fn parse_setter<T: Sized>(&self, x: &str, set: T) -> Option<Self> {
                    match &x[..] {
                        #(s if s.starts_with(#newtype_variants_names_string) => {
                            return Some(#setter_enum_ident::#newtype_variants(<#newtype_types as RemoteSet>::SetterType::default().parse_setter(&s[#newtype_variants_names_string.len()..], set)?));
                        },)*
                        #(#unit_variants_names_string => {
                            assert_eq!(core::mem::size_of::<T>(), 0);
                            return Some(#setter_enum_ident::#unit_variants);
                        },)*
                        _ => {
                            return None;
                        }
                    };
                }

                fn parse_setter_numeric(&self, x: &str, set: f64) -> Option<Self> {
                    match &x[..] {
                        #(s if s.starts_with(#newtype_variants_names_string) => {
                            return Some(#setter_enum_ident::#newtype_variants(<#newtype_types as RemoteSet>::SetterType::default().parse_setter_numeric(&s[#newtype_variants_names_string.len()..], set)?));
                        },)*
                        _ => {
                            return None;
                        }
                    };
                }
            }

            impl core::fmt::Display for #setter_enum_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#setter_enum_ident::#newtype_variants(ref x) => {
                            write!(f, #newtype_variants_names_string)?;
                            write!(f, "{}", x)?;
                        },)*
                        #(#setter_enum_ident::#unit_variants => {
                            write!(f, " = ")?;
                            write!(f, #unit_variants_names_string)?;
                        },)*
                        _ => {
                            unreachable!();
                        }
                    }
                    Ok(())
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics RemoteSet for #ident #ty_generics #where_clause {
                type SetterType = #setter_enum_ident #ty_generics;

                fn set(&mut self, x: Self::SetterType)  -> Result<(), ()>{
                    match x {
                        #(#setter_enum_ident::#unit_variants =>
                            {
                                *self = #ident::#unit_variants;
                                return Ok(())
                            }
                        )*
                        #(#setter_enum_ident::#newtype_variants(setter) =>
                            match self {
                                #ident::#newtype_variants(ref mut inner) => {
                                    return inner.set(setter)
                                },
                                _ => {
                                    return Err(())
                                }
                            }
                        )*
                        #setter_enum_ident::__None => { unimplemented!() }
                    };
                }
            }
        })
    }
}

#[derive(FromField, Clone)]
#[darling(attributes(remote))]
struct ReceiverFieldVar {
    ty: Type
}


#[derive(FromVariant, Clone)]
#[darling(attributes(remote))]
#[allow(dead_code)]
struct ReceiverVariant {
    ident: Ident,
    fields: Fields<ReceiverFieldVar>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    write_only: bool,
    #[darling(default)]
    read_only: bool,
}
