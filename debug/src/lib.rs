use std::collections::HashSet;

use proc_macro2::Ident;
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, GenericArgument, GenericParam,
    Generics, Lit, Meta, Path, PathArguments, Type, TypePath,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
    let ident = input.ident;
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            add_field_traits_bound(&mut input.generics, &fields);

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

            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
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

fn add_field_traits_bound(generics: &mut Generics, fields: &Fields) {
    let params = &generics.params;
    let ty_ident_generics: Vec<_> = params
        .iter()
        .filter_map(|p| {
            if let GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        })
        .collect();

    let ty_add_where: HashSet<_> = fields
        .iter()
        .filter_map(|f| {
            if let Type::Path(tp) = &f.ty {
                for i in &ty_ident_generics {
                    if tp.path.is_ident(*i) || is_type_in_path(&tp.path, i) {
                        return Some(tp);
                    }
                }
            }
            None
        })
        .collect();

    let where_clause = generics.make_where_clause();
    for ty in ty_add_where {
        where_clause
            .predicates
            .push(syn::parse_quote! { #ty: std::fmt::Debug })
    }
}

// TODO:
// if field type is Option<A<B<C>>>, so we need find the deepest type same as the ident
fn is_type_in_path(path: &Path, ident: &Ident) -> bool {
    if path.segments.len() != 1 {
        return false;
    }
    if let PathArguments::AngleBracketed(arg) = &path.segments[0].arguments {
        for arg in &arg.args {
            if let GenericArgument::Type(Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            })) = arg
            {
                if segments.len() == 1 && &segments[0].ident == ident {
                    return true;
                }
            }
        }
    }
    false
}
