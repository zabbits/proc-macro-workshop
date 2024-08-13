use proc_macro2::Span;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, Attribute, DeriveInput, Expr, ExprLit, ExprPath, Field, Ident,
    LitStr, PathArguments, PathSegment, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
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
                let ty = if let Some(ty) = get_generic_args(&f.ty, "Option") {
                    ty
                } else {
                    &f.ty
                };

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
                let ident = f.ident.as_ref().unwrap();
                let each = builder_each_attr(f);

                if let Some(each) = each {
                    let ty = get_generic_args(&f.ty, "Vec");
                    let each_ident = Ident::new(&each, Span::call_site());
                    let each_fn = quote! {
                        fn #each_ident(&mut self, #each_ident: #ty) -> &mut Self {
                            if self.#ident.is_none() {
                                self.#ident = Some(vec![]);
                            }
                            self.#ident.as_mut().unwrap().push(#each_ident);
                            self
                        }
                    };

                    if ident == each.as_str() {
                        return each_fn;
                    } else {
                        let ty = &f.ty;
                        return quote! {
                            #each_fn

                            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = Some(#ident);
                                self
                            }
                        };
                    }
                }

                let ty = if let Some(ty) = get_generic_args(&f.ty, "Option") {
                    ty
                } else {
                    &f.ty
                };
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
                let each = builder_each_attr(f);
                if each.is_some() {
                    return quote! {
                        #ident: self.#ident.clone().unwrap_or(vec![])
                    };
                }

                if get_generic_args(&f.ty, "Option").is_some() {
                    quote! {
                        #ident: self.#ident.clone()
                    }
                } else {
                    quote! {
                        #ident: self.#ident.clone().ok_or(#err_msg)?
                    }
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
            }
            .into()
        }
        syn::Data::Struct(_) => unimplemented!(),
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}

fn get_generic_args<'a>(ty: &'a Type, ident: &str) -> Option<&'a Type> {
    let seg = first_seg_of_generic(ty, ident)?;
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &seg.arguments
    {
        let arg = args.first()?;
        match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }
    } else {
        None
    }
}

fn first_seg_of_generic<'a>(ty: &'a Type, ident: &str) -> Option<&'a PathSegment> {
    if let Type::Path(type_path) = ty {
        if type_path.path.segments.is_empty() || type_path.path.segments[0].ident != ident {
            None
        } else {
            Some(&type_path.path.segments[0])
        }
    } else {
        None
    }
}

fn builder_each_attr(field: &Field) -> Option<String> {
    field.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("builder") {
            return None;
        }
        let (l, r) = parse_builder_attr(attr).unwrap();
        l.path.is_ident("each");
        Some(r.value())
    })
}

fn parse_builder_attr(attr: &Attribute) -> Result<(ExprPath, LitStr), Box<dyn std::error::Error>> {
    let expr = attr.parse_args::<Expr>()?;

    match expr {
        Expr::Assign(expr) => {
            let l = *expr.left;
            let r = *expr.right;
            if let (
                Expr::Path(p),
                Expr::Lit(ExprLit {
                    lit: syn::Lit::Str(litstr),
                    ..
                }),
            ) = (l, r)
            {
                Ok((p, litstr))
            } else {
                Err("oops".into())
            }
        }
        _ => Err("oops".into()),
    }
}
