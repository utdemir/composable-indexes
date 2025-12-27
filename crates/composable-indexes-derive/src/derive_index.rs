use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta, Type};

pub fn run(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Extract the input type from #[index(Type)] attribute
    let input_type = extract_input_type_from_attrs(&input.attrs)
        .expect("Index derive requires #[index(Type)] attribute");

    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("Index derive only supports structs"),
    };

    // Handle different field types
    match &data_struct.fields {
        // Named fields: struct Foo { field1: T1, field2: T2 }
        Fields::Named(fields_named) => {
            let fields = &fields_named.named;

            // Generate insert, update, and remove implementations by calling each field
            let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

            let insert_calls = field_names.iter().map(|name| {
                quote! { self.#name.insert(seal, op); }
            });

            let update_calls = field_names.iter().map(|name| {
                quote! { self.#name.update(seal, op); }
            });

            let remove_calls = field_names.iter().map(|name| {
                quote! { self.#name.remove(seal, op); }
            });

            quote! {
                impl #impl_generics composable_indexes::core::Index<#input_type> for #struct_name #ty_generics #where_clause {
                    fn insert(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Insert<#input_type>) {
                        #(#insert_calls)*
                    }

                    fn update(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Update<#input_type>) {
                        #(#update_calls)*
                    }

                    fn remove(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Remove<#input_type>) {
                        #(#remove_calls)*
                    }
                }
            }
        }
        // Single unnamed field: struct Foo(T)
        Fields::Unnamed(fields_unnamed) => {
            if fields_unnamed.unnamed.len() != 1 {
                panic!("Index derive for tuple structs only supports structs with exactly one field");
            }

            quote! {
                impl #impl_generics composable_indexes::core::Index<#input_type> for #struct_name #ty_generics #where_clause {
                    fn insert(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Insert<#input_type>) {
                        self.0.insert(seal, op);
                    }

                    fn update(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Update<#input_type>) {
                        self.0.update(seal, op);
                    }

                    fn remove(&mut self, seal: composable_indexes::core::Seal, op: &composable_indexes::core::Remove<#input_type>) {
                        self.0.remove(seal, op);
                    }
                }
            }
        }
        Fields::Unit => {
            panic!("Index derive does not support unit structs");
        }
    }.into()
}

// Helper function to extract the input type from #[index(Type)] attribute
fn extract_input_type_from_attrs(attrs: &[syn::Attribute]) -> Option<Type> {
    for attr in attrs {
        if attr.path().is_ident("index") {
            if let Meta::List(meta_list) = &attr.meta {
                // Parse the tokens inside the attribute as a Type
                let ty: Type = syn::parse2(meta_list.tokens.clone()).ok()?;
                return Some(ty);
            }
        }
    }
    None
}
