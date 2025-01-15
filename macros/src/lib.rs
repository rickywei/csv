use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Expr, GenericParam, parse_quote};
use syn::{DataEnum, DataUnion, DeriveInput};

enum DeriveType {
    Header,
    From,
    To,
}

#[proc_macro_derive(CSVHeader, attributes(csv))]
pub fn csv_header_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    impl_csv_derive(&ast, DeriveType::Header).into()
}

#[proc_macro_derive(CSVFrom, attributes(csv))]
pub fn csv_from_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    impl_csv_derive(&ast, DeriveType::From).into()
}

#[proc_macro_derive(CSVTo, attributes(csv))]
pub fn csv_to_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    impl_csv_derive(&ast, DeriveType::To).into()
}

fn impl_csv_derive(ast: &DeriveInput, dt: DeriveType) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(Display))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut fn_body = match dt {
        DeriveType::Header => quote! {
            let mut inner = Vec::new();
        },
        DeriveType::From => quote! {
            use std::collections::HashMap;
            let mut inner = Self::default();
            let mut m = HashMap::new();
            for (k, v) in header.iter().zip(record.iter()) {
                m.insert(k.clone(), v);
            }
        },
        DeriveType::To => {
            quote! {
                let mut inner = Vec::new();
            }
        }
    };

    match data {
        syn::Data::Struct(s) => {
            for field in s.fields.iter() {
                if field.ident.is_none() {
                } else {
                    let ident = field.ident.as_ref().unwrap();
                    for attr in &field.attrs {
                        if attr.path().is_ident("csv") {
                            match attr.parse_args() {
                                Err(_) => {}
                                Ok(attr) => match attr {
                                    Expr::Assign(expr) => {
                                        if let Expr::Path(path) = *expr.left {
                                            if path.path.is_ident("field") {
                                                let right = expr.right;
                                                match dt{
                                                    DeriveType::Header=>fn_body.extend(quote! {
                                                        inner.push(#right.to_string());
                                                    }),
                                                    DeriveType::From=>fn_body.extend(quote! {
                                                        match m.get(#right) {
                                                            Some(v) => {
                                                                inner.#ident = v.parse()?;
                                                            },
                                                            None => {return Err(ErrorKind::ErrMissField(#right.to_string()).into());},
                                                        }
                                                    }),
                                                    DeriveType::To=>fn_body.extend(quote! {
                                                        inner.push(self.#ident.to_string());
                                                    })
                                                }
                                            }
                                        }
                                    }
                                    Expr::Path(expr) => {
                                        if expr.path.is_ident("flatten") {
                                            let typ = field.ty.clone();
                                            match dt {
                                                DeriveType::Header => fn_body.extend(quote! {
                                                    let tmp = #typ::get_header();
                                                    inner.extend(tmp);
                                                }),
                                                DeriveType::From => fn_body.extend(quote! {
                                                    inner.#ident = #typ::from_csv(header, record)?;
                                                }),
                                                DeriveType::To => fn_body.extend(quote! {
                                                    let tmp = self.#ident.to_csv();
                                                    inner.extend(tmp);
                                                }),
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                            }
                        }
                    }
                }
            }
        }
        syn::Data::Enum(DataEnum { variants, .. }) => {
            return syn::Error::new_spanned(variants, "enum is not supported")
                .to_compile_error()
                .into();
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    match dt {
        DeriveType::Header => quote! {
            impl #impl_generics ::csv::HeaderCSV for #ident #ty_generics #where_clause{
                fn get_header() -> Vec<String>{
                    #fn_body
                    inner
                }
            }
        },
        DeriveType::From => quote! {
            impl #impl_generics ::csv::FromCSV for #ident #ty_generics #where_clause{
                fn from_csv(header: &Vec<String>, record: &Vec<String>) -> Result<Self>{
                    #fn_body
                    Ok(inner)
                }
            }
        },
        DeriveType::To => quote! {
            impl #impl_generics ::csv::ToCSV for #ident #ty_generics #where_clause{
                fn to_csv(&self) -> Vec<String>{
                    #fn_body
                    inner
                }
            }
        },
    }
    .into()
}
