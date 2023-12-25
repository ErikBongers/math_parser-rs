use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(CastAny)]
pub fn cast_any_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Parsing the syntax for macro CastAny failed.");
    impl_cast_any_macro(&ast)
}

fn impl_cast_any_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl CastAny for #name {
            fn as_any_mut(&mut self) ->&mut dyn Any { self }
            fn as_any(& self) ->& dyn Any { self }
        }
    };
    gen.into()
}