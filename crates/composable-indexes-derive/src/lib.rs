use proc_macro::TokenStream;

mod derive_index;
mod derive_shallow_clone;

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

/// Derive macro for automatically implementing the `ShallowClone` trait.
///
/// This macro generates an implementation of `composable_indexes::ShallowClone` for a struct
/// by calling `shallow_clone()` on each field. This is useful for types that use persistent
/// data structures where shallow cloning is more efficient than deep cloning.
///
/// # Field Attributes
///
/// - `#[index(mark_as_shallow)]`: Use regular `clone()` instead of `shallow_clone()` for this field.
///   This is useful for types that don't implement `ShallowClone`.
///
/// # Example
///
/// ```rust
/// use composable_indexes::{index, aggregation};
/// use composable_indexes_derive::ShallowClone;
///
/// #[derive(Clone, ShallowClone)]
/// struct MyIndex {
///     field1: index::TrivialIndex,
///     field2: aggregation::CountIndex,
/// }
/// ```
#[proc_macro_derive(ShallowClone, attributes(index))]
pub fn derive_shallow_clone(input: TokenStream) -> TokenStream {
    derive_shallow_clone::run(input)
}
