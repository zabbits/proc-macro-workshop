use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let bident = Ident::new(&format!("{}Builder", ident), Span::call_site());
    match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named: fields, .. }),
            ..
        }) => {
            // fields in `Builder` struct
            let builder_struct_fields = fields.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                quote! {
                    #ident: Option<#ty>
                }
            });

            // fields returned by `builder` function
            let builder_fn_fields = fields.iter().map(|f| {
                let ident = &f.ident;
                quote! {
                    #ident: None
                }
            });

            quote! {
                impl #ident {
                    pub fn builder() -> #bident {
                        #bident {
                            #(#builder_fn_fields,)*
                        }
                    }
                }

                pub struct #bident {
                    #(#builder_struct_fields,)*
                }
            }.into()
        }
        syn::Data::Struct(_) => unimplemented!(),
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}
