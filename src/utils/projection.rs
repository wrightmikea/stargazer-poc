//! Coordinate projection utilities
//!
//! Handles transformation between celestial coordinates and screen coordinates
//! using equirectangular (plate carrée) projection for the proof of concept.

use crate::data::CelestialCoord;

/// Screen/viewport coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenCoord {
    pub x: f64,
    pub y: f64,
}

impl ScreenCoord {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another point
    pub fn distance(&self, other: &ScreenCoord) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Viewport definition for the star map
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Width of the viewport in pixels
    pub width: f64,
    /// Height of the viewport in pixels
    pub height: f64,
    /// Center RA in hours (0-24)
    pub center_ra: f64,
    /// Center Dec in degrees (-90 to +90)
    pub center_dec: f64,
    /// Zoom level (1.0 = full sky, higher = zoomed in)
    pub zoom: f64,
}

impl Viewport {
    /// Create a new viewport
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            center_ra: 12.0, // Default to center of RA range
            center_dec: 0.0, // Default to celestial equator
            zoom: 1.0,
        }
    }

    /// Field of view in RA (hours)
    pub fn fov_ra(&self) -> f64 {
        24.0 / self.zoom
    }

    /// Field of view in Dec (degrees)
    pub fn fov_dec(&self) -> f64 {
        180.0 / self.zoom
    }

    /// Get the RA range visible in this viewport
    pub fn ra_range(&self) -> (f64, f64) {
        let half_fov = self.fov_ra() / 2.0;
        let min = (self.center_ra - half_fov + 24.0) % 24.0;
        let max = (self.center_ra + half_fov) % 24.0;
        (min, max)
    }

    /// Get the Dec range visible in this viewport
    pub fn dec_range(&self) -> (f64, f64) {
        let half_fov = self.fov_dec() / 2.0;
        let min = (self.center_dec - half_fov).clamp(-90.0, 90.0);
        let max = (self.center_dec + half_fov).clamp(-90.0, 90.0);
        (min, max)
    }

    /// Pan the viewport by a delta in pixels
    pub fn pan(&mut self, dx: f64, dy: f64) {
        // Convert pixel delta to coordinate delta
        let ra_per_pixel = self.fov_ra() / self.width;
        let dec_per_pixel = self.fov_dec() / self.height;

        // Note: RA increases to the left (west), so we negate dx
        self.center_ra = (self.center_ra - dx * ra_per_pixel + 24.0) % 24.0;
        self.center_dec = (self.center_dec + dy * dec_per_pixel).clamp(-90.0, 90.0);
    }

    /// Zoom by a factor, optionally around a point
    pub fn zoom_by(&mut self, factor: f64, anchor: Option<ScreenCoord>) {
        let _old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(1.0, 50.0);

        // If anchor provided, adjust center to keep that point stationary
        if let Some(anchor) = anchor {
            let anchor_coord = self.screen_to_celestial(anchor);
            if let Some(coord) = anchor_coord {
                // Recalculate offset after zoom
                let new_screen = self.celestial_to_screen(&coord);
                let dx = anchor.x - new_screen.x;
                let dy = anchor.y - new_screen.y;
                self.pan(-dx, -dy);
            }
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(1200.0, 600.0)
    }
}

/// Projection trait for converting between coordinate systems
pub trait Projection {
    /// Convert celestial coordinates to screen coordinates
    fn celestial_to_screen(&self, coord: &CelestialCoord) -> ScreenCoord;

    /// Convert screen coordinates to celestial coordinates
    fn screen_to_celestial(&self, screen: ScreenCoord) -> Option<CelestialCoord>;
}

