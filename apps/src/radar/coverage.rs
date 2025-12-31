use adsb_deku::cpr::Position;
use adsb_deku::ICAO;
use ratatui::layout::Rect;
use ratatui::widgets::canvas::{Canvas, Points};
use rsadsb_common::Airplanes;

use crate::range_circles::draw_range_circles;
use crate::{draw_locations, Settings, MAX_PLOT_HIGH, MAX_PLOT_LOW};

/// Accuracy of latitude/longitude for Coverage is affected by this variable.
///
/// ie: 83.912345 -> 83.91. This is specifically so we get more results hitting in the same
/// position for the sake of an usable heatmap
const COVERAGE_MASK: f64 = 100.0;

// Add to the coverage tab data structure `coverage_airplanes`.
//
// Two events cause an addition:
// 1: New plot from a lat/long position that didn't exist before
// 2: New ICAO(plane) at a previously seen location
pub fn populate_coverage(
    adsb_airplanes: &Airplanes,
    coverage_airplanes: &mut Vec<(f64, f64, u32, ICAO)>,
) {
    let all_position = adsb_airplanes.all_position();
    for (all_icao, Position { latitude, longitude, .. }) in all_position {
        let latitude = (latitude * COVERAGE_MASK).round() / COVERAGE_MASK;
        let longitude = (longitude * COVERAGE_MASK).round() / COVERAGE_MASK;

        // Add number to seen number if found already
        let mut found = false;
        for (lat, long, seen_number, icao) in coverage_airplanes.iter_mut() {
            // Reduce the precision of the coverage/heatmap display (XX.XX)
            //
            // This is so that more airplanes are seen as being in the same spot and are
            // colored so that is made clear to the user. If this is to accurate you will never
            // see airplanes in the "same" spot
            let lat = (*lat * COVERAGE_MASK).round() / COVERAGE_MASK;
            let long = (*long * COVERAGE_MASK).round() / COVERAGE_MASK;

            // Found already, but it is a diff icao? if so, update to new icao and add to
            // seen_number for the color to be more "white" later on
            if (latitude, longitude) == (lat, long) && (all_icao != *icao) {
                *seen_number += 1;
                *icao = all_icao;
                found = true;
                break;
            }

            if (latitude, longitude) == (lat, long) {
                found = true;
                break;
            }
        }

        // If an airplane wasn't seen in this position, add a new entry
        if !found {
            coverage_airplanes.push((latitude, longitude, 0, all_icao));
        }
    }
}

/// Render Coverage tab for tui display
pub fn build_tab_coverage(
    f: &mut ratatui::Frame,
    chunks: &[Rect],
    settings: &Settings,
    coverage_airplanes: &[(f64, f64, u32, ICAO)],
) {
    use ratatui::style::Style;
    use ratatui::widgets::Block;

    let canvas = Canvas::default()
        .block(
            Block::bordered()
                .title("Coverage")
                .title_style(Style::default().fg(settings.theme.title))
                .border_style(Style::default().fg(settings.theme.border)),
        )
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            // draw locations
            draw_locations(ctx, settings);

            // draw range circles
            draw_range_circles(ctx, settings);

            // draw ADSB tab airplanes
            for (lat, long, seen_number, _) in coverage_airplanes.iter() {
                let (x, y) = settings.to_xy(*lat, *long);

                // draw dot on location
                ctx.draw(&Points {
                    coords: &[(x, y)],
                    color: settings.theme.coverage_color(*seen_number),
                });
            }
        });
    f.render_widget(canvas, chunks[1]);
}
