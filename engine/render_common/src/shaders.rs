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

impl From<ShaderType> for gl::types::GLenum {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

impl From<&ShaderType> for gl::types::GLenum {
    fn from(value: &ShaderType) -> Self {
        match *value {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

impl std::fmt::Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Vertex => "Vertex",
                Self::Fragment => "Fragment",
                Self::Geometry => "Geometry",
            }
        )
    }
}

pub trait ProgramInternal {
    fn vertex() -> &'static str;
    fn fragment() -> &'static str;
    fn geometry() -> Option<&'static str>;
}
