use std::path::Path;
use std::path::PathBuf;

use eframe::egui;
use std::sync::mpsc::{self, Receiver};

use crate::compression::{file_size, format_file_size, generate_output_path};
use crate::ghostscript::Ghostscript;
use crate::models::{CompressionLevel, CompressionResult, CompressionStatus};

pub fn ensure_pdf_extension(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        path.to_path_buf()
    } else {
        path.with_extension("pdf")
    }
}

pub struct PdfCompressorApp {
    input_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    compression_level: CompressionLevel,

    ghostscript: Option<Ghostscript>,
    ghostscript_error: Option<String>,

    compression_status: CompressionStatus,
    compression_receiver: Option<Receiver<Result<CompressionResult, String>>>,
}

impl Default for PdfCompressorApp {
    fn default() -> Self {
        Self {
            input_file: None,
            output_file: None,
            compression_level: CompressionLevel::Balanced,

            ghostscript: Ghostscript::detect(),
            ghostscript_error: None,

            compression_status: CompressionStatus::Idle,
            compression_receiver: None,
        }
    }
}

impl eframe::App for PdfCompressorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.check_compression_result();

        egui::Frame::central_panel(ui.style())
            .fill(crate::theme::BACKGROUND)
            .inner_margin(24.0)
            .show(ui, |ui| {
                let available_width = ui.available_width();

                let content_width = (available_width - crate::theme::CONTENT_MIN_MARGIN * 2.0)
                    .min(crate::theme::CONTENT_MAX_WIDTH);

                let outer_margin = ((available_width - content_width) / 2.0).max(0.0);

                ui.horizontal(|ui| {
                    ui.add_space(outer_margin);

                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        // Header
                        self.show_header(ui);

                        ui.add_space(crate::theme::SECTION_SPACING);

                        // Formulaire réellement centré
                        let form_width = crate::theme::FORM_MAX_WIDTH.min(content_width);

                        let form_margin = ((content_width - form_width) / 2.0).max(0.0);

                        ui.horizontal(|ui| {
                            ui.add_space(form_margin);

                            ui.vertical(|ui| {
                                ui.set_width(form_width);

                                self.show_file_section(ui);

                                ui.add_space(crate::theme::SECTION_SPACING);

                                self.show_compression_section(ui);

                                ui.add_space(crate::theme::SECTION_SPACING);

                                self.show_compress_button(ui);

                                if !matches!(self.compression_status, CompressionStatus::Idle) {
                                    ui.add_space(16.0);
                                    self.show_compression_status(ui);
                                }

                                if self.ghostscript.is_none() {
                                    ui.add_space(crate::theme::SECTION_SPACING);

                                    self.show_ghostscript_section(ui);
                                }
                            });
                        });

                        let footer_height = 44.0;

                        let footer_bottom = ui.clip_rect().bottom();
                        let footer_top = footer_bottom - footer_height;

                        let footer_rect = egui::Rect::from_min_max(
                            egui::pos2(ui.min_rect().left(), footer_top),
                            egui::pos2(ui.min_rect().right(), footer_bottom),
                        );

                        ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
                            ui.set_width(content_width);
                            self.show_footer(ui);
                        });
                    });
                });
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        crate::theme::BACKGROUND.to_normalized_gamma_f32()
    }
}

impl PdfCompressorApp {
    fn show_compression_results(&self, ui: &mut egui::Ui, results: &[CompressionResult]) {
        if results.is_empty() {
            return;
        }

        if results.len() == 1 {
            self.show_single_result(ui, &results[0]);
        } else {
            self.show_multiple_results(ui, results);
        }
    }

