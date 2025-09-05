use syn::{Error, Expr, Field, Lit, Result as SynResult, Type, TypePath};

#[derive(Debug, Clone)]
pub enum ControlType {
    Range { min: f32, max: f32 },
    IntRange { min: i32, max: i32 },
    Toggle,
    VectorRange { components: Vec<(u8, f32, f32)> }, // Vec of (cc, min, max) for each component
    LinearRgba,                                      // bevy::color::LinearRgba
    Srgba,                                           // bevy::color::Srgba
    Hsla,                                            // bevy::color::Hsla
    Hsva,                                            // bevy::color::Hsva
}

#[derive(Debug, Clone)]
pub enum RangeSpec {
    Float(f32, f32),
    Int(i32, i32),
}

#[derive(Debug, Clone)]
pub struct MidiMapping {
    pub cc: u8,
    pub range: Option<RangeSpec>,
}

/// Parse all #[midi(...)] attributes from a field
pub fn parse_midi_attributes(field: &Field) -> SynResult<Vec<MidiMapping>> {
    let mut mappings = Vec::new();

    for attr in &field.attrs {
        if attr.path().is_ident("midi") {
            let mapping = attr.parse_args_with(|input: syn::parse::ParseStream| {
                // Parse CC number
                let cc: syn::LitInt = input.parse()?;
                let cc_value = cc.base10_parse::<u8>()?;

                // Check if there's a range specification
                let range = if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;

                    // Parse range expression
                    let expr: Expr = input.parse()?;
                    Some(parse_range_expr(&expr)?)
                } else {
                    None
                };

                Ok(MidiMapping {
                    cc: cc_value,
                    range,
                })
            })?;

            mappings.push(mapping);
        }
    }

    Ok(mappings)
}

/// Determine the control type based on the field type and mappings
pub fn determine_control_type(ty: &Type, mappings: &[MidiMapping]) -> SynResult<ControlType> {
    let type_name = get_type_name(ty);

    // Check for Bevy color types
    if let Some(ref name) = type_name {
        match name.as_str() {
            "LinearRgba" => {
                if mappings.len() != 4 {
                    return Err(Error::new_spanned(
                        ty,
                        "LinearRgba requires exactly 4 #[midi] attributes (r, g, b, a)",
                    ));
                }
                return Ok(ControlType::LinearRgba);
            }
            "Srgba" => {
                if mappings.len() != 4 {
                    return Err(Error::new_spanned(
                        ty,
                        "Srgba requires exactly 4 #[midi] attributes (r, g, b, a)",
                    ));
                }
                return Ok(ControlType::Srgba);
            }
            "Hsla" => {
                if mappings.len() != 4 {
                    return Err(Error::new_spanned(
                        ty,
                        "Hsla requires exactly 4 #[midi] attributes (h, s, l, a)",
                    ));
                }
                return Ok(ControlType::Hsla);
            }
            "Hsva" => {
                if mappings.len() != 4 {
                    return Err(Error::new_spanned(
                        ty,
                        "Hsva requires exactly 4 #[midi] attributes (h, s, v, a)",
                    ));
                }
                return Ok(ControlType::Hsva);
            }
            _ => {}
        }
    }

    // Check if type is a vector type (Vec2, Vec3, Vec4)
    if let Some(vec_dim) = get_vector_dimension(ty) {
        if mappings.len() != vec_dim {
            return Err(Error::new_spanned(
                ty,
                format!(
                    "Vector type {} requires exactly {} #[midi] attributes",
                    type_name.unwrap_or_else(|| "Vec".to_string()),
                    vec_dim
                ),
            ));
        }

        let mut components = Vec::new();
        for mapping in mappings {
            let (min, max) = if let Some(ref range) = mapping.range {
                match range {
                    RangeSpec::Float(min, max) => (*min, *max),
                    RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                }
            } else {
                (0.0, 1.0) // Default range
            };
            components.push((mapping.cc, min, max));
        }

        return Ok(ControlType::VectorRange { components });
    }

    // Single mapping expected for scalar types
    if mappings.len() != 1 {
        return Err(Error::new_spanned(
            ty,
            "Scalar types require exactly one #[midi] attribute",
        ));
    }

    let mapping = &mappings[0];

    // Check if type is bool
    if is_bool_type(ty) {
        return Ok(ControlType::Toggle);
    }

    // Check if type is a numeric type
    if let Some(numeric_type) = get_numeric_type(ty) {
        match numeric_type {
            NumericType::Float => {
                let (min, max) = if let Some(ref range) = mapping.range {
                    match range {
                        RangeSpec::Float(min, max) => (*min, *max),
                        RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                    }
                } else {
                    (0.0, 1.0) // Default range for floats
                };
                Ok(ControlType::Range { min, max })
            }
            NumericType::SignedInt => {
                let (min, max) = if let Some(ref range) = mapping.range {
                    match range {
                        RangeSpec::Int(min, max) => (*min, *max),
                        RangeSpec::Float(min, max) => (*min as i32, *max as i32),
                    }
                } else {
                    (0, 127) // Default MIDI range
                };
                Ok(ControlType::IntRange { min, max })
            }
        }
    } else {
        Err(Error::new_spanned(
            ty,
            "MIDI parameters only support bool, f32, f64, i32, vector types (Vec2, Vec3, Vec4), and Bevy color types (LinearRgba, Srgba, Hsla, Hsva)",
        ))
    }
}

