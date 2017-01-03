#![feature(proc_macro, proc_macro_lib)]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;


use syn::{Field, Ident, Body, Variant, VariantData, MacroInput};
use proc_macro::TokenStream;

#[proc_macro_derive(SoA)]
pub fn soa_derive(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = gen_soa_derive(&ast);
    println!("{}", gen);
    gen.parse().unwrap()
}

fn gen_soa_derive(input: &MacroInput) -> quote::Tokens {
    let ident = &input.ident;
    let soa_ident = quote::Ident::from(format!("{}SoA", input.ident));
    let fields = match input.body {
        Body::Struct(ref data) => {
            match data {
                &VariantData::Struct(ref fields) => fields.clone(),
                _ => panic!("Only supports structs."),
            }
        }
        _ => panic!("Only supports structs."),
    };

    let vec_fields: Vec<quote::Tokens> = fields.iter()
        .map(|f| {
            let field_ident = f.ident.clone().unwrap();
            let field_ty = &f.ty;
            quote!{
                pub #field_ident: Vec<#field_ty>
            }
        })
        .collect();

    let field_idents: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();

    let push_self: Vec<quote::Tokens> = fields.iter()
        .map(|f| {
            let field = f.ident.clone().unwrap();
            quote!{
                self.#field.push(#field);
            }
        })
        .collect();

    let deconstruct_list: Vec<quote::Tokens> = fields.iter()
        .map(|f| {
            let field = f.ident.clone().unwrap();
            quote!{
                #field: #field
            }
        })
        .collect();

    quote!{
        #[derive(Debug)]
        struct #soa_ident {
            #(
                #vec_fields,
            )*
        }
        impl #soa_ident {
            pub fn new() -> Self {
                #soa_ident {
                    #(
                        #field_idents : Vec::new(),
                    )*
                }
            }

            pub fn push(&mut self, value: #ident){
                let #ident{#(#deconstruct_list, )*} = value;
                #(
                    #push_self
                )*
            }
        }
    }
}
