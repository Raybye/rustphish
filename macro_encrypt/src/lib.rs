#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

include!("../config.rs");

#[proc_macro]
pub fn encrypt(input: TokenStream) -> TokenStream {
    let input_str = parse_macro_input!(input as LitStr);
    let key = KEY.clone();

    let encrypted_str = simple_encrypt(&input_str.value(), key);

    // Generates the Rust code that decrypts the string at runtime.
    let r#gen = quote! {
        {
            // Built-in function to decrypt the string
            fn simple_decrypt(input: &[u16], key: u16) -> alloc::string::String {
                let decrypted_utf16: alloc::vec::Vec<u16> = input.iter().map(|x| x ^ key).collect();
                alloc::string::String::from_utf16_lossy(&decrypted_utf16)
            }

            simple_decrypt(&[#(#encrypted_str),*], #key)
        }
    };

    r#gen.into()
}

fn simple_encrypt(input: &str, key: u16) -> Vec<u16> {
    let utf16: Vec<u16> = input.encode_utf16().collect();
    utf16.iter().map(|c| c ^ key).collect()
}
