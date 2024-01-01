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

#[proc_macro_derive(Node)]
pub fn node_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Parsing the syntax for macro CastAny failed.");
    crate::impl_node(&ast)
}

fn impl_node(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Node for #name {
            fn get_node_data(&self) -> &NodeData {
                &self.node_data
            }
            fn get_node_data_mut(&mut self) -> &mut NodeData {
                &mut self.node_data
            }
        }
    };
    gen.into()
}