extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, ItemImpl, ItemStruct, parse::Nothing, parse::Parser, parse_macro_input};

#[proc_macro_derive(RegisterDeltaNode)]
pub fn register_delta_node(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name: Ident = ast.ident;

    // TODO:    - Figure out how to automatically deal with type. So that whatever the items on_execute returns is the type for the DeltaNode
    //          - If nothing is returned by the on_execute method then a None: Option should be returned
    //          - Probably need some way to keep track of the state between 
    

    let output = quote! { 
        impl DeltaNode<i32> for #name {
            fn execute(mut self) -> i32 {
                self.pre_execute();
                let res: i32 = self.on_execute();
                self.post_execute();
                res
            }
        }
     };

    output.into()
}

#[proc_macro_attribute]
pub fn delta_node_attributes(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(_args as Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { __num_attributes: i32 })
                .unwrap(),
        );
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { __set_attributes: i32 })
                .unwrap(),
        );
    }

    return quote! {
        #item_struct
    }
    .into();
}

// I think the way to go about this is as follows:
// in the struct macro add the needed attributes, also check for attributes with a [#delta_ignore] macro
// then for each attribute create a  set_[] function, that sets the variable and updates the __set_attributes thing
// also create the initialize function. if the default is not specified [#delta_default(*default value*)] then it is just whatever the rust default is.
// also create a default reset() method, that resets all of the non-ignored values back to defaults

// then with the impl macro we can generate all of the functions (pre_execute, on_execute, post_execute). If they don't exist then create them.
// can also implement the DeltaNode<T> stuff using the info from on_execute...

// #[proc_macro_attribute]
// pub fn delta_node_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
//     let mut item_impl = parse_macro_input!(input as ItemImpl);
//     let _ = parse_macro_input!(_args as Nothing);

//     if let syn::Fields::Named(ref mut fields) = item_impl.fields {
//         fields.named.push(
//             syn::Field::parse_named
//                 .parse2(quote! { __num_attributes: i32 })
//                 .unwrap(),
//         );
//         fields.named.push(
//             syn::Field::parse_named
//                 .parse2(quote! { __set_attributes: i32 })
//                 .unwrap(),
//         );
//     }

//     return quote! {
//         #item_impl
//     }
//     .into();
// }