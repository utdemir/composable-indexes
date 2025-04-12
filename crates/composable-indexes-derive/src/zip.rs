use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, Expr, Token,
};

struct ZipArgs {
    exprs: Punctuated<Expr, Token![,]>,
}

impl Parse for ZipArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let exprs = Punctuated::parse_terminated(input)?;
        Ok(ZipArgs { exprs })
    }
}

pub fn run(input: TokenStream) -> TokenStream {
    let ZipArgs { exprs } = parse_macro_input!(input as ZipArgs);
    let len = exprs.len();

    if len < 2 {
        return TokenStream::from(quote! {
            compile_error!("zip requires at least 2 arguments")
        });
    }

    // Just use the base name for the identifier
    let zip_fn = syn::Ident::new(&format!("zip{}", len), proc_macro2::Span::call_site());

    // Use the path in the expanded code
    let expanded = quote! {
        composable_indexes::indexes::zip::#zip_fn(#exprs)
    };

    TokenStream::from(expanded)
}
