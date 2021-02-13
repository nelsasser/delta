extern crate proc_macro;
use std::str::FromStr;
use std::collections::HashMap;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, spanned::Spanned};
use syn::{Ident, ImplItem, ItemImpl, ItemStruct};
use syn::{parse::Nothing, parse::Parser, parse_macro_input};

#[macro_use]
macro_rules! __delta_hashmap_literal {
    // https://stackoverflow.com/questions/28392008/more-concise-hashmap-initialization
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

// should code generated by the delta macros allways be prepended by two underscores? So it doesn't collide with any user created names

// this macro generates the set and reset methods for all of the registered fields.
// also generates __execute method to implement the DeltaNode trait
// TODO: may want to move the __execute generation into the impl macro because it is easier to get access to the return type in there...
#[proc_macro_derive(RegisterDeltaNode, attributes(delta_ignore, delta_noreset, delta_default))]
pub fn register_delta_node(input: TokenStream) -> TokenStream {
    let ast: ItemStruct = parse_macro_input!(input as ItemStruct);

    let name: Ident = ast.ident;

    // TODO:    - Figure out how to automatically deal with type. So that whatever the items on_execute returns is the type for the DeltaNode
    //          - If nothing is returned by the on_execute method then a None: Option should be returned
    //          - Probably need some way to keep track of the state between 
    

    // add all of the set_* functions for each of the exposed fields in the struct
    // also add all of the reset functions for each of the exposed fields
    let mut set_functions = vec![];
    let mut reset_functions  = vec![];
    let mut reset_calls = vec![];

    for field in ast.fields.iter() {
        // TODO: also want to ignore `pub` fields, since we cannot guarantee control with the set and reset functions, enable generation using a flag
        // TODO: every generated field should have a reset function, but if `noreset` flag is set then it will only be excluded from main reset function

        match &field.ident {
            Some(name) => {

                // these fields should not be exposed to anything but internal and generated functions
                if name.to_string() == "__set_fields" || name.to_string() == "__num_fields" {
                    continue;
                }

                match delta_ignore(field) {
                    Ok(ignore) => if ignore { continue; } else { () },
                    Err(ts) => return ts.into(),
                }

                let ty: &syn::Type = &field.ty;
                let sfunc_name: proc_macro2::Ident = format_ident!("__set_{}", name);
                let rfunc_name: proc_macro2::Ident = format_ident!("__reset_{}", name);

                // generate set functions
                set_functions.insert(set_functions.len(), quote! {
                    pub fn #sfunc_name(&mut self, #name: #ty) {
                        self.#name = #name;
                        self.__set_fields += 1;
                    }
                });
                
                // generate reset functions
                reset_functions.insert(reset_functions.len(), quote! {
                    pub fn #rfunc_name(&mut self) {
                        self.#name = Default::default(); // TODO: allow user to change default value here...
                        self.__set_fields -= 1;
                    } 
                });

                // if this flag is set we want to have generated reset and set functions,
                // but we don't want to automatically call them in the overall reset function
                if let Ok(_) = has_attribute(field, "delta_noreset") {
                    continue;
                }

                // generate the calls to each reset functions to be used in main reset function
                reset_calls.insert(reset_calls.len(), quote! {
                    self.#rfunc_name();
                });
                
            },
            None => continue, // skip un-named fields
        }
    }

    // combine all of the reset and set functions
    let output_set_reset = quote!{
        impl #name {
            #(#set_functions)*
            #(#reset_functions)*

            pub fn __reset(&mut self) {
                #(#reset_calls)*
            }
        }
    };

    // generate code to call both init steps
    let field_list: Vec<proc_macro2::TokenStream> = ast.fields.iter().filter(|x| {
        // filter out all of the fields that we generated
        if let Some(name) = &x.ident {
            return name.to_string() != "__num_fields" && name.to_string() != "__set_fields"
        }
        false
    }).map(|x| {
        // then collect the names of each of the fields
        match &x.ident {
            Some(name) => quote!{ #name },
            _ => proc_macro2::TokenStream::new()
        }
    }).collect();

    let default_init = default_initialize(&field_list, &name);
    
    let output_init = quote! {
        impl #name {
            #default_init
        }
    };

    // add an implementation for the required execution code
    let output_deltanode = quote! { 
        impl DeltaNode<Impulse<i32>, #name> for #name {
            fn __execute(mut self) -> Impulse<i32> {
                self.__pre_execute();
                let res: Impulse<i32> = self.__on_execute();
                self.__post_execute();
                res
            }

            fn __initialize() -> Box<#name> {
                let mut ret: Box<#name> = <#name>::__default_initialize();
                ret.__custom_initialize();
                ret
            }
        }
    };
    
    let output = quote! {
        #output_deltanode
        #output_init
        #output_set_reset
    };

    output.into()
}

