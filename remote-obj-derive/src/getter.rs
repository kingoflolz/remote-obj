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

        let names: Vec<_> = fields.into_iter().map(|field|
            field.ident.unwrap()
        ).collect();

        let vis = &self.vis;
        let inner_derives = &self.derive;

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Clone, Copy)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #getter_enum_ident {
                #(#names(<#types as Getter>::GetterType),)*
                #[default]
                __None,
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics #getter_enum_ident {
                #vis #(fn #method_names<F>(&self, func: F) -> Self where F: Fn(<#types as Getter>::GetterType) -> <#types as Getter>::GetterType {
                    #getter_enum_ident::#names(func(<#types as Getter>::GetterType::default()))
                })*
            }

            #[automatically_derived]
            #[allow(non_camel_case_types)]
            #[derive(#(#inner_derives),*)]
            #vis enum #value_enum_ident {
                #(#names(<#types as Getter>::ValueType)),*
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics Getter for #ident #ty_generics #where_clause {
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
                            let (x, len) = <#types as Getter>::hydrate(x, buf)?;
                            Ok((#value_enum_ident::#names(x), len))
                        },)*
                        #getter_enum_ident::__None => { unimplemented!() }
                    }
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics #value_enum_ident {
                #(fn #names(self) -> <#types as Getter>::ValueType {
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

        tokens.extend(quote! {
            #[automatically_derived]
            #[derive(Default, Clone, Copy)]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #getter_enum_ident {
                GetVariant,
                #(#newtype_variants(<#newtype_types as Getter>::GetterType),)*
                #[default]
                __None,
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics #getter_enum_ident #ty_generics {
                #vis fn make_var<F>(&self, func: F) -> Self where F: Fn(()) -> () {
                    #getter_enum_ident::GetVariant
                }

                #(#vis fn #newtype_method_names<F>(&self, func: F) -> Self where F: Fn(<#newtype_types as Getter>::GetterType) -> <#newtype_types as Getter>::GetterType {
                    #getter_enum_ident::#newtype_variants(func(<#newtype_types as Getter>::GetterType::default()))
                })*
            }

            #[automatically_derived]
            #[derive(#(#inner_derives),*)]
            #[allow(non_camel_case_types)]
            #vis enum #value_enum_ident {
                #(#newtype_value_variants(<#newtype_types as Getter>::ValueType),)*
                #(#unit_variants,)*
                #(#newtype_variants,)*
            }

            #[automatically_derived]
            #[allow(non_snake_case)]
            impl #impl_generics Getter for #ident #ty_generics #where_clause {
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
                            let (x, len) = <#newtype_types as Getter>::hydrate(x, buf)?;
                            Ok((#value_enum_ident::#newtype_value_variants(x), len))
                        },)*
                        _ => { unimplemented!() }
                    }
                }
            }

            #[allow(non_snake_case)]
            impl #impl_generics #value_enum_ident #ty_generics {
                #(fn #newtype_variants(self) -> <#newtype_types as Getter>::ValueType {
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

