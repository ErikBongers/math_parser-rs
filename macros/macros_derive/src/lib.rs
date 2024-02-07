use std::str::FromStr;
use proc_macro::{Delimiter, Spacing, Span, TokenStream, TokenTree};
use proc_macro::token_stream::IntoIter;
use proc_macro::TokenTree::{Ident, Literal};
use std::iter::Peekable;
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

#[proc_macro]
pub fn print_tokens(input: TokenStream) -> TokenStream {
    println!("TOKENSTREAM::");
    println!("{:?}", input);
    TokenStream::new()
}


#[proc_macro]
pub fn define_errors(input: TokenStream) -> TokenStream {
    use proc_macro as pm;
    let mut enum_stream = TokenStream::from_str("#[derive(Clone, Serialize, PartialEq)] pub enum ErrorId").unwrap(); //unwrap: static text
    let mut functions_stream = TokenStream::new();

    let mut it = input.into_iter().peekable();

    let mut values_stream = TokenStream::new();
    while add_one_error(&mut it, &mut values_stream, &mut functions_stream) {}

    enum_stream.extend([
        TokenTree::from(pm::Group::new(Delimiter::Brace, values_stream)),
    ]);
    enum_stream.extend(functions_stream.into_iter());
    enum_stream
}

fn add_one_error(input: &mut Peekable<IntoIter>, enum_stream: &mut TokenStream, function_stream: &mut TokenStream) -> bool {
    use proc_macro as pm;
    //TODO: also check types of tokens.
    let Some(error_id_token) = input.next() else { return false; };
    let Ident(error_id) = &error_id_token else { return false; };
    let Some(_colon) = input.next() else { return false; };
    let Some(message_token) = input.next() else { return false; };
    let Literal(message) = &message_token else { return false; };

    if let Some(_comma) = input.peek() {
        input.next();
    }

    enum_stream.extend([
        error_id_token.clone(),
        TokenTree::from(pm::Punct::new(',', Spacing::Alone)),
    ]);

    let camel_id = to_camel_case(error_id.to_string().as_str());

    let message_str = message .to_string();
    let message_params: Vec<_> = message_str
        .split_inclusive(|c: char| c == '{' || c == '}')
        .filter(|s| s.contains(|c: char| c == '}'))
        .map(|s| &s[0..s.len()-1])
        .collect();
    /* building this function:
    fn some_error_id(param1: &str, range: Range) -> Error {
        Error {
            id: ErrorId::SomeErrorId,
            message: format!("Unknown expression `{param1}`.", param1=param1),
            range,
            stack_trace: None,
        }
    }
    */
    let mut func_stream = TokenStream::from_str("#[inline] pub fn").unwrap(); //unwrap: static text
    // ...some_error_id...
    func_stream.extend([
        TokenTree::from( pm::Ident::new(&camel_id, Span::call_site())),
    ]);

    let mut arg_tokens = TokenStream::new();
    // ...param1: &str,...
    for param_id in &message_params {
        if !is_ident(param_id) {
            panic!("` param {0}` for error {1} is not a valid identifier", param_id, error_id.to_string());
        }
        arg_tokens.extend([
            TokenTree::from(pm::Ident::new(&param_id, message_token.span())),
            TokenTree::from(pm::Punct::new(':', Spacing::Alone)),
            TokenTree::from(pm::Punct::new('&', Spacing::Alone)),
            TokenTree::from(pm::Ident::new("str", message_token.span())),
            TokenTree::from(pm::Punct::new(',', Spacing::Alone)),
        ]);
    }
    let tmp_stream = TokenStream::from_str("range: Range").unwrap(); //unwrap: static text
    arg_tokens.extend(tmp_stream.into_iter());
    let arg_group = TokenTree::from(pm::Group::new(Delimiter::Parenthesis, arg_tokens));
    func_stream.extend([arg_group]);
    let tmp_stream = TokenStream::from_str("-> Error").unwrap(); //unwrap: static text
    func_stream.extend(tmp_stream.into_iter());

    // format! args:
    let mut format_args = TokenStream::new();
    format_args.extend([ message_token.clone()]);
    // ..., param1=param1...
    for param_id in &message_params {
        format_args.extend([
            TokenTree::from(pm::Punct::new(',', Spacing::Alone)),
            TokenTree::from(pm::Ident::new(param_id, Span::call_site())),
            TokenTree::from(pm::Punct::new('=', Spacing::Alone)),
            TokenTree::from(pm::Ident::new(param_id, Span::call_site())),
        ]);
    }
    let format_arg_group = TokenTree::from(pm::Group::new(Delimiter::Parenthesis, format_args));

    let mut error_fields_stream = TokenStream::from_str("id: ErrorId::").unwrap(); //static text
    error_fields_stream.extend([error_id_token.clone()]);
    let tmp_stream = TokenStream::from_str(", message: format!").unwrap(); //unwrap: static text
    error_fields_stream.extend(tmp_stream.into_iter());
    error_fields_stream.extend([format_arg_group]);
    error_fields_stream.extend(TokenStream::from_str(", range, stack_trace: None, error_type: ErrorType::E,")); //TODO: test for type Warning.

    let error_group = TokenTree::from(pm::Group::new(Delimiter::Brace, error_fields_stream));
    let funct_body_group = TokenTree::from(pm::Group::new(Delimiter::Brace, TokenStream::from_iter([
        TokenTree::from(pm::Ident::new("Error", Span::call_site())),
        error_group
    ])));
    func_stream.extend([funct_body_group]);

    function_stream.extend(func_stream.into_iter());
    true
}


fn to_camel_case(id_str: &str) -> String {
    let mut list = Vec::new();
    let mut last = 0;
    for (index, _c) in id_str.match_indices(|c: char| c.is_uppercase()) {
        if last != index {
            list.push(&id_str[last..index]);
        }
        last = index;
    }
    if last < id_str.len() {
        list.push(&id_str[last..]);
    }

    let mut camel_case: String = String::new();
    for part in &list {
        camel_case += part.to_lowercase().as_str();
        camel_case.push('_');
    }
    camel_case.pop();
    camel_case
}


fn is_ident(string: &str) -> bool {
    let mut chars = string.chars();
    if let Some(start) = chars.next() {
        is_id_start(start) && chars.all(is_id_continue)
    } else {
        false
    }
}

fn is_id_start(c: char) -> bool {
    // This is XID_Start OR '_' (which formally is not a XID_Start).
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}
fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}