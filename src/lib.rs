use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// Grid configuration
const GRID_SIZE: u32 = 256; // 256x256 = 65536 for BMP
const CELL_SIZE: f64 = 20.0; // Base cell size at zoom 1.0

#[wasm_bindgen]
pub struct UnicodeExplorer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
    // View state
    offset_x: f64,
    offset_y: f64,
    zoom: f64,
    // Plane (0 = BMP, 1-16 for supplementary)
    plane: u32,
    // Interaction state
    dragging: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
    // Selected character
    selected_codepoint: Option<u32>,
}

#[wasm_bindgen]
impl UnicodeExplorer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<UnicodeExplorer, JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        Ok(UnicodeExplorer {
            canvas,
            ctx,
            width,
            height,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
            plane: 0,
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            selected_codepoint: None,
        })
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
    }

    pub fn set_plane(&mut self, plane: u32) {
        if plane <= 16 {
            self.plane = plane;
        }
    }

    pub fn get_plane(&self) -> u32 {
        self.plane
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    pub fn get_zoom(&self) -> f64 {
        self.zoom
    }

    pub fn zoom_at(&mut self, x: f64, y: f64, delta: f64) {
        let old_zoom = self.zoom;
        let zoom_factor = if delta > 0.0 { 0.9 } else { 1.1 };
        self.zoom = (self.zoom * zoom_factor).clamp(0.1, 10.0);

        // Adjust offset to zoom towards mouse position
        let scale_change = self.zoom / old_zoom;
        self.offset_x = x - (x - self.offset_x) * scale_change;
        self.offset_y = y - (y - self.offset_y) * scale_change;
    }

    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.dragging = true;
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    pub fn drag(&mut self, x: f64, y: f64) {
        if self.dragging {
            self.offset_x += x - self.last_mouse_x;
            self.offset_y += y - self.last_mouse_y;
            self.last_mouse_x = x;
            self.last_mouse_y = y;
        }
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn click(&mut self, x: f64, y: f64) -> Option<u32> {
        let cell_size = CELL_SIZE * self.zoom;
        let grid_x = ((x - self.offset_x) / cell_size).floor() as i32;
        let grid_y = ((y - self.offset_y) / cell_size).floor() as i32;

        if grid_x >= 0 && grid_x < GRID_SIZE as i32 && grid_y >= 0 && grid_y < GRID_SIZE as i32 {
            let codepoint = (self.plane * 0x10000) + (grid_y as u32 * GRID_SIZE) + grid_x as u32;
            self.selected_codepoint = Some(codepoint);
            Some(codepoint)
        } else {
            self.selected_codepoint = None;
            None
        }
    }

    pub fn get_selected(&self) -> Option<u32> {
        self.selected_codepoint
    }

    pub fn render(&self) {
        let ctx = &self.ctx;
        let cell_size = CELL_SIZE * self.zoom;

        // Clear canvas
        ctx.set_fill_style_str("#1a1a2e");
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        // Calculate visible range
        let start_col = ((-self.offset_x) / cell_size).floor().max(0.0) as u32;
        let start_row = ((-self.offset_y) / cell_size).floor().max(0.0) as u32;
        let end_col = ((self.width - self.offset_x) / cell_size).ceil().min(GRID_SIZE as f64) as u32;
        let end_row = ((self.height - self.offset_y) / cell_size).ceil().min(GRID_SIZE as f64) as u32;

        // Set font based on zoom
        let font_size = (cell_size * 0.6).max(8.0).min(32.0);
        ctx.set_font(&format!("{}px sans-serif", font_size));
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");

        for row in start_row..end_row {
            for col in start_col..end_col {
                let codepoint = (self.plane * 0x10000) + (row * GRID_SIZE) + col;
                let x = self.offset_x + (col as f64 * cell_size);
                let y = self.offset_y + (row as f64 * cell_size);

                // Get category color
                let color = get_category_color(codepoint);
                ctx.set_fill_style_str(color);
                ctx.fill_rect(x + 1.0, y + 1.0, cell_size - 2.0, cell_size - 2.0);

                // Draw character if zoom is sufficient
                if self.zoom >= 0.5 {
                    if let Some(ch) = char::from_u32(codepoint) {
                        ctx.set_fill_style_str("#ffffff");
                        let _ = ctx.fill_text(
                            &ch.to_string(),
                            x + cell_size / 2.0,
                            y + cell_size / 2.0,
                        );
                    }
                }

                // Highlight selected
                if self.selected_codepoint == Some(codepoint) {
                    ctx.set_stroke_style_str("#ffcc00");
                    ctx.set_line_width(3.0);
                    ctx.stroke_rect(x + 1.0, y + 1.0, cell_size - 2.0, cell_size - 2.0);
                }
            }
        }

        // Draw grid lines if zoomed in enough
        if self.zoom >= 1.0 {
            ctx.set_stroke_style_str("#333355");
            ctx.set_line_width(0.5);
            
            for col in start_col..=end_col {
                let x = self.offset_x + (col as f64 * cell_size);
                ctx.begin_path();
                ctx.move_to(x, 0.0);
                ctx.line_to(x, self.height);
                ctx.stroke();
            }
            
            for row in start_row..=end_row {
                let y = self.offset_y + (row as f64 * cell_size);
                ctx.begin_path();
                ctx.move_to(0.0, y);
                ctx.line_to(self.width, y);
                ctx.stroke();
            }
        }
    }

    pub fn center_on(&mut self, codepoint: u32) {
        let plane = codepoint / 0x10000;
        let local = codepoint % 0x10000;
        let row = local / GRID_SIZE;
        let col = local % GRID_SIZE;

        self.plane = plane;
        let cell_size = CELL_SIZE * self.zoom;
        self.offset_x = self.width / 2.0 - (col as f64 + 0.5) * cell_size;
        self.offset_y = self.height / 2.0 - (row as f64 + 0.5) * cell_size;
    }
}

fn get_category_color(codepoint: u32) -> &'static str {
    use unic_ucd_category::GeneralCategory;
    
    if let Some(ch) = char::from_u32(codepoint) {
        let cat = GeneralCategory::of(ch);
        match cat {
            GeneralCategory::UppercaseLetter |
            GeneralCategory::LowercaseLetter |
            GeneralCategory::TitlecaseLetter |
            GeneralCategory::ModifierLetter |
            GeneralCategory::OtherLetter => "#2d5a27", // Green - letters
            
            GeneralCategory::DecimalNumber |
            GeneralCategory::LetterNumber |
            GeneralCategory::OtherNumber => "#1a4d7a", // Blue - numbers
            
            GeneralCategory::SpaceSeparator |
            GeneralCategory::LineSeparator |
            GeneralCategory::ParagraphSeparator => "#4a4a4a", // Gray - whitespace
            
            GeneralCategory::Control |
            GeneralCategory::Format |
            GeneralCategory::Surrogate |
            GeneralCategory::PrivateUse => "#5a1a1a", // Dark red - control/special
            
            GeneralCategory::ConnectorPunctuation |
            GeneralCategory::DashPunctuation |
            GeneralCategory::OpenPunctuation |
            GeneralCategory::ClosePunctuation |
            GeneralCategory::InitialPunctuation |
            GeneralCategory::FinalPunctuation |
            GeneralCategory::OtherPunctuation => "#6b4c1a", // Brown - punctuation
            
            GeneralCategory::MathSymbol |
            GeneralCategory::CurrencySymbol |
            GeneralCategory::ModifierSymbol |
            GeneralCategory::OtherSymbol => "#5a1a5a", // Purple - symbols
            
            GeneralCategory::NonspacingMark |
            GeneralCategory::SpacingMark |
            GeneralCategory::EnclosingMark => "#1a5a5a", // Teal - marks
            
            GeneralCategory::Unassigned => "#2a2a2a", // Dark - unassigned
        }
    } else {
        "#2a2a2a" // Invalid codepoint
    }
}