// this macro generates the extra two fields __num_fields and __set_fields that are required under the hood
#[proc_macro_attribute]
pub fn delta_node_struct(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(_args as Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { __num_fields: i32 })
                .unwrap(),
        );
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { __set_fields: i32 })
                .unwrap(),
        );
    }

    return quote! {
        #item_struct
    }
    .into();
}

// this macro goes through the node implementation
// it checks to make sure that the required methods exists, and generates default ones if they don't
// if a method does exist it also checks / modifies it to make it correct (e.g. adding __num_fields and __set_fields fields to __initialize)
#[proc_macro_attribute]
pub fn delta_node_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    let _ = parse_macro_input!(_args as Nothing);

    // need to get the name of the implementation (the `for ...` I guess). Needed to properly generate 
    
    // gather up all of the methods
    let mut methods = vec![];
    for item in item_impl.items.iter() {
        match item {
            ImplItem::Method(method) => {
                methods.insert(methods.len(), method.clone());
            },
            _ => continue,
        }
    }

    // Next, need to go through the found methods and generate code accordingly

    // store all of the generated function here
    let mut generated_functions = vec![];

    // create some helper tables so we only generate methods as needed
    let mut method_flags: HashMap<String, bool> = __delta_hashmap_literal![ "__custom_initialize".to_owned() => false,
                                                                            "__pre_execute".to_owned() => false,
                                                                            "__on_execute".to_owned() => false,
                                                                            "__post_execute".to_owned() => false];

    let attr_name_to_wrap_name: HashMap<String, &str> = __delta_hashmap_literal![   "deltaf-init".to_owned() => "__custom_initialize",
                                                                                    "deltaf-pre".to_owned() => "__pre_execute",
                                                                                    "deltaf-on".to_owned() => "__on_execute",
                                                                                    "deltaf-post".to_owned() => "__post_execute"];

    // go through each of the methods and check if the DeltaNode method requirements are satisfied
    // also generate any needed wrapper function if found attribute
    for method in methods.iter() {
        let method_name = method.sig.ident.to_string();

        // update flag if we found a method with a correct name
        // if name is found then continue to next method because we don't care about any wrapper attributes
        if let Some(m_flag) = method_flags.get_mut(&method_name) {
            *m_flag = true;
            continue;
        }
        
        // go through all of the attributes for each method and check if any specify wrappers to generate
        for attr in method.attrs.iter() {
            let attr_name = attr.tokens.to_string();
            let wrap_name = attr_name_to_wrap_name.get(&attr_name);
        
            match wrap_name {
                Some(wrapper) => {
                    // create a wrapper function
                    generated_functions.insert(generated_functions.len(), 
                        generate_wrapper_s(wrapper, &method_name, None, true, true, true)
                    );
                    
                    // update the flag 
                    if let Some(m_flag) = method_flags.get_mut(*wrapper) {
                        *m_flag = true;
                        // purposfully choosing not to stop iterating over all of the other attributes because a user may want to reuse a method
                        // break; 
                    }
                },
                _ => continue,
            }
        }
    }

    // now we need to get default functions for all of the non-existent required functions
    for (method, flag) in &method_flags {
        if !flag {
            generated_functions.insert(generated_functions.len(), match method.as_ref() {
                "__custom_initialize" => default_custom_initialize(),
                "__pre_execute" => default_pre_execute(),
                "__on_execute" => default_on_execute(),
                "__post_execute" => default_post_execute(),
                _ => proc_macro2::TokenStream::new(),
            });
        }
    }

    // can either push back into array, or if name can be figured out easily then that seems like a neater way of doing it, without modifying any written code
    // can't seem to easily get the name, but this is straightforward, but it does 'modify' the input code, which I don't like
    for md in generated_functions.iter() {
        let q = syn::parse_quote!(#md);
        item_impl.items.push(q);
    }

    let tokens = quote! {
        #item_impl
    };

    // preferred way of getting the output, maybe someday :)
    // let tokens = quote! {
    //     #item_impl

    //     impl #name {
    //         #(#generated_functions)*
    //     }
    // };
    
    tokens.into()
}

