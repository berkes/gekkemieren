use resvg::{tiny_skia, usvg};

pub const DASHBOARD_SVG: &str = include_str!("../assets/dashboard.svg");

const RENDER_SCALE: u32 = 2;
const BAR_HEIGHT: u32 = 150;
const GAUGE_RADIUS: f64 = 52.0;
const GAUGE_ARC_LENGTH: f64 = 2.0 * std::f64::consts::PI * GAUGE_RADIUS;
const GAUGE_FULL_SCALE_PER_MIN: f64 = 600.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DashboardValues {
    pub food_home_total: u64,
    pub food_home_per_min: f64,
    pub food_on_canvas: u64,
    pub food_deposited_total: u64,
    pub deaths: u64,
}

pub fn fill_template(template: &str, values: &DashboardValues, width: u32, height: u32) -> String {
    let bar_y = height.saturating_sub(BAR_HEIGHT);
    let gauge_cx = width.saturating_sub(110);
    let gauge_cy = bar_y + BAR_HEIGHT / 2;
    let per_min_frac = (values.food_home_per_min / GAUGE_FULL_SCALE_PER_MIN).clamp(0.0, 1.0);
    let gauge_arc_offset = GAUGE_ARC_LENGTH * (1.0 - per_min_frac);

    let replacements = [
        ("{{width}}", width.to_string()),
        ("{{height}}", height.to_string()),
        ("{{bar_y}}", bar_y.to_string()),
        ("{{bar_y_plus_42}}", (bar_y + 42).to_string()),
        ("{{bar_y_plus_102}}", (bar_y + 102).to_string()),
        ("{{bar_y_plus_132}}", (bar_y + 132).to_string()),
        ("{{gauge_cx}}", gauge_cx.to_string()),
        ("{{gauge_cy}}", gauge_cy.to_string()),
        ("{{gauge_cy_plus_6}}", (gauge_cy + 6).to_string()),
        ("{{food_home_total}}", values.food_home_total.to_string()),
        ("{{food_home_per_min}}", format!("{:.1}", values.food_home_per_min)),
        ("{{food_on_canvas}}", values.food_on_canvas.to_string()),
        ("{{food_deposited_total}}", values.food_deposited_total.to_string()),
        ("{{deaths}}", values.deaths.to_string()),
        (
            "{{food_home_arc}}",
            format!("{:.2}", gauge_arc_offset),
        ),
    ];

    let mut svg = template.to_string();
    for (placeholder, value) in replacements {
        svg = svg.replace(placeholder, &value);
    }
    svg
}

pub fn rasterize(svg: &str, width: u32, height: u32) -> Option<tiny_skia::Pixmap> {
    let mut options = usvg::Options::default();
    options
        .fontdb_mut()
        .load_font_data(epaint_default_fonts::HACK_REGULAR.to_vec());
    options.font_family = "Hack".to_string();

    let tree = usvg::Tree::from_str(svg, &options).ok()?;
    let scale = RENDER_SCALE as f32;
    let mut pixmap = tiny_skia::Pixmap::new(width * RENDER_SCALE, height * RENDER_SCALE)?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    Some(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> DashboardValues {
        DashboardValues {
            food_home_total: 1234,
            food_home_per_min: 42.5,
            food_on_canvas: 987,
            food_deposited_total: 5000,
            deaths: 12,
        }
    }

    #[test]
    fn fill_template_replaces_all_placeholders() {
        let svg = fill_template(DASHBOARD_SVG, &values(), 1100, 1100);
        assert!(!svg.contains("{{"), "unreplaced placeholder left: {}", svg);
        assert!(svg.contains("1234"));
        assert!(svg.contains("42.5"));
        assert!(svg.contains("987"));
        assert!(svg.contains("5000"));
        assert!(svg.contains("12"));
        assert!(svg.contains("width=\"1100\""));
    }

    #[test]
    fn gauge_arc_offset_tracks_per_minute_rate() {
        let mut v = values();
        v.food_home_per_min = 0.0;
        let svg = fill_template(DASHBOARD_SVG, &v, 1100, 1100);
        assert!(svg.contains(&format!("stroke-dashoffset=\"{:.2}\"", GAUGE_ARC_LENGTH)));

        v.food_home_per_min = GAUGE_FULL_SCALE_PER_MIN * 2.0;
        let svg = fill_template(DASHBOARD_SVG, &v, 1100, 1100);
        assert!(svg.contains("stroke-dashoffset=\"0.00\""));
    }

    #[test]
    fn rasterize_returns_scaled_pixmap_with_transparent_background() {
        let svg = fill_template(DASHBOARD_SVG, &values(), 200, 200);
        let pixmap = rasterize(&svg, 200, 200).expect("svg should rasterize");
        assert_eq!(pixmap.width(), 200 * RENDER_SCALE);
        assert_eq!(pixmap.height(), 200 * RENDER_SCALE);

        let corner_alpha = pixmap.pixel(0, 0).expect("corner pixel").alpha();
        assert_eq!(corner_alpha, 0, "background outside the bar must be transparent");

        let bar_y = (200 - BAR_HEIGHT) * RENDER_SCALE;
        let bar_alpha = pixmap
            .pixel(100 * RENDER_SCALE, bar_y + 10 * RENDER_SCALE)
            .expect("bar pixel")
            .alpha();
        assert!(bar_alpha > 0, "bottom bar must be painted");
    }

    #[test]
    fn rasterize_rejects_invalid_svg() {
        assert!(rasterize("<svg", 64, 64).is_none());
    }
}
