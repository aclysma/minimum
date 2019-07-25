use rendy::util::types::vertex;

use vertex::AsVertex;

/// Type for position attribute of vertex.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position2(pub [f32; 2]);
impl<T> From<T> for Position2
where
    T: Into<[f32; 2]>,
{
    fn from(from: T) -> Self {
        Position2(from.into())
    }
}
impl vertex::AsAttribute for Position2 {
    const NAME: &'static str = "position2";
    const FORMAT: gfx_hal::format::Format = gfx_hal::format::Format::Rg32Sfloat;
}

/// Type for color attribute of vertex.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PackedColorU32(pub [u8; 4]);
impl<T> From<T> for PackedColorU32
where
    T: Into<[u8; 4]>,
{
    fn from(from: T) -> Self {
        PackedColorU32(from.into())
    }
}
impl vertex::AsAttribute for PackedColorU32 {
    const NAME: &'static str = "packed_color_u32";
    const FORMAT: gfx_hal::format::Format = gfx_hal::format::Format::Rgba8Unorm;
}

// Format of an imgui vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PosTexColor {
    /// Position of the vertex in 3D space.
    pub position: Position2,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: vertex::TexCoord,
    /// RGBA color value of the vertex.
    pub color: PackedColorU32,
}

#[cfg(not(feature = "spirv-reflection"))]
impl AsVertex for PosTexColor {
    fn vertex() -> vertex::VertexFormat {
        vertex::VertexFormat::new((
            Position2::vertex(),
            vertex::TexCoord::vertex(),
            PackedColorU32::vertex(),
        ))
    }
}