impl Projection for Viewport {
    fn celestial_to_screen(&self, coord: &CelestialCoord) -> ScreenCoord {
        // Equirectangular projection
        let (ra_min, _) = self.ra_range();
        let (_dec_min, dec_max) = self.dec_range();

        // Handle RA wrap-around
        let mut ra_offset = coord.ra - ra_min;

        // For full-sky view (zoom <= 1), use simple linear mapping
        // For zoomed views, handle wrap-around
        if self.zoom > 1.01 {
            if ra_offset < -12.0 {
                ra_offset += 24.0;
            } else if ra_offset > 12.0 {
                ra_offset -= 24.0;
            }
        } else {
            // Full sky: ensure positive offset for linear mapping
            if ra_offset < 0.0 {
                ra_offset += 24.0;
            }
        }

        // RA increases to the right in our projection (traditional star chart style)
        // Note: Some star charts have RA increasing to the left; adjust if needed
        let x = (ra_offset / self.fov_ra()) * self.width;

        // Dec: higher values at top
        let dec_offset = dec_max - coord.dec;
        let y = (dec_offset / self.fov_dec()) * self.height;

        ScreenCoord::new(x, y)
    }

    fn screen_to_celestial(&self, screen: ScreenCoord) -> Option<CelestialCoord> {
        let (ra_min, _) = self.ra_range();
        let (_, dec_max) = self.dec_range();

        // Inverse of celestial_to_screen
        let ra_offset = (screen.x / self.width) * self.fov_ra();
        let ra = (ra_min + ra_offset + 24.0) % 24.0;

        let dec_offset = (screen.y / self.height) * self.fov_dec();
        let dec = dec_max - dec_offset;

        if (-90.0..=90.0).contains(&dec) {
            Some(CelestialCoord::new_wrapped(ra, dec))
        } else {
            None
        }
    }
}

/// Level-of-detail settings for progressive rendering
#[derive(Debug, Clone, Copy)]
pub struct LodSettings {
    /// Minimum magnitude to show at zoom level 1.0
    pub base_magnitude: f64,
    /// Additional magnitude depth per zoom level
    pub magnitude_per_zoom: f64,
    /// Maximum magnitude to ever show
    pub max_magnitude: f64,
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            base_magnitude: 4.0,
            magnitude_per_zoom: 0.5,
            max_magnitude: 6.5,
        }
    }
}

impl LodSettings {
    /// Get the magnitude limit for a given zoom level
    pub fn magnitude_limit(&self, zoom: f64) -> f64 {
        let extra = (zoom - 1.0) * self.magnitude_per_zoom;
        (self.base_magnitude + extra).min(self.max_magnitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let vp = Viewport::new(1200.0, 600.0);
        assert_eq!(vp.width, 1200.0);
        assert_eq!(vp.height, 600.0);
    }

    #[test]
    fn test_projection_roundtrip() {
        let vp = Viewport::default();
        let original = CelestialCoord::new(12.0, 0.0);

        let screen = vp.celestial_to_screen(&original);
        let back = vp.screen_to_celestial(screen).unwrap();

        assert!((original.ra - back.ra).abs() < 0.01);
        assert!((original.dec - back.dec).abs() < 0.01);
    }

    #[test]
    fn test_viewport_pan() {
        let mut vp = Viewport::default();
        let initial_ra = vp.center_ra;

        vp.pan(100.0, 0.0);
        assert_ne!(vp.center_ra, initial_ra);
    }

    #[test]
    fn test_viewport_zoom() {
        let mut vp = Viewport::default();
        vp.zoom_by(2.0, None);
        assert_eq!(vp.zoom, 2.0);

        // Test zoom limits
        vp.zoom_by(100.0, None);
        assert_eq!(vp.zoom, 50.0);
    }

    #[test]
    fn test_lod_settings() {
        let lod = LodSettings::default();

        let mag1 = lod.magnitude_limit(1.0);
        let mag2 = lod.magnitude_limit(5.0);

        assert!(mag2 > mag1);
        assert!(mag2 <= lod.max_magnitude);
    }

    #[test]
    fn test_screen_distance() {
        let p1 = ScreenCoord::new(0.0, 0.0);
        let p2 = ScreenCoord::new(3.0, 4.0);

        assert!((p1.distance(&p2) - 5.0).abs() < 0.001);
    }
}
