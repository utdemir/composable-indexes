use proc_macro::TokenStream;

mod zip;

#[proc_macro]
pub fn zip(input: TokenStream) -> TokenStream {
    zip::run(input)
}
