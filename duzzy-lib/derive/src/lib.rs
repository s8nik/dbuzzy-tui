extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DuzzyListImpl)]
pub fn duzzy_list(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl duzzy_lib::DuzzyList for #name {
            fn next(&mut self) {
                let len = self.length();
                let state = self.state();

                let i = state.selected().map(|i| {
                    if i >= len - 1 {
                        0
                    } else {
                        i + 1
                    }
                });

                state.select(i);
            }

            fn prev(&mut self) {
                let len = self.length();
                let state = self.state();

                let i = state.selected().map(|i| {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                });

                state.select(i);
            }
        }
    };

    TokenStream::from(expanded)
}
