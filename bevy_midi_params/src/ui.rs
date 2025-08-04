#[cfg(feature = "ui")]
use crate::{MidiController, MidiParamsRegistration};
#[cfg(feature = "ui")]
use bevy::prelude::*;
#[cfg(feature = "ui")]
use bevy_egui::{egui, EguiContexts};

/// System to render MIDI control UI
#[cfg(feature = "ui")]
pub fn midi_control_ui(
    mut contexts: EguiContexts,
    midi_controller: Res<MidiController>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("🎛️ MIDI Live Controls")
        .default_width(450.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Hardware MIDI Control");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("(auto-saves changes)");
                });
            });

            ui.separator();

            // Connection status
            ui.horizontal(|ui| {
                let connected = !midi_controller.values.is_empty();
                let (icon, color) = if connected {
                    ("🟢", egui::Color32::GREEN)
                } else {
                    ("🔴", egui::Color32::RED)
                };

                ui.colored_label(color, icon);
                ui.label(if connected {
                    "MIDI Connected"
                } else {
                    "No MIDI Input"
                });

                if !connected && ui.button("🔄 Retry").clicked() {
                    // This would trigger a reconnection attempt
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // This is where we'd render UI for each registered type
                // For now, show a placeholder that explains the auto-generation

                ui.collapsing("📋 Registered MIDI Types", |ui| {
                    for registration in inventory::iter::<MidiParamsRegistration> {
                        ui.label(format!("✅ {}", registration.type_name));
                    }

                    if midi_controller.number_of_registered_types() == 0{
                        ui.colored_label(egui::Color32::YELLOW, "No MIDI types registered yet");
                        ui.small("Add #[derive(MidiParams)] to your Resource structs");
                    }
                });

                ui.collapsing("🎛️ MIDI Reference", |ui| {
                    ui.label("AKAI MIDImix Layout:");
                    ui.separator();

                    ui.group(|ui| {
                        ui.label("🎛️ Knobs:");
                        ui.label("  • Row 1: CCs 1-8");
                        ui.label("  • Row 2: CCs 9-16");
                        ui.label("  • Row 3: CCs 17-24");

                        ui.label("📏 Faders:");
                        ui.label("  • Channels 1-8: CCs 19-26");
                        ui.label("  • Master: CC 62");

                        ui.label("🔘 Buttons:");
                        ui.label("  • Top row: CCs 33-40");
                        ui.label("  • Bottom row: CCs 49-56");
                    });
                });

                ui.collapsing("📊 Live MIDI Values", |ui| {
                    if midi_controller.values.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "No MIDI input received");
                        ui.small("Turn a knob or move a fader to see values here");
                    } else {
                        ui.columns(4, |columns| {
                            for (&cc, &value) in &midi_controller.values {
                                let col_idx = cc as usize % 4;
                                let mapping = midi_controller.get_mappings().get(&cc);

                                let display = if let Some(mapping) = mapping {
                                    format!("CC{} ({}): {:.2}", cc, mapping.field_name, value)
                                } else {
                                    format!("CC{}: {:.2}", cc, value)
                                };

                                columns[col_idx].label(display);
                            }
                        });
                    }
                });
            });
        });
}

// Dummy UI function for when ui feature is disabled
#[cfg(not(feature = "ui"))]
pub fn midi_control_ui() {
    // No-op when UI is disabled
}
