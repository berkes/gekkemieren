//! Prints the dashboard SVG with placeholder values substituted to stdout.
//!
//! Preview the HUD while editing assets/dashboard.svg in an editor:
//!
//! ```sh
//! cargo run --example render_dashboard > preview.svg
//! ```
//!
//! then open preview.svg in Inkscape or any image viewer.

use gekkemieren::dashboard::{DASHBOARD_SVG, DashboardValues, fill_template};

fn main() {
    let values = DashboardValues {
        food_home_total: 1234,
        food_home_per_min: 42.5,
        food_on_canvas: 987,
        food_deposited_total: 5000,
        deaths: 12,
        scout_ratio: 0.2,
    };
    print!("{}", fill_template(DASHBOARD_SVG, &values));
}
