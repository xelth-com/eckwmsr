use crate::utils::encryption::{eck_crc, eck_url_encrypt, to_base32_char};
use printpdf::{
    BuiltinFont, Color, ColorBits, ColorSpace, Image, ImageTransform, ImageXObject,
    IndirectFontRef, Mm, PdfDocument, PdfLayerReference, Px, Rgb,
};
use qrcode::{EcLevel, QrCode};
use serde::Deserialize;
use std::env;
use std::io::BufWriter;

#[derive(Deserialize, Debug, Clone)]
pub struct ElementConfig {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ContentConfig {
    pub qr1: Option<ElementConfig>,
    pub qr2: Option<ElementConfig>,
    pub qr3: Option<ElementConfig>,
    pub checksum: Option<ElementConfig>,
    pub serial: Option<ElementConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RegalConfig {
    pub index: i32,
    pub columns: i32,
    pub rows: i32,
    pub start_index: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WarehouseConfig {
    pub regals: Vec<RegalConfig>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LabelConfig {
    #[serde(rename = "type")]
    pub label_type: String,
    pub start_number: i64,
    pub count: i32,
    pub cols: i32,
    pub rows: i32,
    pub margin_top: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_bottom: f64,
    pub gap_x: f64,
    pub gap_y: f64,
    pub is_tight_mode: bool,
    pub serial_digits: i32,
    pub content_config: Option<ContentConfig>,
    pub warehouse_config: Option<WarehouseConfig>,
}

fn calculate_warehouse_location(
    place_index: i64,
    config: &WarehouseConfig,
) -> Option<(i32, i32, i32)> {
    for r in &config.regals {
        let places_in_regal = r.columns * r.rows;
        let end_idx = r.start_index + places_in_regal - 1;

        if place_index >= r.start_index as i64 && place_index <= end_idx as i64 {
            let index_in_regal = (place_index as i32) - r.start_index;
            let column = index_in_regal / r.rows;
            let row = index_in_regal % r.rows;
            return Some((r.index, column + 1, row + 1));
        }
    }
    None
}

fn format_serial(num: i64, prefix: &str, serial_digits: i32) -> String {
    let padded = format!("{:018}", num);
    if serial_digits > 0 && serial_digits < 18 {
        let start = 18 - serial_digits as usize;
        return format!("{}{}", prefix, &padded[start..]);
    }
    format!("{}{}", prefix, padded)
}

/// Render a QR code to raw grayscale pixel data and return (pixels, width)
fn render_qr_pixels(data: &str) -> Result<(Vec<u8>, usize), anyhow::Error> {
    let code = QrCode::with_error_correction_level(data, EcLevel::L)?;
    let modules = code.to_colors();
    let module_count = code.width() as usize;

    // Scale up: each QR module becomes scale x scale pixels
    let scale = 4_usize; // 4px per module for good resolution
    let img_size = module_count * scale;
    let mut pixels = vec![255u8; img_size * img_size]; // white background

    for row in 0..module_count {
        for col in 0..module_count {
            let idx = row * module_count + col;
            if modules[idx] == qrcode::Color::Dark {
                // Fill the scaled block with black
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = (row * scale + dy) * img_size + (col * scale + dx);
                        pixels[px] = 0; // black
                    }
                }
            }
        }
    }

    Ok((pixels, img_size))
}

/// Add a QR code image to a PDF layer at the specified position and size (in mm)
fn add_qr_to_layer(
    layer: &PdfLayerReference,
    qr_data: &str,
    pos_x: f64,
    pos_y: f64,
    size_mm: f64,
) -> Result<(), anyhow::Error> {
    let (pixels, img_size) = render_qr_pixels(qr_data)?;

    let xobj = ImageXObject {
        width: Px(img_size),
        height: Px(img_size),
        color_space: ColorSpace::Greyscale,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: pixels,
        image_filter: None,
        smask: None,
        clipping_bbox: None,
    };

    let pdf_image = Image::from(xobj);

    // Calculate scale to achieve desired mm size
    // At 300 DPI: image_pt = img_size / 300 * 72
    // We want size_mm in pt: size_pt = size_mm * 2.8346
    // scale = size_pt / image_pt
    let dpi = 300.0_f32;
    let size_pt = size_mm as f32 * 2.8346;
    let image_pt = img_size as f32 / dpi * 72.0;
    let scale = size_pt / image_pt;

    let transform = ImageTransform {
        translate_x: Some(Mm(pos_x as f32)),
        translate_y: Some(Mm(pos_y as f32)),
        scale_x: Some(scale),
        scale_y: Some(scale),
        dpi: Some(dpi),
        ..Default::default()
    };

    pdf_image.add_to_layer(layer.clone(), transform);
    Ok(())
}

pub fn generate_labels_pdf(mut cfg: LabelConfig) -> Result<Vec<u8>, anyhow::Error> {
    // Defaults (matching Go handler)
    if cfg.cols == 0 {
        cfg.cols = 2;
    }
    if cfg.rows == 0 {
        cfg.rows = 8;
    }
    if cfg.count == 0 {
        cfg.count = cfg.cols * cfg.rows;
    }
    if cfg.label_type.is_empty() {
        cfg.label_type = "i".to_string();
    }
    // Default margins if all zero
    if cfg.margin_top == 0.0
        && cfg.margin_bottom == 0.0
        && cfg.margin_left == 0.0
        && cfg.margin_right == 0.0
    {
        cfg.margin_top = 7.0;
        cfg.margin_bottom = 7.0;
        cfg.margin_left = 7.0;
        cfg.margin_right = 7.0;
    }

    let (doc, page1, layer1) =
        PdfDocument::new("Labels", Mm(210.0), Mm(297.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let page_width: f64 = 210.0;
    let page_height: f64 = 297.0;

    let mut extra_x = 0.0;
    let mut extra_y = 0.0;
    if !cfg.is_tight_mode {
        extra_x = cfg.gap_x / 2.0;
        extra_y = cfg.gap_y / 2.0;
    }

    let eff_margin_left = cfg.margin_left + extra_x;
    let eff_margin_right = cfg.margin_right + extra_x;
    let eff_margin_top = cfg.margin_top + extra_y;
    let eff_margin_bottom = cfg.margin_bottom + extra_y;

    let avail_w = page_width - eff_margin_left - eff_margin_right;
    let avail_h = page_height - eff_margin_top - eff_margin_bottom;

    let total_gap_x = (cfg.cols - 1) as f64 * cfg.gap_x;
    let total_gap_y = (cfg.rows - 1) as f64 * cfg.gap_y;
    let label_w = (avail_w - total_gap_x) / cfg.cols as f64;
    let label_h = (avail_h - total_gap_y) / cfg.rows as f64;

    let instance_suffix = env::var("INSTANCE_SUFFIX").unwrap_or_else(|_| "IB".to_string());
    let labels_per_page = cfg.cols * cfg.rows;

    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let courier_font = doc.add_builtin_font(BuiltinFont::CourierBold)?;

    for i in 0..cfg.count {
        if i > 0 && i % labels_per_page == 0 {
            let (new_page, new_layer) =
                doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(new_page).get_layer(new_layer);
        }

        let index_on_page = i % labels_per_page;
        let col_idx = index_on_page % cfg.cols;
        let row_idx = index_on_page / cfg.cols;

        let origin_x = eff_margin_left + (col_idx as f64) * (label_w + cfg.gap_x);
        let top_y = eff_margin_top + (row_idx as f64) * (label_h + cfg.gap_y);
        let origin_y = page_height - top_y - label_h;

        let current_id = cfg.start_number + (i as i64);
        let id_string = format!("{}{:018}", cfg.label_type, current_id);

        let encrypted_code = eck_url_encrypt(&id_string).unwrap_or_else(|_| id_string.clone());

        let field1 = match cfg.label_type.as_str() {
            "i" => format_serial(current_id, "!", cfg.serial_digits),
            "b" => format_serial(current_id, "#", cfg.serial_digits),
            "p" => format_serial(current_id, "_", cfg.serial_digits),
            "l" => format_serial(current_id, "*", cfg.serial_digits),
            _ => format_serial(current_id, "", cfg.serial_digits),
        };

        let field2 = if cfg.label_type == "p" && cfg.warehouse_config.is_some() {
            if let Some((r, c, row)) =
                calculate_warehouse_location(current_id, cfg.warehouse_config.as_ref().unwrap())
            {
                format!(
                    "{}{}{}",
                    to_base32_char(r as usize),
                    to_base32_char(c as usize),
                    to_base32_char(row as usize)
                )
            } else {
                "???".to_string()
            }
        } else {
            eck_crc(current_id)
        };

        let min_side = label_w.min(label_h);

        if cfg.content_config.is_none() {
            // Default "Master QR Puzzle" layout
            let qr1_scale = 0.85;
            let qr1_size = label_h * qr1_scale;

            // QR1 (Large, left side)
            let qr1_data = format!("ECK1.COM/{}{}", encrypted_code, instance_suffix);
            let qr1_x = origin_x + 2.0;
            let qr1_y = origin_y + (label_h - qr1_size) / 2.0;
            let _ = add_qr_to_layer(&current_layer, &qr1_data, qr1_x, qr1_y, qr1_size);

            // Checksum (Large, center-right)
            let cs_scale = 0.45;
            let cs_size = label_h * cs_scale;
            let cs_x = origin_x + qr1_size + 8.0;
            let cs_y = origin_y + label_h / 2.0 - cs_size / 4.0;
            current_layer.use_text(
                &field2,
                (cs_size * 2.5) as f32,
                Mm(cs_x as f32),
                Mm(cs_y as f32),
                &font,
            );

            // Approximate checksum text width
            let cs_width = field2.len() as f64 * cs_size * 0.6;

            // Serial (Small, below checksum)
            let s_scale = 0.12;
            let s_size = label_h * s_scale;
            let serial_x = origin_x + qr1_size + 8.0;
            let serial_y = origin_y + label_h * 0.25;
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
            current_layer.use_text(
                &field1,
                (s_size * 2.5) as f32,
                Mm(serial_x as f32),
                Mm(serial_y as f32),
                &courier_font,
            );
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));

            // QR2 & QR3 (Small, right side)
            let s_qr_scale = 0.32;
            let s_qr_size = label_h * s_qr_scale;
            let right_x = origin_x + qr1_size + cs_width + 16.0;

            if right_x + s_qr_size < origin_x + label_w {
                let qr2_data = format!("ECK2.COM/{}{}", encrypted_code, instance_suffix);
                let qr2_y = origin_y + label_h - s_qr_size - 3.0;
                let _ = add_qr_to_layer(&current_layer, &qr2_data, right_x, qr2_y, s_qr_size);

                let qr3_data = format!("ECK3.COM/{}{}", encrypted_code, instance_suffix);
                let qr3_y = origin_y + 3.0;
                let _ = add_qr_to_layer(&current_layer, &qr3_data, right_x, qr3_y, s_qr_size);
            }
        } else {
            let cc = cfg.content_config.as_ref().unwrap();

            // Helper: draw text at element position
            let draw_text = |layer: &PdfLayerReference,
                             text: &str,
                             el: &ElementConfig,
                             f: &IndirectFontRef,
                             gray: f64| {
                let size = min_side * el.scale;
                let pos_x = origin_x + (el.x * label_w / 100.0);
                let pos_y = origin_y + (el.y * label_h / 100.0);

                if gray > 0.0 {
                    layer.set_fill_color(Color::Rgb(Rgb::new(
                        gray as f32,
                        gray as f32,
                        gray as f32,
                        None,
                    )));
                }
                layer.use_text(text, (size * 2.5) as f32, Mm(pos_x as f32), Mm(pos_y as f32), f);
                if gray > 0.0 {
                    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                }
            };

            // Draw text first (back layer)
            if let Some(ref cs_cfg) = cc.checksum {
                draw_text(&current_layer, &field2, cs_cfg, &font, 0.0);
            }
            if let Some(ref serial_cfg) = cc.serial {
                draw_text(&current_layer, &field1, serial_cfg, &courier_font, 0.3);
            }

            // Draw QR codes on top
            let draw_qr = |prefix: &str, el: &ElementConfig| {
                let qr_data = format!("{}/{}{}", prefix, encrypted_code, instance_suffix);
                let size = min_side * el.scale;
                let pos_x = origin_x + (el.x * label_w / 100.0);
                let pos_y = origin_y + (el.y * label_h / 100.0);
                let _ = add_qr_to_layer(&current_layer, &qr_data, pos_x, pos_y, size);
            };

            if let Some(ref qr1) = cc.qr1 {
                draw_qr("ECK1.COM", qr1);
            }
            if let Some(ref qr2) = cc.qr2 {
                draw_qr("ECK2.COM", qr2);
            }
            if let Some(ref qr3) = cc.qr3 {
                draw_qr("ECK3.COM", qr3);
            }
        }
    }

    let mut buf = Vec::new();
    doc.save(&mut BufWriter::new(&mut buf))?;
    Ok(buf)
}
