extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::DeriveInput;

#[proc_macro_derive(FromCSV)]
pub fn from_csv_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    impl_from_csv(&ast)
}

fn impl_from_csv(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
}
