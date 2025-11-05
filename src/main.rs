#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, Visuals};
use eframe::{App, CreationContext};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::fs::File;
use std::io::{Read, Write, Cursor};
use zip::{ZipArchive, ZipWriter};
use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};

struct DocxCompressorApp {
    input_path: String,
    output_path: String,
    quality: u8,
    max_width: u32,
    progress: Arc<Mutex<f32>>,
    status: Arc<Mutex<String>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    final_elapsed: Arc<Mutex<Option<f32>>>,
}

impl DocxCompressorApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let font_bytes = include_bytes!("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("system_chinese_font".to_owned(), FontData::from_static(font_bytes));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "system_chinese_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            input_path: String::new(),
            output_path: String::new(),
            quality: 70,
            max_width: 1280,
            progress: Arc::new(Mutex::new(0.0)),
            status: Arc::new(Mutex::new("等待文件...".to_owned())),
            start_time: Arc::new(Mutex::new(None)),
            final_elapsed: Arc::new(Mutex::new(None)),
        }
    }

    fn start_compression(&mut self) {
        let progress = Arc::clone(&self.progress);
        let status = Arc::clone(&self.status);
        let start_time = Arc::clone(&self.start_time);
        let final_elapsed = Arc::clone(&self.final_elapsed);
        let input = self.input_path.clone();
        let output = self.output_path.clone();
        let quality = self.quality;
        let max_width = self.max_width;

        // 重置进度和时间
        *progress.lock().unwrap() = 0.0;
        *start_time.lock().unwrap() = Some(Instant::now());
        *final_elapsed.lock().unwrap() = None;

        thread::spawn(move || {
            *status.lock().unwrap() = format!("正在压缩: {}...", input);

            let file = File::open(&input).expect("无法打开输入文件");
            let mut archive = ZipArchive::new(file).expect("无法读取 DOCX 文件");

            let mut buffer = Cursor::new(Vec::new());
            {
                let mut zip_writer = ZipWriter::new(&mut buffer);
                let total_files = archive.len();

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let name = file.name().to_string();
                    let mut data = Vec::new();
                    file.read_to_end(&mut data).unwrap();

                    if name.starts_with("word/media/") && (name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg")) {
                        if let Ok(img) = ImageReader::new(Cursor::new(&data)).with_guessed_format().unwrap().decode() {
                            let img = img.resize(
                                max_width.min(img.width()),
                                max_width.min(img.height()),
                                image::imageops::FilterType::Lanczos3,
                            );

                            let mut img_buf = Vec::new();
                            if name.ends_with(".png") {
                                let encoder = PngEncoder::new(&mut img_buf);
                                encoder.write_image(&img.to_rgba8(), img.width(), img.height(), ColorType::Rgba8.into()).unwrap();
                            } else {
                                let mut encoder = JpegEncoder::new_with_quality(&mut img_buf, quality);
                                encoder.encode_image(&img).unwrap();
                            }

                            data = img_buf;
                        }
                    }

                    zip_writer.start_file(name, zip::write::FileOptions::default()).unwrap();
                    zip_writer.write_all(&data).unwrap();

                    *progress.lock().unwrap() = (i + 1) as f32 / total_files as f32;
                }

                zip_writer.finish().unwrap();
            }

            let mut out_file = File::create(&output).expect("无法创建输出文件");
            out_file.write_all(&buffer.into_inner()).unwrap();

            // 保存最终耗时
            let elapsed = start_time.lock().unwrap().unwrap().elapsed().as_secs_f32();
            *final_elapsed.lock().unwrap() = Some(elapsed);
            *progress.lock().unwrap() = 1.0;
            *status.lock().unwrap() = format!("✅ 压缩完成，用时 {:.1}s", elapsed);
        });
    }
}

impl App for DocxCompressorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(RichText::new("📘 DOCX 图片压缩器").size(30.0).color(Color32::from_rgb(100, 180, 255)));
                ui.label(RichText::new("轻量级文档优化工具").size(15.0).color(Color32::from_gray(180)));
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("📂 输入文件:");
                ui.text_edit_singleline(&mut self.input_path);
                if ui.button("选择...").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Word 文档", &["docx"]).pick_file() {
                        self.input_path = path.display().to_string();
                        self.output_path = self.input_path.replace(".docx", "_压缩后.docx");
                        *self.status.lock().unwrap() = "✅ 已选择文件，请点击开始压缩".to_string();
                        *self.final_elapsed.lock().unwrap() = None;
                    }
                }
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("💾 输出文件:");
                ui.text_edit_singleline(&mut self.output_path);
                if ui.button("选择...").clicked() {
                    if let Some(path) = FileDialog::new().save_file() {
                        self.output_path = path.display().to_string();
                        *self.status.lock().unwrap() = "✅ 已选择文件，请点击开始压缩".to_string();
                        *self.final_elapsed.lock().unwrap() = None;
                    }
                }
            });

            ui.add_space(10.0);
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("⚙ 压缩质量 (1-100):");
                ui.add(egui::Slider::new(&mut self.quality, 1..=100));
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("📏 最大宽度 (px):");
                ui.add(egui::Slider::new(&mut self.max_width, 300..=4000));
            });

            ui.add_space(15.0);

            let current_status = self.status.lock().unwrap().clone();
            ui.label(format!("🪄 状态: {}", current_status));

            let progress_value = *self.progress.lock().unwrap();
            ui.add_space(10.0);

            // 时间显示逻辑
            let (elapsed, remaining) = if progress_value >= 1.0 {
                let final_time = *self.final_elapsed.lock().unwrap();
                (final_time.unwrap_or(0.0), 0.0)
            } else {
                if let Some(start) = *self.start_time.lock().unwrap() {
                    let elapsed = start.elapsed().as_secs_f32();
                    let prog = progress_value.clamp(0.01, 1.0);
                    let remaining = (elapsed / prog) - elapsed;
                    (elapsed, remaining)
                } else {
                    (0.0, 0.0)
                }
            };

            // 进度条
            ui.add(
                egui::ProgressBar::new(progress_value)
                    .desired_width(ui.available_width())
                    .show_percentage()
                    .fill(Color32::from_rgb(90, 170, 255)),
            );

            // 时间显示在进度条下方
            ui.horizontal(|ui| {
                ui.label(format!("已用: {:.1}s | 剩余: {:.1}s", elapsed, remaining));
            });

            ui.add_space(25.0);

            let can_start = !self.input_path.is_empty() && !self.output_path.is_empty();
            let button = ui.add_sized([220.0, 45.0], egui::Button::new(RichText::new("🚀 开始压缩").size(20.0)));
            if can_start && button.clicked() {
                self.start_compression();
            }

            ui.add_space(20.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("by AdySec").size(15.0).color(Color32::from_gray(170)).italics());
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(33));
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_min_inner_size([500.0, 400.0])
            .with_drag_and_drop(true)
            .with_title(if cfg!(target_os = "linux") { "DOCX Compressor" } else { "DOCX 图片压缩器" }),
        ..Default::default()
    };

    eframe::run_native(
        if cfg!(target_os = "linux") { "DOCX Compressor" } else { "DOCX 图片压缩器" },
        native_options,
        Box::new(|cc| Box::new(DocxCompressorApp::new(cc))),
    )
}

