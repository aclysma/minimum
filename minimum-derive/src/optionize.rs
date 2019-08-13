
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, Fields, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
struct OptionizedTypeArgs {
    t: syn::Type,
}

mod keyword {
    syn::custom_keyword!(optionized_type);
}

impl Parse for OptionizedTypeArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {

        let content;
        let _parens = syn::parenthesized!(content in input);

        content.parse::<keyword::optionized_type>()?;
        content.parse::<Token![=]>()?;
        let t: syn::Type = content.parse()?;

        Ok(OptionizedTypeArgs {
            t,
        })
    }
}

/*
EXAMPLE INPUT:

    #[derive(minimum_derive::Inspect, minimum_derive::Optionize)]
    pub struct MyStruct {
        pub a: f32,
        pub b: f32,
        pub c: glm::Vec2,
        pub d: glm::Vec3
    }

    #[derive(minimum_derive::Inspect, minimum_derive::Optionize)]
    pub struct MyStruct2 {

        #[inspect(inspector = InspectRenderAsSlider)]
        pub a: f32,
        pub b: f32,
        pub c: glm::Vec2,
        pub d: glm::Vec3,
        #[optionize(optionized_type = MyStructOptionized)]
        pub s: MyStruct
    }

EXAMPLE OUTPUT:

    impl OptionizedMember<MyStruct> for MyStructOptionized {
        fn empty() -> Self {
            Self {
                a: DefaultOptionized::empty(),
                b: DefaultOptionized::empty(),
                c: DefaultOptionized::empty(),
                d: DefaultOptionized::empty(),
            }
        }

        fn assign_to_optionized(dest: &mut MyStructOptionized, source: &MyStruct) {
            OptionizedMember::assign_to_optionized(&mut dest.a, &source.a);
            OptionizedMember::assign_to_optionized(&mut dest.b, &source.b);
            OptionizedMember::assign_to_optionized(&mut dest.c, &source.c);
            OptionizedMember::assign_to_optionized(&mut dest.d, &source.d);
        }

        fn merge_to_optionized(dest: &mut MyStructOptionized, source: &MyStruct) {
            OptionizedMember::merge_to_optionized(&mut dest.a, &source.a);
            OptionizedMember::merge_to_optionized(&mut dest.b, &source.b);
            OptionizedMember::merge_to_optionized(&mut dest.c, &source.c);
            OptionizedMember::merge_to_optionized(&mut dest.d, &source.d);
        }
    }

    impl OptionizedMember<MyStruct2> for MyStruct2Optionized {
        fn empty() -> Self {
            Self {
                a: DefaultOptionized::empty(),
                b: DefaultOptionized::empty(),
                c: DefaultOptionized::empty(),
                d: DefaultOptionized::empty(),
                s: MyStructOptionized::empty(),
            }
        }

        fn assign_to_optionized(dest: &mut MyStruct2Optionized, source: &MyStruct2) {
            OptionizedMember::assign_to_optionized(&mut dest.a, &source.a);
            OptionizedMember::assign_to_optionized(&mut dest.b, &source.b);
            OptionizedMember::assign_to_optionized(&mut dest.c, &source.c);
            OptionizedMember::assign_to_optionized(&mut dest.d, &source.d);
            OptionizedMember::assign_to_optionized(&mut dest.s, &source.s);
        }

        fn merge_to_optionized(dest: &mut MyStruct2Optionized, source: &MyStruct2) {
            OptionizedMember::merge_to_optionized(&mut dest.a, &source.a);
            OptionizedMember::merge_to_optionized(&mut dest.b, &source.b);
            OptionizedMember::merge_to_optionized(&mut dest.c, &source.c);
            OptionizedMember::merge_to_optionized(&mut dest.d, &source.d);
            OptionizedMember::merge_to_optionized(&mut dest.s, &source.s);
        }
    }
*/


pub fn impl_optionize_macro(ast: &syn::DeriveInput) -> TokenStream {

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    //TODO: Take this as a parameter instead of assuming a name
    let optionized_struct_name = format!("{}Optionized", &ast.ident);
    let optionized_struct_name = syn::Ident::new(&optionized_struct_name, proc_macro2::Span::call_site());

    let mut optionized_field_types = vec![];

    for field in fields {
        let mut optionized_field_type : Option<OptionizedTypeArgs> = None;
        let field_type = field.ty.clone();

        for attr in field.attrs.iter().filter(|x| x.path.is_ident("optionize")) {
            let args = syn::parse2::<OptionizedTypeArgs>(attr.tts.clone());
            let args = match args {
                Ok(data) => data,
                Err(err) => {
                    return TokenStream::from(err.to_compile_error());
                }
            };
            optionized_field_type = Some(args);
        }

        // Use DefaultOptionized<_> or whatever was provided via #[optionize(optionized_type = _)]
        optionized_field_types.push(optionized_field_type.unwrap_or_else(|| {
            let t = quote!(DefaultOptionized<#field_type>);
            let t = syn::parse2::<syn::Type>(t).unwrap();

            OptionizedTypeArgs {
                t
            }
        }));
    }

    let struct_name1 = &ast.ident;
    let struct_name2 = &ast.ident;
    let struct_name3 = &ast.ident;

    let optionized_struct_name1 = optionized_struct_name.clone();
    let optionized_struct_name2 = optionized_struct_name.clone();
    let optionized_struct_name3 = optionized_struct_name.clone();
    let optionized_struct_name4 = optionized_struct_name.clone();

    let field_name1 = fields.iter().map(|field| &field.ident);
    let field_name2 = fields.iter().map(|field| &field.ident);
    let field_name3 = fields.iter().map(|field| &field.ident);
    let field_name4 = fields.iter().map(|field| &field.ident);
    let field_name5 = fields.iter().map(|field| &field.ident);
    let field_name6 = fields.iter().map(|field| &field.ident);

    let optionized_field_types1 = optionized_field_types.iter().map(|x| x.t.clone());
    let optionized_field_types2 = optionized_field_types.iter().map(|x| x.t.clone());

    let result = TokenStream::from(quote! {

        #[derive(minimum_derive::Inspect)]
        pub struct #optionized_struct_name1 {
            #(
                #field_name1: #optionized_field_types1,
            )*
        }

        impl OptionizedMember<#struct_name1> for #optionized_struct_name2 {
            fn empty() -> Self {
                Self {
                    // List every field, initializing them with <FieldType>::empty()
                    #(
                        #field_name2: <#optionized_field_types2>::empty(),
                    )*
                }
            }

            fn assign_to_optionized(dest: &mut #optionized_struct_name3, source: &#struct_name2) {
                // Blindly call OptionizedMember::assign_to_optionized for every field
                #(
                    OptionizedMember::assign_to_optionized(&mut dest.#field_name3, &source.#field_name4);
                )*
            }

            fn merge_to_optionized(dest: &mut #optionized_struct_name4, source: &#struct_name3) {
                // Blindly call OptionizedMember::merge_to_optionized for every field
                #(
                    OptionizedMember::merge_to_optionized(&mut dest.#field_name5, &source.#field_name6);
                )*
            }
        }
    });

    result
}
