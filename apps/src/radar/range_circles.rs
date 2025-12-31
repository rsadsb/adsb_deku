use ratatui::widgets::canvas::{Circle, Context};

use crate::Settings;

/// Draw range circles around the receiver location
pub fn draw_range_circles(ctx: &mut Context<'_>, settings: &Settings) {
    // Skip drawing if disabled
    if settings.opts.disable_range_circles {
        return;
    }

    // Get the range circles from the command line options
    let ranges = &settings.opts.range_circles.0;

    // Get the receiver location (0,0) in the canvas coordinates
    let (x, y) = settings.to_xy(settings.lat, settings.long);

    // Draw each range circle
    for &range in ranges {
        let lat_offset = range / 111.0;
        let point_at_range = settings.to_xy(settings.lat + lat_offset, settings.long);

        // Calculate the radius in canvas units
        let radius = ((point_at_range.1 - y).powi(2) + (point_at_range.0 - x).powi(2)).sqrt();

        // Draw the circle
        ctx.draw(&Circle { x, y, radius, color: settings.theme.range_circles });

        let label_x = x;
        let label_y = y - radius;
        ctx.print(
            label_x,
            label_y,
            ratatui::text::Span::styled(
                format!("{}km", range),
                ratatui::style::Style::default().fg(settings.theme.range_labels),
            ),
        );
    }
}
