use rendy::shader::{PathBufShaderInfo, ShaderKind, SourceLanguage};

use std::path::PathBuf;

lazy_static::lazy_static! {

    // IMGUI
    pub static ref IMGUI_VERTEX: PathBufShaderInfo = PathBufShaderInfo::new(
        //concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fx/imgui.vert"),
        PathBuf::from("assets/imgui.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    );

    pub static ref IMGUI_FRAGMENT: PathBufShaderInfo = PathBufShaderInfo::new(
        //concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fx/imgui.frag"),
        PathBuf::from("assets/imgui.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    );

    pub static ref IMGUI_SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*IMGUI_VERTEX).unwrap()
        .with_fragment(&*IMGUI_FRAGMENT).unwrap();
}

#[cfg(feature = "spirv-reflection")]
lazy_static::lazy_static! {
    pub static ref IMGUI_SHADER_REFLECTION: SpirvReflection = IMGUI_SHADERS.reflect().unwrap();
}