fn default_initialize(field_list: &Vec<proc_macro2::TokenStream>, name: &Ident, ) -> proc_macro2::TokenStream {
    // TODO: if using the default initialize the value should be the user specified default value if it exists
    if let Ok(num_fields) = proc_macro2::TokenStream::from_str(&field_list.len().to_string()) {
        let tokens = quote! {
            pub fn __default_initialize() -> Box<#name> {
                Box::new( #name { #(#field_list: Default::default(), )* __num_fields: #num_fields, __set_fields: 0})
            }
        };
        tokens
    } else {
        proc_macro2::TokenStream::new()
    }

}

fn default_custom_initialize() -> proc_macro2::TokenStream {
    // the default custom initialize should also just be an empty placeholder
    generate_wrapper_s("__custom_initialize", "", None, true, true, false)
}

fn default_pre_execute() -> proc_macro2::TokenStream {
    // default pre-execute is an empty, static, placeholder function so that everything compiles
    generate_wrapper_s("__pre_execute", "", None, true, false, false)
}

fn default_on_execute() -> proc_macro2::TokenStream {
    // It is starting to feel more and more like I should get ride of the DeltaNode<T> nonsense, 
    // but at the same time is GUARANTEES I return something, even if that something is None, it seems like it would be better than returning ()
    // let tokens = quote! {
    //     // also if it is like this then it can be static, no need for `&mut self`
    //     pub fn __on_execute(&mut self) -> Option<i32> { // Have set return type to some sort of Option<...> if I want to return None, use Option<i32> as a placeholder b/c I'm not sure of better options
    //         None // default execution function should do nothing, but to satisfy DeltaNode generics need to return None
    //     }
    // };
    // tokens

    // basically the above, but is static and generated using a function. All of the above comments still apply though
    generate_wrapper_s("__on_execute", "Impulse::<i32>::NOOP", Some("Impulse<i32>"), true, true, false)
}

fn default_post_execute() -> proc_macro2::TokenStream {
    // default post execute resets all of registered fields 
    generate_wrapper_s("__post_execute", "__reset", None, true, true, true)
}

