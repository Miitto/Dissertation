use std::fmt::{Display, Formatter};

pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
}
impl TryFrom<&str> for ShaderType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "vert" | "vertex" => Self::Vertex,
            "frag" | "fragment" => Self::Fragment,
            "geom" | "geometry" => Self::Geometry,
            _ => return Err(format!("Invalid Shader Type {}", value)),
        })
    }
}

impl TryFrom<String> for ShaderType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Vertex => "Vertex",
            Self::Fragment => "Fragment",
            Self::Geometry => "Geometry",
        })
    }
}
