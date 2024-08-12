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

            // builder functions
            let builder_fn = fields.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                quote! {
                    fn #ident(&mut self, #ident: #ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            });

            // build function
            let build_fn_fields = fields.iter().map(|f| {
                let ident = &f.ident;
                let err_msg = format!("{} is not set", ident.as_ref().unwrap());
                quote! {
                    #ident: self.#ident.clone().ok_or(#err_msg)?
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

                impl #bident {
                    pub fn build(&mut self) -> Result<#ident, Box<dyn std::error::Error>> {
                        Ok(#ident { #(#build_fn_fields,)* })
                    }

                    #(#builder_fn)*
                }
            }.into()
        }
        syn::Data::Struct(_) => unimplemented!(),
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}
