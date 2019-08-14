
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, Fields, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
struct FieldArgs {
    inspector: Option<syn::Ident>,
    wrapping_type: Option<syn::Ident>
}

mod keyword {
    syn::custom_keyword!(inspector);
}

struct ParsedField {
    name: syn::Ident,
    ty: syn::Type,
    inspector: syn::Type,
    wrapping_type: Option<syn::Type>
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

EXAMPLE OUTPUT:

    impl InspectRenderDefault for MyStruct {
        fn render(&self, _label: &'static str, ui: &imgui::Ui) {
            let header = ui.collapsing_header(imgui::im_str!("MyStruct")).build();
            ui.indent();
            InspectRenderDefault::render(&self.a, "a", ui);
            InspectRenderAsSlider::render(&self.b, "b", ui);
            InspectRenderDefault::render(&self.c, "c", ui);
            InspectRenderDefault::render(&self.d, "d", ui);
            ui.unindent();
        }

        fn render_mut(&mut self, _label: &'static str, ui: &imgui::Ui) {
            let header = ui.collapsing_header(imgui::im_str!("MyStruct")).build();
            ui.indent();
            InspectRenderDefault::render_mut(&mut self.a, "a", ui);
            InspectRenderAsSlider::render_mut(&mut self.b, "b", ui);
            InspectRenderDefault::render_mut(&mut self.c, "c", ui);
            InspectRenderDefault::render_mut(&mut self.d, "d", ui);
            ui.unindent();
        }
    }
*/

fn get_lit_str<'a>(lit: &'a syn::Lit) -> Result<&'a syn::LitStr, ()> {
    get_lit_str2( lit)
}

fn get_lit_str2<'a>(
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        Err(())
    }
}

pub fn impl_inspect_macro(ast: &syn::DeriveInput) -> TokenStream {

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let mut parsed_fields = vec![];


    for field in fields {

        let mut inspector : Option<syn::Type> = None;
        let mut wrapping_type : Option<syn::Type> = None;

        for attr in field.attrs.iter().filter(|x| x.path.is_ident("inspect")) {

            let parsed_meta = attr.parse_meta();
            if let Ok(syn::Meta::List(ref meta)) = parsed_meta {
                let metas : Vec<_> = meta.nested.iter().cloned().collect();
                for meta in metas {
                    match meta {
                        syn::NestedMeta::Meta(syn::Meta::NameValue(ref m)) if &m.ident.to_string() == "inspector" => {
                            use quote::ToTokens;
                            let str = get_lit_str(&m.lit).expect("could not convert inspector value to ListStr");
                            let ty = str.parse();
                            if ty.is_err() {
                                println!("ERROR");
                                return TokenStream::from(quote!(compile_error!(format!("{:?}", ty.err);)));
                            }

                            if inspector.is_some() {
                                return TokenStream::from(quote!(compile_error!("inspector specified more than once!")));
                            }

                            inspector = Some(ty.unwrap());
                        },
                        syn::NestedMeta::Meta(syn::Meta::NameValue(ref m)) if &m.ident.to_string() == "wrapping_type" => {
                            use quote::ToTokens;
                            let str = get_lit_str(&m.lit).unwrap();
                            let ty = str.parse();
                            if ty.is_err() {
                                return TokenStream::from(quote!(compile_error!(format!("{:?}", ty.err);)));
                            }

                            if wrapping_type.is_some() {
                                return TokenStream::from(quote!(compile_error!("wrapping_type specified more than once!")));
                            }

                            wrapping_type = Some(ty.unwrap());
                        },
                        _ => {
                            return TokenStream::from(quote!(compile_error!("Unrecognized metadata in inspect field attribute");));
                        }
                    }
                }
            }
        }

        // Inspector must be non-null, default to "InspectRenderDefault"
        let inspector = inspector.unwrap_or_else(||
            syn::parse2::<syn::Type>(quote!(InspectRenderDefault)).unwrap()
        );

        parsed_fields.push(ParsedField {
            name: field.ident.as_ref().unwrap().clone(),
            ty: field.ty.clone(),
            inspector,
            wrapping_type

        });
    }

    let struct_name1 = &ast.ident;
    let struct_name2 = &ast.ident;
    let struct_name3 = &ast.ident;
    let struct_name4 = &ast.ident;

    let mut render_impls = vec![];
    let mut render_mut_impls = vec![];

    for parsed_field in &parsed_fields {
        let render = if let Some(value) = &parsed_field.wrapping_type {
            let inspector = &parsed_field.inspector;
            let field_name1 = &parsed_field.name;
            let field_name2 = &parsed_field.name;
            let field_name3 = &parsed_field.name;
            let field_type = &parsed_field.ty;
            let wrapping_type1 = &parsed_field.wrapping_type;
            let wrapping_type2 = &parsed_field.wrapping_type;
            quote! {
                <#wrapping_type2 as #inspector<#field_type>>::render(&[&data[0].#field_name1], stringify!(#field_name3), ui);
            }
        } else {
            let inspector = &parsed_field.inspector;
            let field_type1 = &parsed_field.ty;
            let field_type2 = &parsed_field.ty;
            let field_name1 = &parsed_field.name;
            let field_name2 = &parsed_field.name;

            quote! {
                <#field_type1 as #inspector<#field_type2>>::render(&[&data[0].#field_name1], stringify!(#field_name2), ui);
            }
        };

        let render_mut = if let Some(value) = &parsed_field.wrapping_type {
            let inspector = &parsed_field.inspector;
            let field_name1 = &parsed_field.name;
            let field_name2 = &parsed_field.name;
            let field_name3 = &parsed_field.name;
            let field_type = &parsed_field.ty;
            let wrapping_type1 = &parsed_field.wrapping_type;
            let wrapping_type2 = &parsed_field.wrapping_type;
            quote! {
                let mut values : Vec<_> = data.iter_mut().map(|x| &mut x.#field_name1).collect();
                <#wrapping_type2 as #inspector<#field_type>>::render_mut(&mut values.as_mut_slice(), stringify!(#field_name3), ui);
            }
        } else {
            let inspector = &parsed_field.inspector;
            let field_type1 = &parsed_field.ty;
            let field_type2 = &parsed_field.ty;
            let field_name1 = &parsed_field.name;
            let field_name2 = &parsed_field.name;

            quote! {
                let mut values : Vec<_> = data.iter_mut().map(|x| &mut x.#field_name1).collect();
                <#field_type1 as #inspector<#field_type2>>::render_mut(&mut values.as_mut_slice(), stringify!(#field_name2), ui);
            }
        };

        render_impls.push(render);
        render_mut_impls.push(render_mut);
    }

    TokenStream::from(quote! {

        impl InspectRenderDefault<#struct_name1> for #struct_name2 {
            fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui) {
                let header_name = stringify!(#struct_name3);
                let header = ui.collapsing_header(&imgui::im_str!( "{}", header_name)).build();
                ui.indent();
                #(
                    #render_impls
                )*
                ui.unindent();
            }

            fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui) {
                let header_name = stringify!(#struct_name4);
                let header = ui.collapsing_header(&imgui::im_str!("{}", header_name)).build();
                ui.indent();
                #(
                    #render_mut_impls
                )*
                ui.unindent();
            }
        }
    })
}
