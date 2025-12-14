use proc_macro::TokenStream;

mod derive_index;

/// Derive macro for automatically implementing the `Index` trait.
///
/// This macro generates an implementation of `composable_indexes::core::Index` for a struct
/// where each field is itself an `Index`. This allows you to compose multiple indexes together
/// without manually writing the Index trait implementation.
///
/// # Required Attribute
///
/// The `#[index(Type)]` attribute must be specified to indicate the type being indexed.
/// ```
#[proc_macro_derive(Index, attributes(index))]
pub fn derive_index(input: TokenStream) -> TokenStream {
    derive_index::run(input)
}
