#![allow(dead_code)]
#![recursion_limit="2048"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate rustfmt;
extern crate itertools;

use syn::{Field, Ident, Body, Variant, VariantData, MacroInput};
use proc_macro::TokenStream;

#[proc_macro_derive(Vector)]
pub fn soa_derive(input: TokenStream) -> TokenStream {
    use rustfmt::*;
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = gen_vec_derive(&ast);
    let mut out = String::new();
    // let config =
    //    config::Config { write_mode: config::WriteMode::Plain, ..config::Config::default() };
    // let fmt_r = format_input::<Vec<u8>>(Input::Text(gen.to_string()),
    //                                    &config,
    //                                    Some(unsafe { out.as_mut_vec() }));
    // if let Ok(fmt) = fmt_r {
    //    println!("{}", out);
    // }
    gen.parse().unwrap()
}
struct OpTrait {
    name: Ident,
    fn_name: Ident,
    op: Ident,
}

impl OpTrait {
    pub fn new<T: Into<Ident>>(name: T, fn_name: T, op: T) -> Self {
        OpTrait {
            name: name.into(),
            fn_name: fn_name.into(),
            op: op.into(),
        }
    }
}

fn impl_op_vec(ident: &Ident,
               field_idents: &Vec<Ident>,
               name: Ident,
               fn_name: Ident,
               op: Ident)
               -> quote::Tokens {
    let s: &Vec<_> = &field_idents.iter()
        .map(|ident| quote::Ident::from(format!("{}: self.{} {} other.{}", ident, ident, op, ident)))
        .collect();
    quote!{
        impl<T: Num> ::std::ops::#name for #ident<T>{
            type Output = Self;
            fn #fn_name(self, other: Self) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }

        impl<'a, 'b, T: Num + Copy> ::std::ops::#name<&'b #ident<T>> for &'a #ident<T>{
            type Output = #ident<T>;
            fn #fn_name(self, other: &'b #ident<T>) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }

        impl<'a, T: Num + Copy> ::std::ops::#name<#ident<T>> for &'a #ident<T>{
            type Output = #ident<T>;
            fn #fn_name(self, other: #ident<T>) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }

        impl<'a, T: Num + Copy> ::std::ops::#name<&'a #ident<T>> for #ident<T>{
            type Output = #ident<T>;
            fn #fn_name(self, other: &'a #ident<T>) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }
    }
}

fn impl_op_scalar(ident: &Ident,
                  field_idents: &Vec<Ident>,
                  name: Ident,
                  fn_name: Ident,
                  op: Ident)
                  -> quote::Tokens {
    let s: &Vec<_> = &field_idents.iter()
        .map(|ident| quote::Ident::from(format!("{}: self.{} {} other", ident, ident, op)))
        .collect();
    quote!{
        impl<'a, T: Num + Copy> ::std::ops::#name<T> for &'a #ident<T>{
            type Output = #ident<T>;
            fn #fn_name(self, other: T) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }

        impl<T: Num + Copy> ::std::ops::#name<T> for #ident<T>{
            type Output = Self;
            fn #fn_name(self, other: T) -> Self::Output{
                #ident{
                    #(
                        #s,
                    )*
                }
            }
        }
    }
}

