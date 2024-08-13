use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    match input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => {
            let dbg_fields = fields.iter().map(|f| {
                let name = format!("{}", f.ident.as_ref().unwrap());
                let ident = &f.ident;
                quote! { field(#name, &self.#ident) }
            });
            quote! {
                impl std::fmt::Debug for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        f.debug_struct(stringify!(#ident))
                            #(.#dbg_fields)*
                            .finish()
                    }
                }
            }.into()
        }
        _ => unimplemented!(),
    }
}
