// ===== bevy_midi_params_derive/src/lib.rs =====

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
    let mut persistence_fields = Vec::new();
    let mut load_fields = Vec::new();
    let mut change_detection = Vec::new();

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
                            // ui_changed |= ui.add(egui::Slider::new(&mut self.#field_name, #min..=#max)
                            //     .text(format!("{:.3}", self.#field_name))).changed();
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

        // Persistence for all fields (not just MIDI ones)
        persistence_fields.push(quote! {
            data.insert(#field_name_str, &self.#field_name);
        });

        load_fields.push(quote! {
            if let Some(value) = data.get(#field_name_str) {
                self.#field_name = value;
            }
        });

        change_detection.push(quote! {
            (self.#field_name != old.#field_name)
        });
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

            fn to_persist_data(&self) -> bevy_midi_params::PersistData {
                let mut data = bevy_midi_params::PersistData::new();
                #(#persistence_fields)*
                data
            }

            fn from_persist_data(&mut self, data: &bevy_midi_params::PersistData) {
                #(#load_fields)*
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(dead_code)]
            fn has_changed_from(&self, old: &Self) -> bool {
                #(#change_detection)||*
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
            let midi_attr = parse_midi_meta(&attr.meta)?;
            return Ok(Some((midi_attr.cc, midi_attr.control_type)));
        }
    }
    Ok(None)
}

fn parse_midi_meta(meta: &Meta) -> SynResult<MidiAttr> {
    match meta {
        Meta::List(meta_list) => {
            let tokens = &meta_list.tokens;
            syn::parse2(tokens.clone())
        }
        _ => Err(Error::new_spanned(
            meta,
            "Expected #[midi(...)] with parameters",
        )),
    }
}

// Parse different attribute formats:
// #[midi(1, 0.0..1.0)]          - CC range control
// #[midi(2, 0.0..=5.0)]          - CC range control (inclusive)
// #[midi(3, button)]             - CC button/toggle
// #[midi(4)]                     - CC default range 0.0..1.0
// #[midi(note = 18, button)]     - Note-based button
// #[midi(cc = 33, button)]       - CC-based button (explicit)
struct MidiAttr {
    cc: u8,
    control_type: ControlType,
    is_note: bool,
}

impl syn::parse::Parse for MidiAttr {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        let mut cc = None;
        let mut is_note = false;
        let mut control_type = None;

        // Check if first token is an identifier (for named parameters)
        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "cc" => {
                    let _eq: Token![=] = input.parse()?;
                    let cc_lit: Lit = input.parse()?;
                    cc = Some(extract_u8_from_lit(&cc_lit)?);
                }
                "note" => {
                    let _eq: Token![=] = input.parse()?;
                    let note_lit: Lit = input.parse()?;
                    cc = Some(extract_u8_from_lit(&note_lit)? + 128); // Offset notes
                    is_note = true;
                }
                _ => return Err(Error::new_spanned(ident, "Expected 'cc' or 'note'")),
            }
        } else {
            // First token is a number (traditional syntax)
            let cc_lit: Lit = input.parse()?;
            cc = Some(extract_u8_from_lit(&cc_lit)?);
        }

        let cc = cc.ok_or_else(|| {
            Error::new(proc_macro2::Span::call_site(), "Missing CC or note number")
        })?;

        if input.is_empty() {
            // Default range for CC, error for note
            if is_note {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    "Note mappings must specify 'button'",
                ));
            }
            return Ok(MidiAttr {
                cc,
                control_type: ControlType::Range { min: 0.0, max: 1.0 },
                is_note,
            });
        }

        let _comma: Token![,] = input.parse()?;

        if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "button" {
                control_type = Some(ControlType::Button);
            } else {
                return Err(Error::new_spanned(
                    ident,
                    "Expected 'button' or range (e.g., 0.0..1.0)",
                ));
            }
        } else {
            // Parse range: 0.0..1.0 or 0.0..=1.0
            if is_note {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    "Note mappings can only be buttons",
                ));
            }

            let start: Lit = input.parse()?;
            let start = extract_f32_from_lit(&start)?;

            let _dots: Token![..] = input.parse()?;
            let _inclusive = input.parse::<Token![=]>().is_ok();

            let end: Lit = input.parse()?;
            let end = extract_f32_from_lit(&end)?;

            if start >= end {
                return Err(Error::new(
                    proc_macro2::Span::call_site(),
                    "Range start must be less than end",
                ));
            }

            control_type = Some(ControlType::Range {
                min: start,
                max: end,
            });
        }

        let control_type = control_type
            .ok_or_else(|| Error::new(proc_macro2::Span::call_site(), "Missing control type"))?;

        Ok(MidiAttr {
            cc,
            control_type,
            is_note,
        })
    }
}

fn extract_u8_from_lit(lit: &Lit) -> SynResult<u8> {
    match lit {
        Lit::Int(int) => {
            let val = int.base10_parse::<u8>()?;
            if val > 127 {
                return Err(Error::new_spanned(int, "MIDI CC/Note must be 0-127"));
            }
            Ok(val)
        }
        _ => Err(Error::new_spanned(lit, "Expected integer for MIDI CC/Note")),
    }
}

fn extract_f32_from_lit(lit: &Lit) -> SynResult<f32> {
    match lit {
        Lit::Float(f) => f.base10_parse::<f32>(),
        Lit::Int(i) => Ok(i.base10_parse::<i32>()? as f32),
        _ => Err(Error::new_spanned(lit, "Expected number for range")),
    }
}
