use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Error, Expr, Field, Fields, Lit, Meta, Path,
    Result as SynResult, Token,
};

/// Derive macro for MIDI parameter mapping
#[proc_macro_derive(MidiParams, attributes(midi))]
pub fn derive_midi_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_midi_params(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_midi_params(input: &DeriveInput) -> SynResult<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "MidiParams only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                name,
                "MidiParams can only be derived for structs",
            ))
        }
    };

    let mut midi_mappings = Vec::new();
    let mut midi_updates = Vec::new();
    let mut ui_controls = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        if let Some(midi_attr) = parse_midi_attribute(field)? {
            let (cc, control_type) = midi_attr;

            match control_type {
                ControlType::Range { min, max } => {
                    // MIDI mapping
                    midi_mappings.push(quote! {
                        bevy_midi_params::MidiMapping::range(#cc, #field_name_str, #min, #max)
                    });

                    // MIDI update logic
                    midi_updates.push(quote! {
                        #cc => {
                            let new_value = #min + value * (#max - #min);
                            if (self.#field_name - new_value).abs() > f32::EPSILON {
                                self.#field_name = new_value;
                                changed = true;
                            }
                        }
                    });

                    // UI control
                    let display_name = field_name_str.replace('_', " ");
                    ui_controls.push(quote! {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} (CC{}):", #display_name, #cc));
                            ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name, #min..=#max)
                                .text(format!("{:.3}", self.#field_name))).changed();
                        });
                    });
                }
                ControlType::Button => {
                    // MIDI mapping
                    midi_mappings.push(quote! {
                        bevy_midi_params::MidiMapping::button(#cc, #field_name_str)
                    });

                    // MIDI update logic
                    midi_updates.push(quote! {
                        #cc => {
                            if value > 0.5 {
                                self.#field_name = !self.#field_name;
                                changed = true;
                            }
                        }
                    });

                    // UI control
                    let display_name = field_name_str.replace('_', " ");
                    ui_controls.push(quote! {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} (CC{}):", #display_name, #cc));
                            ui_changed |= ui.checkbox(&mut self.#field_name, "").changed();
                        });
                    });
                }
            }
        }
    }

    let type_name_str = name.to_string();

    let expanded = quote! {
        impl #impl_generics bevy_midi_params::MidiControllable for #name #ty_generics #where_clause {
            fn update_from_midi(&mut self, cc: u8, value: f32) -> bool {
                let mut changed = false;
                match cc {
                    #(#midi_updates)*
                    _ => {}
                }
                changed
            }

            fn get_midi_mappings() -> Vec<bevy_midi_params::MidiMapping> {
                vec![#(#midi_mappings),*]
            }

            #[cfg(feature = "ui")]
            fn render_ui(&mut self, ui: &mut egui::Ui) -> bool {
                let mut ui_changed = false;
                ui.heading(#type_name_str);
                ui.separator();
                #(#ui_controls)*
                ui_changed
            }

            #[cfg(not(feature = "ui"))]
            fn render_ui(&mut self, _ui: &mut ()) -> bool {
                false
            }

            fn get_type_name() -> &'static str {
                #type_name_str
            }
        }

        // Auto-register this type when it's used
        bevy_midi_params::inventory::submit! {
            bevy_midi_params::MidiParamsRegistration {
                type_name: #type_name_str,
                register_fn: |app: &mut bevy::prelude::App| {
                    bevy_midi_params::register_midi_type::<#name #ty_generics>(app);
                },
            }
        }
    };

    Ok(expanded)
}

#[derive(Debug, Clone)]
enum ControlType {
    Range { min: f32, max: f32 },
    Button,
}

fn parse_midi_attribute(field: &Field) -> SynResult<Option<(u8, ControlType)>> {
    for attr in &field.attrs {
        if attr.path().is_ident("midi") {
            let mut cc: Option<u8> = None;
            let mut control_type: Option<ControlType> = None;

            attr.parse_nested_meta(|meta| {
                // First positional argument is CC number
                if cc.is_none() && meta.path.is_ident("") {
                    return Err(meta.error("Expected CC number as first argument"));
                }

                // Parse CC number
                if cc.is_none() {
                    if let Ok(lit) = meta.value()?.parse::<Lit>() {
                        if let Lit::Int(int_lit) = lit {
                            cc = Some(int_lit.base10_parse()?);
                            return Ok(());
                        }
                    }
                    // If not a value, try as a direct literal
                    if let Ok(Lit::Int(int_lit)) = meta.input.parse() {
                        cc = Some(int_lit.base10_parse()?);
                        return Ok(());
                    }
                }

                // Parse control type
                if meta.path.is_ident("button") {
                    control_type = Some(ControlType::Button);
                    return Ok(());
                }

                // Parse range
                if control_type.is_none() {
                    // Try to parse range expression (e.g., 0.0..1.0)
                    let range_expr: Expr = meta.input.parse()?;
                    if let Some((min, max)) = parse_range_expr(&range_expr) {
                        control_type = Some(ControlType::Range { min, max });
                        return Ok(());
                    }
                }

                Err(meta.error("Unexpected attribute"))
            })?;

            if let (Some(cc), Some(control_type)) = (cc, control_type) {
                return Ok(Some((cc, control_type)));
            } else {
                return Err(Error::new_spanned(
                    attr,
                    "midi attribute requires CC number and control type",
                ));
            }
        }
    }

    Ok(None)
}

fn parse_range_expr(expr: &Expr) -> Option<(f32, f32)> {
    // Parse range expressions like 0.0..1.0 or -1.0..1.0
    if let Expr::Range(range) = expr {
        let start = range.start.as_ref().and_then(|e| parse_float_expr(e))?;
        let end = range.end.as_ref().and_then(|e| parse_float_expr(e))?;
        return Some((start, end));
    }
    None
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