// Helper function to get character info
#[wasm_bindgen]
pub fn get_char_info(codepoint: u32) -> String {
    use unic_ucd_category::GeneralCategory;
    use unic_ucd_block::Block;
    
    let hex = format!("U+{:04X}", codepoint);
    
    if let Some(ch) = char::from_u32(codepoint) {
        let name = unicode_names2::name(ch)
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unnamed>".to_string());
        
        let category = GeneralCategory::of(ch);
        let block = Block::of(ch)
            .map(|b| b.name.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        format!(
            "{{\"codepoint\":\"{}\",\"char\":\"{}\",\"name\":\"{}\",\"category\":\"{:?}\",\"block\":\"{}\"}}",
            hex,
            ch.escape_default(),
            name,
            category,
            block
        )
    } else {
        format!("{{\"codepoint\":\"{}\",\"char\":null,\"name\":\"Invalid\",\"category\":\"Invalid\",\"block\":\"Invalid\"}}", hex)
    }
}

// Search function
#[wasm_bindgen]
pub fn search_characters(query: &str, limit: u32) -> Vec<u32> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    
    // Check if it's a hex codepoint search
    if query_lower.starts_with("u+") || query_lower.starts_with("0x") {
        let hex_str = query_lower.trim_start_matches("u+").trim_start_matches("0x");
        if let Ok(cp) = u32::from_str_radix(hex_str, 16) {
            if char::from_u32(cp).is_some() {
                return vec![cp];
            }
        }
    }
    
    // Search by name
    for cp in 0..=0x10FFFF_u32 {
        if results.len() >= limit as usize {
            break;
        }
        
        if let Some(ch) = char::from_u32(cp) {
            if let Some(name) = unicode_names2::name(ch) {
                if name.to_string().to_lowercase().contains(&query_lower) {
                    results.push(cp);
                }
            }
        }
    }
    
    results
}