fn gen_vec_derive(input: &MacroInput) -> quote::Tokens {
    let ident = &input.ident;
    let type_ident = &input.generics.ty_params[0].ident;

    let fields = match input.body {
        Body::Struct(ref data) => {
            match data {
                &VariantData::Struct(ref fields) => fields.clone(),
                _ => panic!("Only supports structs."),
            }
        }
        _ => panic!("Only supports structs."),
    };

    let dim = fields.len();
    let ndim = dim.pow(dim as u32);
    let field_idents_: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
    let field_idents1 = &field_idents_;
    let field_idents2 = &field_idents_;

    let s_iter: Vec<_> = (0..field_idents_.len())
        .map(|i| {
            let iter: Vec<_> = field_idents_.iter()
                .map(|ident| ::std::iter::repeat(ident.clone()).take(dim.pow(i as u32)))
                .collect();
            let iter2 = iter.into_iter().fold(Vec::new(), |acc, iter| {
                acc.into_iter().chain(iter).collect::<Vec<_>>()
            });

            iter2.into_iter().cycle().take(ndim).collect::<Vec<_>>()
        })
        .collect();

    let swizzle_ident: Vec<_> = (0..ndim)
        .map(|idx| {
            let mut name = String::new();
            for i in 0..dim {
                name.push_str(&s_iter[i][idx].as_ref());
            }
            name
        })
        .collect();

    let swizzle: Vec<_> = swizzle_ident.into_iter()
        .map(|name| {
            let swizzle_field = name.chars().map(|c| quote::Ident::new(format!("{}", c)));
            let fn_name = quote::Ident::new(name.clone());
            quote!{
            pub fn #fn_name(&self) -> Self{
                #ident{
                    #(
                        #field_idents1: self.#swizzle_field,
                    )*
                }
            }
        }
        })
        .collect();

    let optraits = vec![OpTrait::new("Add", "add", "+"),
                        OpTrait::new("Mul", "mul", "*"),
                        OpTrait::new("Div", "div", "/"),
                        OpTrait::new("Sub", "sub", "-")];

    let impl_op_scalar = optraits.iter()
        .map(|ot| {
            impl_op_scalar(ident,
                           &field_idents_,
                           ot.name.clone(),
                           ot.fn_name.clone(),
                           ot.op.clone())
        });

    let impl_op_vec = optraits.iter()
        .map(|ot| {
            impl_op_vec(ident,
                        &field_idents_,
                        ot.name.clone(),
                        ot.fn_name.clone(),
                        ot.op.clone())
        });

    let first_field = field_idents_[0].clone();
    let number_of_fields = field_idents_.len();
    quote!{
        impl<T> #ident<T>
            where T: Num + Copy + Zero {
            pub fn new(#(#field_idents1: T, )*) -> Self{
                #ident{
                    #(
                        #field_idents1: #field_idents2,
                    )*
                }
            }

            #[inline]
            pub fn zero() -> Self{
                #ident{
                    #(
                        #field_idents1: T::zero(),
                    )*
                }
            }

            #[inline]
            pub fn dot(&self, other: &Self) -> T{
                #(
                    self.#field_idents1 * other.#field_idents2
                )+ *
            }

            #[inline]
            pub fn length_sq(&self) -> T {
                self.dot(self)
            }

            #[inline]
            pub fn as_raw_slice(&self) -> &[T]{
                let ptr = &self.#first_field as *const T;
                unsafe {
                    ::std::slice::from_raw_parts(ptr, #number_of_fields)
                }
            }

            #[inline]
            pub fn distance_sq(&self, other: &Self) -> T{
                (self - other).length_sq()
            }

            pub fn map<F, B>(&self,f: F) -> #ident<B>
                where F: Fn(T) -> B {
                #ident{
                    #(
                        #field_idents1: f(self.#field_idents2),
                    )*
                }
            }

            #(#swizzle)*
        }

        impl<T> #ident<T>
            where T: Float + Copy {

            #[inline]
            pub fn max(&self) -> T{
                variadic2!(T::max, #(self.#field_idents1),*)
            }

            #[inline]
            pub fn min(&self) -> T{
                variadic2!(T::min, #(self.#field_idents1),*)
            }

            #[inline]
            pub fn length(&self) -> T {
                self.length_sq().sqrt()
            }

            #[inline]
            pub fn project(&self, other: &Self) -> Self {
                other * (self.dot(other) / other.length_sq())
            }

            #[inline]
            pub fn normalize(&self) -> Option<Self> {
                 let len_sq = self.length_sq();
                 if len_sq == T::one() {
                     Some(*self)
                 } else if len_sq == T::zero() {
                     None
                 } else {
                     Some(self / len_sq.sqrt())
                 }
            }

            #[inline]
            pub fn reflect_normal(&self, normal: &Self) -> Self{
                let two: T = NumCast::from(2).unwrap();
                let r = self - normal;
                r
            }

            #[inline]
            pub fn distance(&self, other: &Self) -> T{
                self.distance_sq(other).sqrt()
            }

            #[inline]
            pub fn lerp(&self, target: &Self, scale: T) -> Self{
                self + (target - self) * scale
            }
        }

        // 'match' is too slow here, see: https://godbolt.org/g/Bgqyx2
        impl<T> ::std::ops::Index<usize> for #ident<T> {
            type Output = T;
            fn index(&self, idx: usize) -> &T{
                unsafe {
                    let ptr = &self.#first_field as *const T;
                    &*ptr.offset(idx as isize)
                }
            }
        }

        #(#impl_op_vec)*
        #(#impl_op_scalar)*
    }
}
