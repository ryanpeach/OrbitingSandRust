use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Produces a trace! log when calling the function
#[proc_macro_attribute]
pub fn call_log(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.to_string();
    let fn_block = input.block;
    let fn_signature = input.sig;

    // Create the new function body with the trace statement added
    let expanded = quote! {
        #fn_signature {
            bevy::log::trace!("{} running", #fn_name);
            #fn_block
        }
    };

    // Return the generated function
    TokenStream::from(expanded)
}

/// Produces a trace_once! log when calling the function
#[proc_macro_attribute]
pub fn call_log_once(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.to_string();
    let fn_block = input.block;
    let fn_signature = input.sig;

    // Create the new function body with the trace statement added
    let expanded = quote! {
        #fn_signature {
            bevy::log::trace_once!("{} running", #fn_name);
            #fn_block
        }
    };

    // Return the generated function
    TokenStream::from(expanded)
}
