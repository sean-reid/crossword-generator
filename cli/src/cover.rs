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

    /// Calculate full cover dimensions with bleed
    /// KDP requires 0.125" bleed on all sides
    pub fn calculate_cover_dimensions(&self, color: bool) -> CoverDimensions {
        let spine_width = self.calculate_spine_width(color);
        let bleed = 0.125;
        
        // Total width = bleed + back + spine + front + bleed
        let width = bleed + self.trim_width + spine_width + self.trim_width + bleed;
        // Total height = bleed + height + bleed
        let height = bleed + self.trim_height + bleed;
        
        CoverDimensions {
            total_width: width,
            total_height: height,
            spine_width,
            back_cover_width: self.trim_width,
            front_cover_width: self.trim_width,
            bleed,
        }
    }

    /// Generate paperback cover by modifying template SVG
    pub fn generate_paperback_cover(
        &self,
        template_path: &str,
        title: &str,
        subtitle: &str,
        author: &str,
        color: bool,
    ) -> Result<String> {
        let mut svg = fs::read_to_string(template_path)?;
        let dims = self.calculate_cover_dimensions(color);
        
        // Convert to pixels (assuming 96 DPI for SVG)
        let px_spine = (dims.spine_width * 96.0) as u32;
        
        // ONLY update spine width - leave everything else as-is
        svg = svg.replace("width=\"40.5\"", &format!("width=\"{}\"", px_spine));
        
        // Replace title text
        svg = svg.replace("CROSSWORD", title);
        svg = svg.replace("PUZZLES", subtitle);
        
        // Replace author
        svg = svg.replace("BY SEAN REID", &format!("BY {}", author.to_uppercase()));
        
        Ok(svg)
    }

    /// Generate ebook cover (simpler - no spine)
    pub fn generate_ebook_cover(
        &self,
        template_path: &str,
        title: &str,
        subtitle: &str,
        author: &str,
    ) -> Result<String> {
        let mut svg = fs::read_to_string(template_path)?;
        
        // Replace title text
        svg = svg.replace("CROSSWORD", title);
        svg = svg.replace("PUZZLES", subtitle);
        
        // Replace author
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
