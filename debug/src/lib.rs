use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Expr, ExprLit, Field, GenericParam, Generics, Lit, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as DeriveInput);
    add_traits_bound(&mut input.generics);
    // println!("{:#?}", input);
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let dbg_fields = fields.iter().map(|f| {
                let name = format!("{}", f.ident.as_ref().unwrap());
                let ident = &f.ident;
                let dbg_attr = get_debug_attr(f);
                if let Some(fmt) = dbg_attr {
                    quote! { field(#name, &format_args!(#fmt, &self.#ident)) }
                } else {
                    quote! { field(#name, &self.#ident) }
                }
            });
            quote! {
                impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
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

fn get_debug_attr(f: &Field) -> Option<String> {
    f.attrs.iter().find_map(|attr| {
        // meta must be a NamedValue
        if let Meta::NameValue(nv) = &attr.meta {
            if !nv.path.is_ident("debug") {
                return None;
            }
            match &nv.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(litstr),
                    ..
                }) => Some(litstr.value()),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn add_traits_bound(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
        }
    }
}
