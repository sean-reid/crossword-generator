use anyhow::Result;
use std::fs;

pub struct CoverGenerator {
    page_count: usize,
    trim_width: f32,
    trim_height: f32,
}

impl CoverGenerator {
    pub fn new(page_count: usize, trim_width: f32, trim_height: f32) -> Self {
        Self {
            page_count,
            trim_width,
            trim_height,
        }
    }

    /// Calculate spine width based on page count
    /// Black & white interior: 0.002252" per page
    /// Color interior: 0.002347" per page
    pub fn calculate_spine_width(&self, color: bool) -> f32 {
        let per_page = if color { 0.002347 } else { 0.002252 };
        self.page_count as f32 * per_page
    }

    /// Generate paperback cover - ONLY replaces text
    /// User must design template at correct dimensions using KDP calculator
    pub fn generate_paperback_cover(
        &self,
        template_path: &str,
        title: &str,
        subtitle: &str,
        author: &str,
        color: bool,
    ) -> Result<String> {
        let mut svg = fs::read_to_string(template_path)?;
        
        // Calculate and print spine info for user reference
        let spine_width_in = self.calculate_spine_width(color);
        let spine_width_px = spine_width_in * 96.0;
        
        println!("\n📐 Spine Calculation:");
        println!("   Page count: {}", self.page_count);
        println!("   Spine width: {:.4}\" ({:.1} px at 96 DPI)", spine_width_in, spine_width_px);
        println!("   Interior: {}", if color { "Color" } else { "Black & White" });
        println!("\n   💡 Ensure your template is designed for this spine width");
        println!("      Use: https://kdp.amazon.com/en_US/cover-templates");
        
        // ONLY replace text - preserve all design and dimensions
        svg = svg.replace("CROSSWORD", title);
        svg = svg.replace("PUZZLES", subtitle);
        svg = svg.replace("BY SEAN REID", &format!("BY {}", author.to_uppercase()));
        
        Ok(svg)
    }

    /// Generate ebook cover - ONLY replaces text
    pub fn generate_ebook_cover(
        &self,
        template_path: &str,
        title: &str,
        subtitle: &str,
        author: &str,
    ) -> Result<String> {
        let mut svg = fs::read_to_string(template_path)?;
        
        // ONLY replace text
        svg = svg.replace("CROSSWORD", title);
        svg = svg.replace("PUZZLES", subtitle);
        svg = svg.replace("BY SEAN REID", &format!("BY {}", author.to_uppercase()));
        
        Ok(svg)
    }
}

pub struct CoverDimensions {
    pub total_width: f32,
    pub total_height: f32,
    pub spine_width: f32,
    pub back_cover_width: f32,
    pub front_cover_width: f32,
    pub bleed: f32,
}
