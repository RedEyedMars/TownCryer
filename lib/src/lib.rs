use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Saveable)]
pub fn derive_saveable(input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let primitives = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str",
    ];

    // Ensure we are working with an Enum
    let generated = match &ast.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Unnamed(fields) => {
                    let (prims, nonprims): (Vec<(syn::Index, &Type)>, Vec<(syn::Index, &Type)>) =
                        fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, f)| (syn::Index::from(i), &f.ty))
                            .partition(|(i, ty)| {
                                let ty_string = quote!(#ty).to_string();
                                primitives.contains(&ty_string.as_str())
                            });
                    let prims: Vec<syn::Index> = prims.into_iter().map(|(a, _)| a).collect();
                    let nonprims: Vec<syn::Index> = nonprims.into_iter().map(|(a, _)| a).collect();

                    quote! {
                        impl #impl_generics crate::save::Saveable  for #name #ty_generics #where_clause {
                            fn save(&self,
                                serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
                                context: &mut crate::save::SaveContext) -> std::result::Result<(), rmp_serde::encode::Error> {
                                    use serde::Serialize;
                                    #(self.#prims.serialize(&mut *serializer)?;)*
                                    #(self.#nonprims.save(&mut *serializer, context)?;)*
                                    Ok(())
                            }
                        }
                    }
                }
                Fields::Named(fields) => {
                    // Filter and map over fields to build details *only* for non-primitives
                    let (prims, nonprims): (Vec<(&syn::Ident, &Type)>, Vec<(&syn::Ident, &Type)>) =
                        fields
                            .named
                            .iter()
                            .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
                            .partition(|(i, ty)| {
                                let ty_string = quote!(#ty).to_string();
                                primitives.contains(&ty_string.as_str())
                            });
                    let prims: Vec<&syn::Ident> = prims.into_iter().map(|(a, _)| a).collect();
                    let nonprims: Vec<&syn::Ident> = nonprims.into_iter().map(|(a, _)| a).collect();

                    quote! {
                        impl #impl_generics crate::save::Saveable  for #name #ty_generics #where_clause {
                            fn save(&self,
                                serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
                                context: &mut crate::save::SaveContext) -> std::result::Result<(), rmp_serde::encode::Error> {
                                    use serde::Serialize;
                                    #(self.#prims.serialize(&mut *serializer)?;)*
                                    #(self.#nonprims.save(&mut *serializer, context)?;)*
                                    Ok(())
                            }
                        }
                    }
                }
                _ => quote! {
                    panic!("AHH");
                },
            }
        }
        Data::Enum(data_enum) => {
            let arms = data_enum
                .variants
                .iter()
                .enumerate()
                .map(|(index, variant)| {
                    let var_name = &variant.ident;
                    let index = index as u8;
                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                #name::#var_name => #index.serialize(&mut *serializer)
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let bindings = fields.unnamed.iter().enumerate().map(|(i, _)| {
                                syn::Ident::new(&format!("i{}", i), proc_macro2::Span::call_site())
                            });

                            let (prims, nonprims): (
                                Vec<(syn::Ident, &Type)>,
                                Vec<(syn::Ident, &Type)>,
                            ) = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, f)| {
                                    (
                                        syn::Ident::new(
                                            &format!("i{}", i),
                                            proc_macro2::Span::call_site(),
                                        ),
                                        &f.ty,
                                    )
                                })
                                .partition(|(i, ty)| {
                                    let ty_string = quote!(#ty).to_string();
                                    primitives.contains(&ty_string.as_str())
                                });
                            let prims: Vec<syn::Ident> =
                                prims.into_iter().map(|(a, _)| a).collect();
                            let nonprims: Vec<syn::Ident> =
                                nonprims.into_iter().map(|(a, _)| a).collect();

                            quote! {
                                #name::#var_name(#(#bindings),*) => {
                                    #index.serialize(&mut *serializer)?;
                                    #(#prims.serialize(&mut *serializer)?;)*
                                    #(#nonprims.save(&mut *serializer, context)?;)*
                                    Ok(())
                                }
                            }
                        }
                        Fields::Named(fields) => {
                            let field_names: Vec<&syn::Ident> = fields
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().unwrap())
                                .collect();

                            // Filter and map over fields to build details *only* for non-primitives
                            let (prims, nonprims): (
                                Vec<(&syn::Ident, &Type)>,
                                Vec<(&syn::Ident, &Type)>,
                            ) = fields
                                .named
                                .iter()
                                .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
                                .partition(|(i, ty)| {
                                    let ty_string = quote!(#ty).to_string();
                                    primitives.contains(&ty_string.as_str())
                                });
                            let prims: Vec<&syn::Ident> =
                                prims.into_iter().map(|(a, _)| a).collect();
                            let nonprims: Vec<&syn::Ident> =
                                nonprims.into_iter().map(|(a, _)| a).collect();

                            quote! {
                                #name::#var_name{#(#field_names),*} => {
                                    #index.serialize(&mut *serializer)?;
                                    #(#prims.serialize(&mut *serializer)?;)*
                                    #(#nonprims.save(&mut *serializer, context)?;)*
                                    Ok(())
                                }
                            }
                        }
                    }
                });
            quote! {
                impl crate::save::Saveable for #name {
                    fn save(&self,
                        serializer: &mut rmp_serde::Serializer<&mut Vec<u8>>,
                        context: &mut crate::save::SaveContext) -> std::result::Result<(), rmp_serde::encode::Error> {
                        use serde::Serialize;
                        match self {
                            #(#arms,)*
                        }
                    }
                }
            }
        }
        x => panic!("Saveable derive macro only supports enums. was: {:?}", x),
    };

    TokenStream::from(generated)
}
