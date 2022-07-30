use darling::{ast::{Data, Fields}, FromDeriveInput, FromField, FromVariant};
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
    fn getter_fields_to_emit(&self) -> Vec<ReceiverField> {
        self.data
            .as_ref()
            .take_struct()
            .expect("FieldNames only supports named structs")
            .into_iter()
            .filter(|field| {
                !field.skip && !field.write_only
            } )
            .map(|x| (*x).clone())
            .collect()
    }
}

impl Receiver {
    fn to_tokens_struct(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let getter_enum_ident = format_ident!("{}Getter", ident);
        let value_enum_ident = format_ident!("{}Value", ident);

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let fields = self.getter_fields_to_emit();

        let types: Vec<_> = fields.iter().map(|field|
            strip_ref(field.ty.clone())
        ).collect();

        let method_names: Vec<_> = fields.clone().into_iter().map(|field| {
            format_ident!("make_{}", field.ident.unwrap())
        }).collect();

        let names: Vec<_> = fields.clone().into_iter().map(|field|
            field.ident.unwrap()
        ).collect();

        let names_string: Vec<String> = fields.into_iter().map(|field| {
            format!(".{}", field.ident.unwrap())
        }).collect();

        let vis = &self.vis;
        let inner_derives = &self.derive;

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Clone, Hash, PartialEq, Eq, Copy)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #getter_enum_ident {
                #(#names(<#types as RemoteGet>::GetterType),)*
                #[default]
                __None,
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics #getter_enum_ident {
                #(#vis fn #method_names<F>(&self, func: F) -> Self where F: Fn(<#types as RemoteGet>::GetterType) -> <#types as RemoteGet>::GetterType {
                    #getter_enum_ident::#names(func(<#types as RemoteGet>::GetterType::default()))
                })*
            }

            impl Getter for #getter_enum_ident {
                fn parse_getter(&self, s: &str) -> Option<Self> {
                    match &s[..] {
                        #(s if s.starts_with(#names_string) => {
                            return Some(#getter_enum_ident::#names(<#types as RemoteGet>::GetterType::default().parse_getter(&s[#names_string.len()..])?));
                        })*,
                        _ => {
                            return None;
                        }
                    };
                }
            }

            impl core::fmt::Display for #getter_enum_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#getter_enum_ident::#names(ref x) => {
                            write!(f, #names_string)?;
                            write!(f, "{}", x)
                        },)*
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }

            #[automatically_derived]
            #[allow(non_camel_case_types)]
            #[derive(#(#inner_derives),*)]
            #[derive(Copy, Clone)]
            #vis enum #value_enum_ident {
                #(#names(<#types as RemoteGet>::ValueType)),*
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics RemoteGet for #ident #ty_generics #where_clause {
                type ValueType = #value_enum_ident;
                type GetterType = #getter_enum_ident;

                fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
                    Ok(match x {
                        #(#getter_enum_ident::#names(x) => #value_enum_ident::#names(self.#names.get(x)?),)*
                        #getter_enum_ident::__None => { unimplemented!() }
                    })
                }

                fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
                    match x {
                        #(#getter_enum_ident::#names(x) => {
                            let (x, len) = <#types as RemoteGet>::hydrate(x, buf)?;
                            Ok((#value_enum_ident::#names(x), len))
                        },)*
                        #getter_enum_ident::__None => { unimplemented!() }
                    }
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics #value_enum_ident {
                #(fn #names(self) -> <#types as RemoteGet>::ValueType {
                    match self {
                        #value_enum_ident::#names(x) => x,
                        _ => unreachable!(),
                    }
                })*
            }

            impl #impl_generics Value for #value_enum_ident {
                fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
                    match self {
                        #(#value_enum_ident::#names(inner) => inner.dehydrate(x), )*
                        _ => unreachable!(),
                    }
                }

                fn as_float(&self) -> Option<f32> {
                    match self {
                        #(#value_enum_ident::#names(inner) => inner.as_float(), )*
                        _ => None,
                    }
                }

                fn parse_value<T: Sized>(self, x: &str) -> Option<T> {
                    match &x[..] {
                        #(s if s.starts_with(#names_string) => {
                            return match self {
                                #value_enum_ident::#names(x) => x.parse_value(&s[#names_string.len()..]),
                                _ => None
                            }
                        },)*
                        _ => {
                            return None;
                        }
                    };
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
        let getter_enum_ident = format_ident!("{}Getter", ident);
        let value_enum_ident = format_ident!("{}Value", ident);

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let unit_variants = self.unit_variants();
        let newtype_variants = self.newtype_variants();
        let newtype_types = self.newtype_types();

        let newtype_method_names: Vec<_> = newtype_variants.iter().map(|field| {
            format_ident!("make_{}", field)
        }).collect();
        let newtype_value_variants: Vec<_> = newtype_variants.iter().map(|field| {
            format_ident!("{}Value", field)
        }).collect();
        let vis = &self.vis;
        let inner_derives = &self.derive;

        let newtype_names_string: Vec<String> = newtype_variants.clone().into_iter().map(|field| {
            format!("::{}", field)
        }).collect();

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Clone, Hash, PartialEq, Eq, Copy)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #getter_enum_ident {
                GetVariant,
                #(#newtype_variants(<#newtype_types as RemoteGet>::GetterType),)*
                #[default]
                __None,
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics #getter_enum_ident #ty_generics {
                #vis fn make_var<F>(&self, func: F) -> Self where F: Fn(()) -> NullGetter {
                    #getter_enum_ident::GetVariant
                }

                #(#vis fn #newtype_method_names<F>(&self, func: F) -> Self where F: Fn(<#newtype_types as RemoteGet>::GetterType) -> <#newtype_types as RemoteGet>::GetterType {
                    #getter_enum_ident::#newtype_variants(func(<#newtype_types as RemoteGet>::GetterType::default()))
                })*
            }

            impl Getter for #getter_enum_ident {
                fn parse_getter(&self, s: &str) -> Option<Self> {
                    match &s[..] {
                        "VARIANT" => return Some(#getter_enum_ident::GetVariant),
                        #(s if s.starts_with(#newtype_names_string) => {
                            return Some(#getter_enum_ident::#newtype_variants(<#newtype_types as RemoteGet>::GetterType::default().parse_getter(&s[#newtype_names_string.len()..])?));
                        })*,
                        _ => {
                            return None;
                        }
                    };
                }
            }

            impl core::fmt::Display for #getter_enum_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#getter_enum_ident::#newtype_variants(ref x) => {
                            write!(f, #newtype_names_string)?;
                            write!(f, "{}", x)?;
                        },)*
                        #getter_enum_ident::GetVariant => {
                            write!(f, "GetVariant")?;
                        },
                        _ => {
                            unreachable!();
                        }
                    }
                    Ok(())
                }
            }

            #[automatically_derived]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #[derive(Copy, Clone)]
            #vis enum #value_enum_ident {
                #(#newtype_value_variants(<#newtype_types as RemoteGet>::ValueType),)*
                #(#unit_variants,)*
                #(#newtype_variants,)*
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics RemoteGet for #ident #ty_generics #where_clause {
                type ValueType = #value_enum_ident #ty_generics;
                type GetterType = #getter_enum_ident #ty_generics;

                fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
                    Ok(match x {
                        #getter_enum_ident::GetVariant => {
                            match self {
                                #(#ident::#newtype_variants(_) => #value_enum_ident::#newtype_variants,)*
                                #(#ident::#unit_variants => #value_enum_ident::#unit_variants,)*
                            }
                        }
                        #(#getter_enum_ident::#newtype_variants(inner) => {
                            #value_enum_ident::#newtype_value_variants(match self {
                                Self::#newtype_variants(x) => x.get(inner)?,
                                _ => return Err(())
                            })
                        },)*
                        #getter_enum_ident::__None => { unimplemented!() }
                    })
                }

                fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
                    match x {
                        #(#getter_enum_ident::#newtype_variants(x) => {
                            let (x, len) = <#newtype_types as RemoteGet>::hydrate(x, buf)?;
                            Ok((#value_enum_ident::#newtype_value_variants(x), len))
                        },)*
                        _ => { unimplemented!() }
                    }
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics #value_enum_ident #ty_generics {
                #(fn #newtype_variants(self) -> <#newtype_types as RemoteGet>::ValueType {
                    match self {
                        Self::#newtype_value_variants(x) => x,
                        _ => unreachable!(),
                    }
                })*
            }

            impl #impl_generics Value for #value_enum_ident #ty_generics {
                fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
                    match self {
                        #(#value_enum_ident::#newtype_value_variants(inner) => inner.dehydrate(x), )*
                        _ => unreachable!(),
                    }
                }

                fn as_float(&self) -> Option<f32> {
                    match self {
                        #(#value_enum_ident::#newtype_value_variants(inner) => inner.as_float(), )*
                        _ => None,
                    }
                }

                fn parse_value<T: Sized>(self, x: &str) -> Option<T> {
                    match &x[..] {
                        #(s if s.starts_with(#newtype_names_string) => {
                            return match self {
                                #value_enum_ident::#newtype_value_variants(x) => x.parse_value(&s[#newtype_names_string.len()..]),
                                _ => None
                            }
                        },)*
                        _ => {
                            return None;
                        }
                    };
                }
            }
        })
    }
}

#[derive(FromField, Clone)]
#[darling(attributes(variant_names))]
struct ReceiverFieldVar {
    ty: Type
}


#[derive(FromVariant, Clone)]
#[darling(attributes(variant_names))]
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