fn generate_wrapper(wrap_name: proc_macro2::TokenStream, func_name: proc_macro2::TokenStream, func_return: Option<proc_macro2::TokenStream>, public: bool, use_self: bool, call_func: bool) -> proc_macro2::TokenStream {
    // eww yuck!
    // get token to make function public if specified
    let ts_pub  = match proc_macro2::TokenStream::from_str(if public { "pub" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `pub` token! {:?}", e);
            proc_macro2::TokenStream::new()
        },
    };
    
    // get tokens to make wrapper have `&mut self` as arguments if use_self if true
    let ts_self_arg = match proc_macro2::TokenStream::from_str(if use_self { "&mut self" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `self` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // get tokens to make function call a call of `self.*`, only gets activated when using &mut self and calling a function (because it would be called with self.)
    let ts_self_call = match proc_macro2::TokenStream::from_str(if use_self && call_func { "self." } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `self` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // get tokens to actually call the function
    let ts_call = match proc_macro2::TokenStream::from_str(if call_func { "()" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `()` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // build the wrapping function
    let tokens = match func_return {
        Some(ret) => quote! {
            #ts_pub fn #wrap_name(#ts_self_arg) #ret {
                #ts_self_call #func_name #ts_call
            }
        },
        None => quote! {
            #ts_pub fn #wrap_name(#ts_self_arg) {
                #ts_self_call #func_name #ts_call;
            }
        },
    };

    tokens
}

// port of generate_wrapper, accepts &str inputs in place of tokenstreams
fn generate_wrapper_s(wrap_name: &str, func_name: &str, func_return: Option<&str>, public: bool, use_self: bool, call_func: bool) -> proc_macro2::TokenStream{
    // eww yuck!
    // get token to make function public if specified
    let ts_pub  = match proc_macro2::TokenStream::from_str(if public { "pub" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `pub` token! {:?}", e);
            proc_macro2::TokenStream::new()
        },
    };
    
    // get tokens to make wrapper have `&mut self` as arguments if use_self if true
    let ts_self_arg = match proc_macro2::TokenStream::from_str(if use_self { "&mut self" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `&mut self` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // get tokens to make function call a call of `self.*`, only gets activated when using &mut self and calling a function (because it would be called with self.)
    let ts_self_call = match proc_macro2::TokenStream::from_str(if use_self && call_func { "self." } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `self.` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // get tokens to actually call the function
    let ts_call = match proc_macro2::TokenStream::from_str(if call_func { "()" } else { "" }) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `()` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // convert the wrapper name to a token stream
    let ts_wrap_name = match proc_macro2::TokenStream::from_str(wrap_name) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `wrapper_name` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // convert the function name to a token stream
    let ts_func_name = match proc_macro2::TokenStream::from_str(func_name) {
        Ok(x) => x,
        Err(e) => {
            println!("Error generating wrapper when getting `func_name` tokens! {:?}", e);
            proc_macro2::TokenStream::new()
        }
    };

    // build the wrapping function
    let tokens = match func_return {
        Some(ret) => if let Ok(rt) = proc_macro2::TokenStream::from_str(ret) {
            quote! {
                #ts_pub fn #ts_wrap_name(#ts_self_arg) -> #rt {
                    #ts_self_call #ts_func_name #ts_call
                }
            }    
        } else {
            quote! { compile_error!("Invalid return type when generating wrapper") }
        },
        None => quote! {
            #ts_pub fn #ts_wrap_name(#ts_self_arg) {
                #ts_self_call #ts_func_name #ts_call;
            }
        },
    };

    tokens
}

fn is_outer_attribute(attr: &syn::Attribute) -> bool {
    match attr.style {
        // make sure it is an outer style
        syn::AttrStyle::Outer => true,
        _ => false,
    }
}

fn has_attribute<'a>(field: &'a syn::Field, attr_type: &str) -> Result<&'a syn::Attribute, bool> {
    for attr in &field.attrs {
        // check if the attribute is an Outer attribute (the first kind listed here https://docs.rs/syn/1.0.4/syn/struct.Attribute.html)
        if !is_outer_attribute(attr) {
            continue
        }

        if attr.path.is_ident(attr_type) {
            return Ok(&attr);
        }
    }
    Err(false)
}

fn is_public(field: &syn::Field) -> bool {
    match field.vis {
        syn::Visibility::Public(_) => true,
        _ => false,
    }
}

// there has to be a better way to do this then to use nested results, but idk how...
fn get_arg_literal(attr: &syn::Attribute, raise_error: bool, error_msg: &str) -> Result<syn::Lit, Result<proc_macro2::TokenStream, bool>>{
    if let Ok(meta) = attr.parse_meta() {
        match meta {
            syn::Meta::List(meta_list) => {
                if meta_list.nested.len() == 0 || meta_list.nested.len() > 1 {
                    return Err(Err(false));
                }

                return match &meta_list.nested[0] {
                    syn::NestedMeta::Meta(m) => { 
                        // TODO: if the raise_error flag is set then that means there must be a literal here or nothing else.
                        // if a meta is found then it will raise a compiler error to inform the user
                        // it will print out the specified error message
                        if raise_error {
                            let err = quote_spanned! {m.__span() => compile_error!(error_msg); };
                            Err(Ok(err))
                        } else {
                            Err(Err(false)) 
                        }
                    } ,
                    syn::NestedMeta::Lit(lit) => Ok(lit.clone()),
                }

            },
            _ => return Err(Err(false)), 
        }
    }
    Err(Err(false)) // hmm yes, it appears as though your error has produced an error... intriguing
}

fn delta_ignore(field: &syn::Field) -> Result<bool, proc_macro2::TokenStream> {
    // don't generate set or reset functions if the delta-ignore flag is set
    if let Ok(attr) = has_attribute(field, "delta_ignore") {
        // if we found an argument we will handle it. If no argument is found then assume default behavior
        match get_arg_literal(attr, true, "Argument for `delta_ignore` must be a bool.") {
            // we expect this argument to be a bool type
            Ok(arg) => {
                return match arg {
                    // if it is true then we ignore, if it is false then generate
                    // this will override the default behavior if a field is public
                    syn::Lit::Bool(b) => Ok(b.value),
                    // If it is any other kind of literal then we ignore it
                    _ =>  { 
                        let err = quote_spanned! {arg.span() => compile_error!("Argument for `delta_ignore` must be a bool."); };
                        return Err(err);
                    }
                }
            }, 
            Err(r) => {
                if let Ok(ts) = r { return Err(ts); } // Indicates there was a Meta instead of an expected literal
                return Ok(true); // If no arguments specified then the default is true
            }
        }
    }

    Ok(is_public(field)) // If there is no delta ignore attribute then we check if it is public value since they are ignored by default
}
// I think the way to go about this is as follows:
// in the struct macro add the needed attributes, also check for attributes with a [#delta_ignore] macro
// then for each attribute create a  set_[] function, that sets the variable and updates the __set_fields thing
// also create the initialize function. if the default is not specified [#delta_default(*default value*)] then it is just whatever the rust default is.
// also create a default reset() method, that resets all of the non-ignored values back to defaults

// then with the impl macro we can generate all of the functions (pre_execute, on_execute, post_execute). If they don't exist then create them.
// can also implement the DeltaNode<T> stuff using the info from on_execute...