fn parse_range_expr(expr: &Expr) -> SynResult<RangeSpec> {
    // Parse range expressions like 0.0..1.0 or -1.0..1.0 or 0..127
    if let Expr::Range(range) = expr {
        let start = range
            .start
            .as_ref()
            .ok_or_else(|| Error::new_spanned(expr, "Range must have a start value"))?;
        let end = range
            .end
            .as_ref()
            .ok_or_else(|| Error::new_spanned(expr, "Range must have an end value"))?;

        // Try parsing as integers first
        if let (Some(min), Some(max)) = (parse_int_expr(start), parse_int_expr(end)) {
            return Ok(RangeSpec::Int(min, max));
        }

        // Otherwise parse as floats
        let min = parse_float_expr(start)
            .ok_or_else(|| Error::new_spanned(start, "Invalid range start value"))?;
        let max = parse_float_expr(end)
            .ok_or_else(|| Error::new_spanned(end, "Invalid range end value"))?;

        Ok(RangeSpec::Float(min, max))
    } else {
        Err(Error::new_spanned(
            expr,
            "Expected range expression (e.g., 0.0..1.0 or 0..127)",
        ))
    }
}

fn parse_float_expr(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(lit) => {
            if let Lit::Float(f) = &lit.lit {
                f.base10_parse().ok()
            } else if let Lit::Int(i) = &lit.lit {
                i.base10_parse::<i32>().ok().map(|v| v as f32)
            } else {
                None
            }
        }
        Expr::Unary(unary) => {
            if let syn::UnOp::Neg(_) = unary.op {
                parse_float_expr(&unary.expr).map(|v| -v)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_int_expr(expr: &Expr) -> Option<i32> {
    match expr {
        Expr::Lit(lit) => {
            if let Lit::Int(i) = &lit.lit {
                i.base10_parse().ok()
            } else {
                None
            }
        }
        Expr::Unary(unary) => {
            if let syn::UnOp::Neg(_) = unary.op {
                parse_int_expr(&unary.expr).map(|v| -v)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "bool";
        }
    }
    false
}

fn get_type_name(ty: &Type) -> Option<String> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Some(segment.ident.to_string());
        }
    }
    None
}

fn get_vector_dimension(ty: &Type) -> Option<usize> {
    get_type_name(ty).and_then(|name| match name.as_str() {
        "Vec2" => Some(2),
        "Vec3" => Some(3),
        "Vec4" => Some(4),
        _ => None,
    })
}

#[derive(Debug, Clone, Copy)]
enum NumericType {
    Float,
    SignedInt,
}

fn get_numeric_type(ty: &Type) -> Option<NumericType> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            let ident_str = segment.ident.to_string();
            match ident_str.as_str() {
                "f32" | "f64" => Some(NumericType::Float),
                "i32" => Some(NumericType::SignedInt),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}
