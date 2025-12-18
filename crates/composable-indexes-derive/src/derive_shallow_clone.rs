use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

pub fn run(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("ShallowClone derive only supports structs"),
    };

    // Handle different field types
    match &data_struct.fields {
        // Named fields: struct Foo { field1: T1, field2: T2 }
        Fields::Named(fields_named) => {
            let fields = &fields_named.named;

            // Generate field initialization by calling shallow_clone() or clone() based on attributes
            let field_clones = fields.iter().map(|field| {
                let name = &field.ident;
                let use_regular_clone = has_mark_as_shallow_attr(&field.attrs);
                
                if use_regular_clone {
                    quote! { #name: self.#name.clone() }
                } else {
                    quote! { #name: self.#name.shallow_clone() }
                }
            });

            quote! {
                impl #impl_generics composable_indexes::ShallowClone for #struct_name #ty_generics #where_clause {
                    fn shallow_clone(&self) -> Self {
                        Self {
                            #(#field_clones,)*
                        }
                    }
                }
            }
        }
        // Single unnamed field: struct Foo(T)
        Fields::Unnamed(fields_unnamed) => {
            if fields_unnamed.unnamed.len() != 1 {
                panic!("ShallowClone derive for tuple structs only supports structs with exactly one field");
            }

            quote! {
                impl #impl_generics composable_indexes::ShallowClone for #struct_name #ty_generics #where_clause {
                    fn shallow_clone(&self) -> Self {
                        Self(self.0.shallow_clone())
                    }
                }
            }
        }
        Fields::Unit => {
            panic!("ShallowClone derive does not support unit structs");
        }
    }.into()
}

// Helper function to check if a field has #[index(mark_as_shallow)] attribute
fn has_mark_as_shallow_attr(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("index") {
            if let Meta::List(meta_list) = &attr.meta {
                // Parse the tokens to check if it contains "mark_as_shallow"
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str == "mark_as_shallow" {
                    return true;
                }
            }
        }
    }
    false
}