    fn show_single_result(&self, ui: &mut egui::Ui, result: &CompressionResult) {
        let reduction = result.reduction_percent();
        let saved = result.input_size.saturating_sub(result.output_size);

        crate::theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Titre
            ui.horizontal_centered(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());

                let painter = ui.painter();

                painter.circle_filled(rect.center(), 8.0, crate::theme::ACCENT);

                let p1 = rect.center() + egui::vec2(-4.0, 0.0);
                let p2 = rect.center() + egui::vec2(-1.0, 3.0);
                let p3 = rect.center() + egui::vec2(5.0, -4.0);

                painter.line_segment([p1, p2], egui::Stroke::new(2.0, egui::Color32::WHITE));

                painter.line_segment([p2, p3], egui::Stroke::new(2.0, egui::Color32::WHITE));

                ui.label(
                    egui::RichText::new("Compression terminée")
                        .size(crate::theme::FONT_LG)
                        .strong()
                        .color(crate::theme::ACCENT),
                );
            });

            ui.add_space(16.0);

            // Avant / Après
            ui.horizontal_centered(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(150.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label(
                            egui::RichText::new("Avant")
                                .size(crate::theme::FONT_SM)
                                .color(crate::theme::TEXT_SECONDARY),
                        );

                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(format_file_size(result.input_size))
                                .size(crate::theme::FONT_SM)
                                .strong(),
                        );
                    });
                });

                ui.add_space(80.0);

                ui.vertical(|ui| {
                    ui.set_width(150.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label(
                            egui::RichText::new("Après")
                                .size(crate::theme::FONT_SM)
                                .color(crate::theme::TEXT_SECONDARY),
                        );

                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(format_file_size(result.output_size))
                                .size(crate::theme::FONT_SM)
                                .strong(),
                        );
                    });
                });
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            // Gain
            ui.horizontal_centered(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Gain : {}", format_file_size(saved)))
                            .size(crate::theme::FONT_SM)
                            .color(crate::theme::TEXT_SECONDARY),
                    );

                    ui.add_space(3.0);

                    ui.label(
                        egui::RichText::new(format!("-{reduction:.1} %"))
                            .size(24.0)
                            .strong()
                            .color(crate::theme::ACCENT),
                    );
                });
            });
        });
    }

    fn show_multiple_results(&self, ui: &mut egui::Ui, results: &[CompressionResult]) {
        let total_input_size: u64 = results.iter().map(|result| result.input_size).sum();

        let total_output_size: u64 = results.iter().map(|result| result.output_size).sum();

        let reduction_percent = if total_input_size == 0 {
            0.0
        } else {
            100.0 * (1.0 - total_output_size as f64 / total_input_size as f64)
        };

        crate::theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(format!("✓ {} PDF compressés", results.len()))
                        .size(18.0)
                        .strong()
                        .color(crate::theme::ACCENT),
                );

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format_file_size(total_input_size))
                            .size(16.0)
                            .strong(),
                    );

                    ui.label(
                        egui::RichText::new("→")
                            .size(16.0)
                            .color(crate::theme::TEXT_SECONDARY),
                    );

                    ui.label(
                        egui::RichText::new(format_file_size(total_output_size))
                            .size(16.0)
                            .strong(),
                    );
                });

                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(format!("-{reduction_percent:.1} %"))
                        .size(22.0)
                        .strong()
                        .color(crate::theme::ACCENT),
                );
            });

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for result in results {
                        self.show_result_row(ui, result);
                    }
                });
        });
    }

    fn show_result_row(&self, ui: &mut egui::Ui, result: &CompressionResult) {
        let file_name = result
            .input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Document PDF");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(file_name).size(14.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("-{:.1} %", result.reduction_percent()))
                        .color(crate::theme::ACCENT),
                );

                ui.label(
                    egui::RichText::new(format!(
                        "{} → {}",
                        format_file_size(result.input_size),
                        format_file_size(result.output_size),
                    ))
                    .size(13.5)
                    .color(crate::theme::TEXT_SECONDARY),
                );
            });
        });

        ui.add_space(8.0);
    }

    fn select_input_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .pick_file()
        {
            self.output_file = Some(generate_output_path(&path));
            self.input_file = Some(path);
            self.compression_status = CompressionStatus::Idle;
        }
    }
    fn select_output_file(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("PDF", &["pdf"]);

        if let Some(output) = &self.output_file {
            if let Some(file_name) = output.file_name() {
                dialog = dialog.set_file_name(file_name.to_string_lossy());
            }
        }

        if let Some(path) = dialog.save_file() {
            self.output_file = Some(ensure_pdf_extension(&path));
        }
    }
    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(">")
                    .size(26.0)
                    .strong()
                    .color(crate::theme::ACCENT_WARM),
            );

            ui.label(
                egui::RichText::new("PicoShrink")
                    .size(28.0)
                    .strong()
                    .monospace()
                    .color(crate::theme::TEXT_PRIMARY),
            );
        });

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            // Décalage correspondant approximativement au "> "
            ui.add_space(31.0);

            ui.label(
                egui::RichText::new("Compressez vos fichiers PDF simplement et localement.")
                    .size(14.0)
                    .color(crate::theme::TEXT_SECONDARY),
            );
        });
    }

    fn show_ghostscript_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ghostscript");

        match &self.ghostscript {
            Some(ghostscript) => {
                ui.label("Ghostscript détecté");

                ui.label(format!("Version : {}", ghostscript.version()));

                ui.label(format!(
                    "Exécutable : {}",
                    ghostscript.executable().display()
                ));
            }

            None => {
                ui.label("Ghostscript n'a pas été détecté automatiquement.");

                if ui.button("Sélectionner Ghostscript").clicked() {
                    self.select_ghostscript();
                }
            }
        }

        if let Some(error) = &self.ghostscript_error {
            ui.label(format!("Erreur : {error}"));
        }
    }

    fn show_file_section(&mut self, ui: &mut egui::Ui) {
        crate::theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("Entrée")
                    .size(crate::theme::FONT_LG)
                    .strong()
                    .color(crate::theme::TEXT_PRIMARY),
            );

            ui.add_space(12.0);

            match &self.input_file {
                Some(path) => {
                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Document PDF");

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(file_name).size(15.0).strong());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Ok(size) = file_size(path) {
                                ui.label(
                                    egui::RichText::new(format_file_size(size))
                                        .size(13.5)
                                        .color(crate::theme::TEXT_SECONDARY),
                                );
                            }
                        });
                    });

                    ui.add_space(2.0);

                    ui.label(
                        egui::RichText::new(path.display().to_string())
                            .size(13.5)
                            .color(crate::theme::TEXT_SECONDARY),
                    );

                    ui.add_space(18.0);

                    if let Some(output) = self.output_file.clone() {
                        ui.label(
                            egui::RichText::new("Sortie")
                                .size(crate::theme::FONT_LG)
                                .strong()
                                .color(crate::theme::TEXT_PRIMARY),
                        );

                        ui.add_space(2.0);

                        ui.label(
                            egui::RichText::new(output.display().to_string())
                                .size(13.5)
                                .color(crate::theme::TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            let output_name = output
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("document_compressed.pdf");

                            ui.label(egui::RichText::new(output_name).size(14.0));

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Modifier").clicked() {
                                        self.select_output_file();
                                    }
                                },
                            );
                        });
                    }

                    ui.add_space(14.0);

                    if ui.button("Changer de PDF").clicked() {
                        self.select_input_file();
                    }
                }

                None => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Aucun PDF sélectionné")
                                .size(14.0)
                                .color(crate::theme::TEXT_SECONDARY),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Choisir un PDF").clicked() {
                                self.select_input_file();
                            }
                        });
                    });
                }
            }
        });
    }

    fn show_compression_section(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Niveau de compression")
                .size(crate::theme::FONT_LG)
                .strong(),
        );

        ui.add_space(10.0);

        ui.columns(3, |columns| {
            self.compression_option(
                &mut columns[0],
                CompressionLevel::HighQuality,
                "Haute qualité",
                "Préserve davantage les images.",
            );

            self.compression_option(
                &mut columns[1],
                CompressionLevel::Balanced,
                "Équilibrée",
                "Bon compromis, recommandée.",
            );

            self.compression_option(
                &mut columns[2],
                CompressionLevel::Strong,
                "Forte",
                "Réduit davantage la taille.",
            );
        });
    }
    fn compression_option(
        &mut self,
        ui: &mut egui::Ui,
        level: CompressionLevel,
        title: &str,
        description: &str,
    ) {
        let selected = self.compression_level == level;

        let fill = if selected {
            crate::theme::CARD_SELECTED
        } else {
            crate::theme::CARD_BACKGROUND
        };

        let stroke = if selected {
            egui::Stroke::new(2.0, crate::theme::ACCENT)
        } else {
            egui::Stroke::new(1.0, crate::theme::BORDER)
        };

        let response = egui::Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(10.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_height(90.0);

                ui.label(
                    egui::RichText::new(title)
                        .size(crate::theme::FONT_MD)
                        .strong(),
                );

                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(description)
                        .size(crate::theme::FONT_SM)
                        .color(crate::theme::TEXT_SECONDARY),
                );
            })
            .response;

        if response.interact(egui::Sense::click()).clicked() {
            self.compression_level = level;
        }
    }
    fn show_compress_button(&mut self, ui: &mut egui::Ui) {
        let running = matches!(self.compression_status, CompressionStatus::Running);

        let can_compress = self.input_file.is_some()
            && self.output_file.is_some()
            && self.ghostscript.is_some()
            && !running;

        let button_width = crate::theme::BUTTON_WIDTH;
        let available_width = ui.available_width();

        ui.horizontal(|ui| {
            let left_space = ((available_width - button_width) / 2.0).max(0.0);

            ui.add_space(left_space);

            let button = egui::Button::new(
                egui::RichText::new("Compresser le PDF")
                    .size(crate::theme::FONT_LG)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(crate::theme::ACCENT)
            .corner_radius(10.0)
            .min_size(egui::vec2(button_width, 46.0));

            if ui.add_enabled(can_compress, button).clicked() {
                self.start_compression();
            }
        });
    }

    fn show_compression_status(&self, ui: &mut egui::Ui) {
        match &self.compression_status {
            CompressionStatus::Idle => {}

            CompressionStatus::Running => {
                ui.label("Compression en cours…");

                ui.add(
                    egui::ProgressBar::new(0.5)
                        .animate(true)
                        .text("Traitement du PDF"),
                );

                ui.ctx().request_repaint();
            }

            CompressionStatus::Success(results) => {
                self.show_compression_results(ui, results);
            }

            CompressionStatus::Error(error) => {
                ui.label(format!("Erreur : {error}"));
            }
        }
    }

    fn show_footer(&mut self, ui: &mut egui::Ui) {
        ui.set_min_height(36.0);

        ui.vertical_centered(|ui| {
            ui.separator();
            ui.add_space(6.0);

            ui.columns(3, |columns| {
                columns[0].label(
                    egui::RichText::new(format!("PicoShrink {}", env!("CARGO_PKG_VERSION")))
                        .size(13.5)
                        .color(crate::theme::TEXT_SECONDARY),
                );

                columns[1].vertical_centered(|ui| match &self.ghostscript {
                    Some(ghostscript) => {
                        ui.label(
                            egui::RichText::new(format!("Ghostscript {} ✓", ghostscript.version()))
                                .size(13.5)
                                .color(crate::theme::ACCENT),
                        );
                    }

                    None => {
                        ui.label(
                            egui::RichText::new("Ghostscript non détecté")
                                .size(13.5)
                                .color(crate::theme::TEXT_SECONDARY),
                        );
                    }
                });

                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(env!("CARGO_PKG_LICENSE"))
                            .size(13.5)
                            .color(crate::theme::TEXT_SECONDARY),
                    );
                });
            });
        });
    }

    fn start_compression(&mut self) {
        let Some(input) = self.input_file.clone() else {
            self.compression_status = CompressionStatus::Error("Aucun fichier sélectionné".into());
            return;
        };

        let Some(output) = self.output_file.clone() else {
            self.compression_status =
                CompressionStatus::Error("Aucun fichier de sortie défini".into());
            return;
        };

        let Some(ghostscript) = self.ghostscript.clone() else {
            self.compression_status = CompressionStatus::Error("Ghostscript indisponible".into());
            return;
        };

        let level = self.compression_level;

        let (sender, receiver) = mpsc::channel();

        self.compression_receiver = Some(receiver);
        self.compression_status = CompressionStatus::Running;

        std::thread::spawn(move || {
            let result = ghostscript.compress(&input, &output, level);

            let result = match result {
                Ok(()) => match (file_size(&input), file_size(&output)) {
                    (Ok(input_size), Ok(output_size)) => Ok(CompressionResult {
                        input,
                        output,
                        input_size,
                        output_size,
                    }),

                    (Err(error), _) | (_, Err(error)) => Err(error),
                },

                Err(error) => Err(error),
            };

            let _ = sender.send(result);
        });
    }

    fn check_compression_result(&mut self) {
        let Some(receiver) = &self.compression_receiver else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(result)) => {
                self.compression_status = CompressionStatus::Success(vec![result]);

                self.compression_receiver = None;
            }

            Ok(Err(error)) => {
                self.compression_status = CompressionStatus::Error(error);

                self.compression_receiver = None;
            }

            Err(mpsc::TryRecvError::Empty) => {}

            Err(mpsc::TryRecvError::Disconnected) => {
                self.compression_status =
                    CompressionStatus::Error("Le thread de compression s'est interrompu".into());

                self.compression_receiver = None;
            }
        }
    }

    fn select_ghostscript(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };

        match Ghostscript::from_path(path) {
            Ok(ghostscript) => {
                self.ghostscript = Some(ghostscript);
                self.ghostscript_error = None;
            }

            Err(error) => {
                self.ghostscript = None;
                self.ghostscript_error = Some(error);
            }
        }
    }
}
