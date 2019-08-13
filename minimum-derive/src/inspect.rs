
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, Fields, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
struct InspectorArgs {
    ident: syn::Ident,
}

mod keyword {
    syn::custom_keyword!(inspector);
}

impl Parse for InspectorArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {

        let content;
        let _parens = syn::parenthesized!(content in input);

        content.parse::<keyword::inspector>()?;
        content.parse::<Token![=]>()?;
        let ident: syn::Ident = content.parse()?;

        Ok(InspectorArgs {
            ident,
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

pub fn impl_inspect_macro(ast: &syn::DeriveInput) -> TokenStream {

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let struct_name1 = &ast.ident;
    let struct_name2 = &ast.ident;
    let struct_name3 = &ast.ident;

    let field_name1 = fields.iter().map(|field| &field.ident);
    let field_name2 = fields.iter().map(|field| &field.ident);
    let field_name3 = fields.iter().map(|field| &field.ident);
    let field_name4 = fields.iter().map(|field| &field.ident);
    let field_type1 = fields.iter().map(|field| &field.ty);
    let field_type2 = fields.iter().map(|field| &field.ty);

    let mut inspector_names = vec![];

    for field in fields {
        let mut inspector_name : Option<InspectorArgs> = None;

        for attr in field.attrs.iter().filter(|x| x.path.is_ident("inspect")) {
            let args = syn::parse2::<InspectorArgs>(attr.tts.clone());
            let args = match args {
                Ok(data) => data,
                Err(err) => {
                    return TokenStream::from(err.to_compile_error());
                }
            };
            inspector_name = Some(args);
        }

        inspector_names.push(inspector_name.unwrap_or_else(|| InspectorArgs { ident: syn::Ident::new("InspectRenderDefault", proc_macro2::Span::call_site()) }));
    }

    let inspector_name1 = inspector_names.iter().map(|x| x.ident.clone());
    let inspector_name2 = inspector_names.iter().map(|x| x.ident.clone());

    TokenStream::from(quote! {

        impl InspectRenderDefault for #struct_name1 {
            fn render(&self, label: &'static str, ui: &imgui::Ui) {
                let header_name = stringify!(#struct_name2);
                let header = ui.collapsing_header(&imgui::im_str!( "{}", header_name      )).build();
                ui.indent();
                #(
                    <#field_type1 as #inspector_name1>::render(&self.#field_name1, stringify!(#field_name2), ui);
                )*
                ui.unindent();
            }

            fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
                let header_name = stringify!(#struct_name3);
                let header = ui.collapsing_header(&imgui::im_str!("{}", header_name)).build();
                ui.indent();
                #(
                    <#field_type2 as #inspector_name2>::render_mut(&mut self.#field_name3, stringify!(#field_name4), ui);
                )*
                ui.unindent();
            }
        }
    })
}
