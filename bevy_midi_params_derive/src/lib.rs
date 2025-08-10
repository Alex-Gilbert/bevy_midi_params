use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Result as SynResult};

mod utils;
use utils::{determine_control_type, parse_midi_attributes, ControlType, MidiMapping, RangeSpec};

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
        let field_type = &field.ty;

        // Parse all midi attributes on this field
        let mappings = parse_midi_attributes(field)?;

        if !mappings.is_empty() {
            // Determine control type based on field type and mappings
            let control_type = determine_control_type(field_type, &mappings)?;

            match control_type {
                ControlType::Range { min, max } => {
                    // Single range control
                    let mapping = &mappings[0];
                    let cc = mapping.cc;

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
                            ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name, #min..=#max)).changed();
                        });
                    });
                }
                ControlType::VectorRange { components } => {
                    // Multiple component controls for vector types
                    let component_names = ["x", "y", "z", "w"];
                    let display_name = field_name_str.replace('_', " ");

                    // Add a group label for the vector
                    ui_controls.push(quote! {
                        ui.label(format!("{}:", #display_name));
                    });

                    for (i, (cc, min, max)) in components.iter().enumerate() {
                        let comp_name = component_names[i];
                        let field_comp_name = format!("{}.{}", field_name_str, comp_name);

                        // MIDI mapping
                        midi_mappings.push(quote! {
                            bevy_midi_params::MidiMapping::range(#cc, #field_comp_name, #min, #max)
                        });

                        // MIDI update logic - access component by index
                        let idx = i;
                        midi_updates.push(quote! {
                            #cc => {
                                let new_value = #min + value * (#max - #min);
                                if (self.#field_name[#idx] - new_value).abs() > f32::EPSILON {
                                    self.#field_name[#idx] = new_value;
                                    changed = true;
                                }
                            }
                        });

                        // UI control for this component
                        ui_controls.push(quote! {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} (CC{}):", #comp_name, #cc));
                                ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name[#idx], #min..=#max)).changed();
                            });
                        });
                    }
                }
                ControlType::Toggle => {
                    let mapping = &mappings[0];
                    let cc = mapping.cc;

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
                ControlType::IntRange { min, max } => {
                    let mapping = &mappings[0];
                    let cc = mapping.cc;

                    // MIDI mapping
                    midi_mappings.push(quote! {
                        bevy_midi_params::MidiMapping::range(#cc, #field_name_str, #min as f32, #max as f32)
                    });

                    // MIDI update logic
                    midi_updates.push(quote! {
                        #cc => {
                            let new_value = (#min as f32 + value * (#max - #min) as f32).round() as i32;
                            if self.#field_name != new_value {
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
                            ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name, #min..=#max)).changed();
                        });
                    });
                }
                ControlType::LinearRgba => {
                    // LinearRgba has red, green, blue, alpha fields
                    if mappings.len() != 4 {
                        return Err(Error::new_spanned(
                            field_type,
                            "LinearRgba requires exactly 4 #[midi] attributes (r, g, b, a)",
                        ));
                    }

                    let display_name = field_name_str.replace('_', " ");
                    let component_idents = [
                        quote::format_ident!("red"),
                        quote::format_ident!("green"),
                        quote::format_ident!("blue"),
                        quote::format_ident!("alpha"),
                    ];
                    let component_labels = ["r", "g", "b", "a"];

                    ui_controls.push(quote! {
                        ui.label(format!("{} (LinearRgba):", #display_name));
                    });

                    for (i, mapping) in mappings.iter().enumerate() {
                        let cc = mapping.cc;
                        let (min, max) = if let Some(ref range) = mapping.range {
                            match range {
                                RangeSpec::Float(min, max) => (*min, *max),
                                RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                            }
                        } else {
                            (0.0, 1.0)
                        };

                        let comp_ident = &component_idents[i];
                        let comp_label = component_labels[i];
                        let field_comp_name = format!("{}.{}", field_name_str, comp_label);

                        // MIDI mapping
                        midi_mappings.push(quote! {
                            bevy_midi_params::MidiMapping::range(#cc, #field_comp_name, #min, #max)
                        });

                        // MIDI update logic
                        midi_updates.push(quote! {
                            #cc => {
                                let new_value = #min + value * (#max - #min);
                                if (self.#field_name.#comp_ident - new_value).abs() > f32::EPSILON {
                                    self.#field_name.#comp_ident = new_value;
                                    changed = true;
                                }
                            }
                        });

                        // UI control
                        ui_controls.push(quote! {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} (CC{}):", #comp_label, #cc));
                                ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name.#comp_ident, #min..=#max)).changed();
                            });
                        });
                    }
                }
                ControlType::Srgba => {
                    // Srgba has red, green, blue, alpha fields
                    if mappings.len() != 4 {
                        return Err(Error::new_spanned(
                            field_type,
                            "Srgba requires exactly 4 #[midi] attributes (r, g, b, a)",
                        ));
                    }

                    let display_name = field_name_str.replace('_', " ");
                    let component_idents = [
                        quote::format_ident!("red"),
                        quote::format_ident!("green"),
                        quote::format_ident!("blue"),
                        quote::format_ident!("alpha"),
                    ];
                    let component_labels = ["r", "g", "b", "a"];

                    ui_controls.push(quote! {
                        ui.label(format!("{} (Srgba):", #display_name));
                    });

                    for (i, mapping) in mappings.iter().enumerate() {
                        let cc = mapping.cc;
                        let (min, max) = if let Some(ref range) = mapping.range {
                            match range {
                                RangeSpec::Float(min, max) => (*min, *max),
                                RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                            }
                        } else {
                            (0.0, 1.0)
                        };

                        let comp_ident = &component_idents[i];
                        let comp_label = component_labels[i];
                        let field_comp_name = format!("{}.{}", field_name_str, comp_label);

                        // MIDI mapping
                        midi_mappings.push(quote! {
                            bevy_midi_params::MidiMapping::range(#cc, #field_comp_name, #min, #max)
                        });

                        // MIDI update logic
                        midi_updates.push(quote! {
                            #cc => {
                                let new_value = #min + value * (#max - #min);
                                if (self.#field_name.#comp_ident - new_value).abs() > f32::EPSILON {
                                    self.#field_name.#comp_ident = new_value;
                                    changed = true;
                                }
                            }
                        });

                        // UI control
                        ui_controls.push(quote! {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} (CC{}):", #comp_label, #cc));
                                ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name.#comp_ident, #min..=#max)).changed();
                            });
                        });
                    }
                }
                ControlType::Hsla => {
                    // Hsla has hue, saturation, lightness, alpha fields
                    if mappings.len() != 4 {
                        return Err(Error::new_spanned(
                            field_type,
                            "Hsla requires exactly 4 #[midi] attributes (h, s, l, a)",
                        ));
                    }

                    let display_name = field_name_str.replace('_', " ");
                    let component_idents = [
                        quote::format_ident!("hue"),
                        quote::format_ident!("saturation"),
                        quote::format_ident!("lightness"),
                        quote::format_ident!("alpha"),
                    ];
                    let component_labels = ["h", "s", "l", "a"];

                    ui_controls.push(quote! {
                        ui.label(format!("{} (Hsla):", #display_name));
                    });

                    for (i, mapping) in mappings.iter().enumerate() {
                        let cc = mapping.cc;
                        let (min, max) = if let Some(ref range) = mapping.range {
                            match range {
                                RangeSpec::Float(min, max) => (*min, *max),
                                RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                            }
                        } else {
                            // Default ranges for HSL
                            match i {
                                0 => (0.0, 360.0), // hue
                                _ => (0.0, 1.0),   // saturation, lightness, alpha
                            }
                        };

                        let comp_ident = &component_idents[i];
                        let comp_label = component_labels[i];
                        let field_comp_name = format!("{}.{}", field_name_str, comp_label);

                        // MIDI mapping
                        midi_mappings.push(quote! {
                            bevy_midi_params::MidiMapping::range(#cc, #field_comp_name, #min, #max)
                        });

                        // MIDI update logic
                        midi_updates.push(quote! {
                            #cc => {
                                let new_value = #min + value * (#max - #min);
                                if (self.#field_name.#comp_ident - new_value).abs() > f32::EPSILON {
                                    self.#field_name.#comp_ident = new_value;
                                    changed = true;
                                }
                            }
                        });

                        // UI control
                        ui_controls.push(quote! {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} (CC{}):", #comp_label, #cc));
                                ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name.#comp_ident, #min..=#max)).changed();
                            });
                        });
                    }
                }
                ControlType::Hsva => {
                    // Hsva has hue, saturation, value, alpha fields
                    if mappings.len() != 4 {
                        return Err(Error::new_spanned(
                            field_type,
                            "Hsva requires exactly 4 #[midi] attributes (h, s, v, a)",
                        ));
                    }

                    let display_name = field_name_str.replace('_', " ");
                    let component_idents = [
                        quote::format_ident!("hue"),
                        quote::format_ident!("saturation"),
                        quote::format_ident!("value"),
                        quote::format_ident!("alpha"),
                    ];
                    let component_labels = ["h", "s", "v", "a"];

                    ui_controls.push(quote! {
                        ui.label(format!("{} (Hsva):", #display_name));
                    });

                    for (i, mapping) in mappings.iter().enumerate() {
                        let cc = mapping.cc;
                        let (min, max) = if let Some(ref range) = mapping.range {
                            match range {
                                RangeSpec::Float(min, max) => (*min, *max),
                                RangeSpec::Int(min, max) => (*min as f32, *max as f32),
                            }
                        } else {
                            // Default ranges for HSV
                            match i {
                                0 => (0.0, 360.0), // hue
                                _ => (0.0, 1.0),   // saturation, value, alpha
                            }
                        };

                        let comp_ident = &component_idents[i];
                        let comp_label = component_labels[i];
                        let field_comp_name = format!("{}.{}", field_name_str, comp_label);

                        // MIDI mapping
                        midi_mappings.push(quote! {
                            bevy_midi_params::MidiMapping::range(#cc, #field_comp_name, #min, #max)
                        });

                        // MIDI update logic
                        midi_updates.push(quote! {
                            #cc => {
                                let new_value = #min + value * (#max - #min);
                                if (self.#field_name.#comp_ident - new_value).abs() > f32::EPSILON {
                                    self.#field_name.#comp_ident = new_value;
                                    changed = true;
                                }
                            }
                        });

                        // UI control
                        ui_controls.push(quote! {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} (CC{}):", #comp_label, #cc));
                                ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name.#comp_ident, #min..=#max)).changed();
                            });
                        });
                    }
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
