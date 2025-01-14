use std::any::Any;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{self, Expr, ExprAssign, GenericParam, parse_quote};
use syn::{DataEnum, DataUnion, DeriveInput};

#[proc_macro_derive(ToCSVMacro, attributes(csv))]
pub fn to_csv_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    impl_to_csv(&ast).into()
}

fn impl_to_csv(ast: &DeriveInput) -> TokenStream {
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
    let mut header = quote! {
        let mut inner = Vec::new();
    };
    let mut record = quote! {
        let mut inner = Vec::new();
    };
    match data {
        syn::Data::Struct(s) => {
            for field in s.fields.iter() {
                if field.ident.is_none() {
                } else {
                    let mut csv_field_name = None;
                    for attr in &field.attrs {
                        if attr.path().is_ident("csv") {
                            match attr.parse_args() {
                                Err(_) => {}
                                Ok(attr) => match attr {
                                    Expr::Assign(expr) => {
                                        csv_field_name = Some(expr.right);
                                    }
                                    _ => {}
                                },
                            }
                        }
                    }
                    if let Some(field_name) = csv_field_name {
                        header.extend(quote! {
                            inner.push(#field_name.to_string());
                        });

                        let field_ident = field.ident.as_ref().unwrap();
                        record.extend(quote! {
                            inner.push(format!("{}", self.#field_ident));
                        });
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

    let output = quote! {
        impl #impl_generics ::csv::ToCSV for #ident #ty_generics #where_clause{
            fn to_header(&self) -> Vec<String>{
                #header
                inner
            }

            fn to_record(&self) -> Vec<String>{
                #record
                inner
            }
        }
    };

    output.into()
}
