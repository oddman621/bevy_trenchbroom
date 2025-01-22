use class::QuakeClassPropertyType;

use crate::*;

impl TrenchBroomConfig {
    /// Converts this config to a string for writing `fgd` (entity definition) files.
    pub fn to_fgd(&self) -> String {
        use fmt::Write;
        let mut s = String::new();
        macro_rules! write {($($arg:tt)*) => {
            s.write_fmt(format_args!($($arg)*)).ok()
        };}

        // TODO don't write hanging base classes
        for class in self.class_iter() {
            write!("@{:?}Class ", class.info.ty);
    
            if !class.info.base.is_empty() {
                write!("base({}) ", class.info.base.iter().map(|base| base.info.name).join(", "));
            }
    
            if let Some(value) = class.info.color {
                write!("color({value})");
            }
            if let Some(value) = class.info.iconsprite {
                write!("iconsprite({value})");
            }
            if let Some(value) = class.info.size {
                write!("size({value})");
            }
            if let Some(value) = class.info.model {
                write!("model({value})");
            }
    
            write!("= {}", class.info.name);
            if let Some(description) = class.info.description {
                write!(" : \"{description}\"");
            }
            write!("\n[\n");

            for property in class.info.properties {
                write!("\t{}({}): \"{}\" : {} : \"{}\"",
                    property.name,
                    match &property.ty {
                        QuakeClassPropertyType::Value(ty) => ty,
                        QuakeClassPropertyType::Choices(_) => "choices",
                    },
                    property.title.unwrap_or(property.name),
                    {
                        let mut s = property.default_value.unwrap_or(String::new)();
                        s.insert(0, '\"'); s.push('\"');
                        s
                    },
                    property.description.unwrap_or_default(),
                );

                if let QuakeClassPropertyType::Choices(choices) = property.ty {
                    write!(" = \n\t[\n");
                    for (key, title) in choices {
                        write!("\t\t{key} : \"{title}\"\n");
                    }
                    write!("\t]");
                }

                write!("\n");
            }

            write!("]\n\n");
        }
        
        s
    }
}

/// Contains Quake/TrenchBroom-specific parsing and stringification functions.
pub trait FgdType: Sized {
    /// If quotes should be put around this value when writing out an FGD file.
    const FGD_IS_QUOTED: bool = true;
    const PROPERTY_TYPE: QuakeClassPropertyType;

    /// Parses a string into `Self` FGD-style. Used for parsing entity properties.
    fn fgd_parse(input: &str) -> anyhow::Result<Self>; // TODO do we want to keep anyhow?
    /// Converts this value into a string used for writing FGDs.
    fn fgd_to_string(&self) -> String;
    /// Calls `fgd_to_string`, but if `FGD_IS_QUOTED` is true, surrounds the output with quotes.
    fn fgd_to_string_quoted(&self) -> String {
        if Self::FGD_IS_QUOTED {
            format!("\"{}\"", self.fgd_to_string())
        } else {
            self.fgd_to_string()
        }
    }
}

impl FgdType for String {
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("string");
    
    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        Ok(input.to_string())
    }
    fn fgd_to_string(&self) -> String {
        self.clone()
    }
}

impl FgdType for &str {
    const PROPERTY_TYPE: QuakeClassPropertyType = String::PROPERTY_TYPE;
    
    fn fgd_parse(_input: &str) -> anyhow::Result<Self> {
        // Lifetimes don't allow me to just return Some(input) unfortunately.
        unimplemented!("use String::fgd_parse instead");
    }
    fn fgd_to_string(&self) -> String {
        self.to_string()
    }
}

macro_rules! simple_fgd_type_impl {
    ($ty:ty, $quoted:expr, $fgd_type:ident $fgd_type_value:expr) => {
        impl FgdType for $ty {
            const FGD_IS_QUOTED: bool = $quoted;
            const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::$fgd_type($fgd_type_value);

            fn fgd_parse(input: &str) -> anyhow::Result<Self> {
                Ok(input.parse()?)
            }
            fn fgd_to_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

simple_fgd_type_impl!(u8, false, Value "integer");
simple_fgd_type_impl!(u16, false, Value "integer");
simple_fgd_type_impl!(u32, false, Value "integer");
simple_fgd_type_impl!(u64, false, Value "integer");
simple_fgd_type_impl!(usize, false, Value "integer");
simple_fgd_type_impl!(i8, false, Value "integer");
simple_fgd_type_impl!(i16, false, Value "integer");
simple_fgd_type_impl!(i32, false, Value "integer");
simple_fgd_type_impl!(i64, false, Value "integer");
simple_fgd_type_impl!(isize, false, Value "integer");

simple_fgd_type_impl!(bool, true, Choices &[("\"true\"", "true"), ("\"false\"", "false")]);

simple_fgd_type_impl!(f32, true, Value "float");
simple_fgd_type_impl!(f64, true, Value "float");

impl FgdType for Aabb {
    const FGD_IS_QUOTED: bool = false;
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("aabb");

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        let values = <[f32; 6]>::fgd_parse(input)?;
        Ok(Aabb::from_min_max(
            Vec3::from_slice(&values[0..=3]),
            Vec3::from_slice(&values[3..=6]),
        ))
    }
    fn fgd_to_string(&self) -> String {
        let min = self.min();
        let max = self.max();
        format!(
            "{} {} {}, {} {} {}",
            min.x, min.y, min.z, max.x, max.y, max.z
        )
    }
}

impl FgdType for Vec4 {
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("vec4");

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        <[f32; 4]>::fgd_parse(input).map(Vec4::from)
    }
    fn fgd_to_string(&self) -> String {
        format!("{} {} {} {}", self.x, self.y, self.z, self.w)
    }
}
impl FgdType for Vec3 {
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("vector");

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        <[f32; 3]>::fgd_parse(input).map(Vec3::from)
    }
    fn fgd_to_string(&self) -> String {
        format!("{} {} {}", self.x, self.y, self.z)
    }
}
impl FgdType for Vec2 {
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("vec2");

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        <[f32; 2]>::fgd_parse(input).map(Vec2::from)
    }
    fn fgd_to_string(&self) -> String {
        format!("{} {}", self.x, self.y)
    }
}

impl FgdType for Color {
    const PROPERTY_TYPE: QuakeClassPropertyType = QuakeClassPropertyType::Value("color1");

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        <[f32; 3]>::fgd_parse(input)
            .map(Color::srgb_from_array)
            .or(<[f32; 4]>::fgd_parse(input).map(|[r, g, b, a]| Color::srgba(r, g, b, a)))
    }
    fn fgd_to_string(&self) -> String {
        let col = self.to_srgba();
        format!("{} {} {} {}", col.red, col.green, col.blue, col.alpha)
    }
}

// God i love rust's trait system
impl<T: FgdType + Default + Copy, const N: usize> FgdType for [T; N] {
    const PROPERTY_TYPE: QuakeClassPropertyType = T::PROPERTY_TYPE;

    fn fgd_parse(input: &str) -> anyhow::Result<Self> {
        // This might be a problem for FgdTypes that use spaces in their parsing. Oh well!
        let mut out = [T::default(); N];

        for (i, input) in input.split_ascii_whitespace().enumerate() {
            if i >= out.len() {
                return Err(anyhow::anyhow!("Too many elements! Expected: {N}"));
            }
            out[i] = T::fgd_parse(input)?;
        }

        Ok(out)
    }
    fn fgd_to_string(&self) -> String {
        self.iter().map(T::fgd_to_string).join(" ")
    }
}
