use std::path::Path;
use std::path::PathBuf;

use eframe::egui;
use std::sync::mpsc::{self, Receiver};

use crate::compression::{file_size, format_file_size, generate_output_path};
use crate::ghostscript::Ghostscript;
use crate::models::{CompressionLevel, CompressionStatus};

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
    compression_receiver: Option<Receiver<Result<String, String>>>,
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

                let left = (available_width - content_width) / 2.0;

                let rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().left() + left, ui.cursor().top()),
                    egui::vec2(content_width, ui.available_height()),
                );

                // Toute l'application est maintenant limitée
                // à notre colonne centrée.
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    let full_rect = ui.max_rect();
                    let footer_height = 36.0;

                    let content_rect = egui::Rect::from_min_max(
                        full_rect.left_top(),
                        egui::pos2(
                            full_rect.right(),
                            full_rect.bottom() - footer_height - crate::theme::SECTION_SPACING,
                        ),
                    );

                    // Contenu principal
                    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                        ui.set_width(content_width);

                        self.show_header(ui);

                        ui.add_space(crate::theme::SECTION_SPACING);

                        ui.vertical_centered(|ui| {
                            ui.set_width(crate::theme::FORM_MAX_WIDTH);

                            self.show_file_section(ui);

                            ui.add_space(crate::theme::SECTION_SPACING);
                            self.show_compression_section(ui);

                            ui.add_space(crate::theme::SECTION_SPACING);
                            self.show_compress_button(ui);

                            if self.ghostscript.is_none() {
                                ui.add_space(crate::theme::SECTION_SPACING);
                                self.show_ghostscript_section(ui);
                            }
                        });
                    });

                    // Footer collé en bas
                    let footer_rect = egui::Rect::from_min_max(
                        egui::pos2(full_rect.left(), full_rect.bottom() - footer_height),
                        full_rect.right_bottom(),
                    );

                    ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
                        ui.set_width(content_width);
                        self.show_footer(ui);
                    });
                });
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        crate::theme::BACKGROUND.to_normalized_gamma_f32()
    }
}

impl PdfCompressorApp {
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
                egui::RichText::new("Compressez vos fichiers PDF rapidement et simplement.")
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

            ui.label(egui::RichText::new("Document").size(18.0).strong());

            ui.add_space(10.0);

            match &self.input_file {
                Some(path) => {
                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Document PDF");

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(file_name).size(15.0).strong());

                        if let Ok(size) = file_size(path) {
                            ui.label(
                                egui::RichText::new(format_file_size(size))
                                    .small()
                                    .color(crate::theme::TEXT_SECONDARY),
                            );
                        }
                    });

                    ui.label(
                        egui::RichText::new(path.display().to_string())
                            .small()
                            .weak(),
                    );
                }

                None => {
                    ui.label(
                        egui::RichText::new("Aucun PDF sélectionné")
                            .size(15.0)
                            .color(crate::theme::TEXT_SECONDARY),
                    );
                }
            }

            ui.add_space(14.0);

            if let Some(output) = &self.output_file {
                ui.label(egui::RichText::new("Fichier de sortie").small().strong());

                ui.label(
                    egui::RichText::new(output.display().to_string())
                        .small()
                        .color(crate::theme::TEXT_SECONDARY),
                );

                ui.add_space(10.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Choisir un PDF").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .pick_file()
                    {
                        self.output_file = Some(generate_output_path(&path));
                        self.input_file = Some(path);
                        self.compression_status = CompressionStatus::Idle;
                    }
                }

                let can_modify_output = self.output_file.is_some();

                if ui
                    .add_enabled(can_modify_output, egui::Button::new("Modifier la sortie"))
                    .clicked()
                {
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
            });
        });
    }

    fn show_compression_section(&mut self, ui: &mut egui::Ui) {
        crate::theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(egui::RichText::new("Compression").size(18.0).strong());

            ui.add_space(8.0);

            ui.radio_value(
                &mut self.compression_level,
                CompressionLevel::HighQuality,
                "Haute qualité",
            );

            ui.label(
                egui::RichText::new("Préserve davantage la qualité des images.")
                    .small()
                    .weak(),
            );

            ui.add_space(10.0);

            ui.radio_value(
                &mut self.compression_level,
                CompressionLevel::Balanced,
                "Équilibrée",
            );

            ui.label(
                egui::RichText::new("Bon compromis entre qualité et taille du fichier.")
                    .small()
                    .weak(),
            );

            ui.add_space(10.0);

            ui.radio_value(
                &mut self.compression_level,
                CompressionLevel::Strong,
                "Forte",
            );

            ui.label(
                egui::RichText::new("Réduit davantage la taille du PDF.")
                    .small()
                    .weak(),
            );
        });
    }

    fn show_compress_button(&mut self, ui: &mut egui::Ui) {
        let running = matches!(self.compression_status, CompressionStatus::Running);

        let can_compress = self.input_file.is_some()
            && self.output_file.is_some()
            && self.ghostscript.is_some()
            && !running;

        let button = egui::Button::new(
            egui::RichText::new("Compresser le PDF")
                .size(16.0)
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(crate::theme::ACCENT)
        .corner_radius(10.0)
        .min_size(egui::vec2(crate::theme::BUTTON_WIDTH, 46.0));

        if ui.add_enabled(can_compress, button).clicked() {
            self.start_compression();
        }

        ui.add_space(10.0);

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

            CompressionStatus::Success(message) => {
                ui.label(message);
            }

            CompressionStatus::Error(error) => {
                ui.label(format!("Erreur : {error}"));
            }
        }
    }

    fn show_footer(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.add_space(8.0);

        ui.columns(3, |columns| {
            // Gauche
            columns[0].label(
                egui::RichText::new(format!("PicoShrink {}", env!("CARGO_PKG_VERSION")))
                    .small()
                    .color(crate::theme::TEXT_SECONDARY),
            );

            // Centre
            columns[1].vertical_centered(|ui| match &self.ghostscript {
                Some(ghostscript) => {
                    ui.label(
                        egui::RichText::new(format!("Ghostscript {} ✓", ghostscript.version()))
                            .small()
                            .color(crate::theme::ACCENT),
                    );
                }

                None => {
                    ui.label(
                        egui::RichText::new("Ghostscript non détecté")
                            .small()
                            .color(crate::theme::TEXT_SECONDARY),
                    );
                }
            });

            // Droite
            columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(env!("CARGO_PKG_LICENSE"))
                        .small()
                        .color(crate::theme::TEXT_SECONDARY),
                );
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
                Ok(()) => Ok(format!("Compression terminée : {}", output.display())),
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
            Ok(Ok(message)) => {
                self.compression_status = CompressionStatus::Success(message);

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
