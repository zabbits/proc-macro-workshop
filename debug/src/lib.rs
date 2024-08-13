use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = parse_macro_input!(input as DeriveInput);
    TokenStream::new().into()
}
