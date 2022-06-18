use rsadsb_common::{AirplaneDetails, Airplanes};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders};

use crate::{draw_lines, draw_locations, Settings, DEFAULT_PRECISION, MAX_PLOT_HIGH, MAX_PLOT_LOW};

/// Render Map tab for tui display
pub fn build_tab_map<A: tui::backend::Backend>(
    f: &mut tui::Frame<A>,
    chunks: Vec<Rect>,
    settings: &Settings,
    adsb_airplanes: &Airplanes,
) {
    let canvas = Canvas::default()
        .block(Block::default().title("Map").borders(Borders::ALL))
        .x_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .y_bounds([MAX_PLOT_LOW, MAX_PLOT_HIGH])
        .paint(|ctx| {
            draw_lines(ctx);

            // draw locations
            draw_locations(ctx, settings);

            // draw ADSB tab airplanes
            for key in adsb_airplanes.keys() {
                let aircraft_details = adsb_airplanes.aircraft_details(*key);
                if let Some(AirplaneDetails {
                    position,
                    heading,
                    track,
                    ..
                }) = aircraft_details
                {
                    let (x, y) = settings.to_xy(position.latitude, position.longitude);

                    // draw previous positions ("track")
                    if !settings.opts.disable_track {
                        if let Some(track) = track {
                            for coor in track {
                                if let Some(position) = coor.position {
                                    let (x, y) =
                                        settings.to_xy(position.latitude, position.longitude);

                                    // draw dot on location
                                    ctx.draw(&Points {
                                        coords: &[(x, y)],
                                        color: Color::White,
                                    });
                                }
                            }
                        }
                    }

                    // make wings for the angle directions facing toward the heading. This tried to
                    // account for the angles not showing up around the 90 degree mark, of which I
                    // add degrees of the angle before displaying
                    if !settings.opts.disable_heading {
                        if let Some(heading) = heading {
                            const ANGLE: f32 = 20.0;
                            const LENGTH: f32 = 8.0;

                            let addition_heading = (heading % 90.0) / 10.0;
                            let angle: f32 = ANGLE + addition_heading;

                            let heading = heading + 180.0 % 360.0;
                            // wrap around the angle since we are are subtracting
                            let n_heading = if heading > angle {
                                heading - angle
                            } else {
                                (360.0 + heading) - angle
                            };

                            // move the first point out, so that the green point of the aircraft
                            // _usually_ shows.
                            let y_1 = y + f64::from(2.0 * (n_heading.to_radians()).cos());
                            let x_1 = x + f64::from(2.0 * (n_heading.to_radians()).sin());

                            // draw the line out from the aircraft at an angle
                            let y_2 = y + f64::from(LENGTH * (n_heading.to_radians()).cos());
                            let x_2 = x + f64::from(LENGTH * (n_heading.to_radians()).sin());

                            ctx.draw(&Line {
                                x1: x_1,
                                x2: x_2,
                                y1: y_1,
                                y2: y_2,
                                color: Color::Blue,
                            });

                            // repeat for the other side (addition, so just modding)
                            let n_heading = (heading + angle) % 360.0;
                            let y_1 = y + f64::from(2.0 * (n_heading.to_radians()).cos());
                            let x_1 = x + f64::from(2.0 * (n_heading.to_radians()).sin());
                            let y_2 = y + f64::from(LENGTH * (n_heading.to_radians()).cos());
                            let x_2 = x + f64::from(LENGTH * (n_heading.to_radians()).sin());

                            ctx.draw(&Line {
                                x1: x_1,
                                x2: x_2,
                                y1: y_1,
                                y2: y_2,
                                color: Color::Blue,
                            });
                        }
                    }

                    let name = if settings.opts.disable_lat_long {
                        format!("{key}").into_boxed_str()
                    } else {
                        format!(
                            "{key} ({:.DEFAULT_PRECISION$}, {:.DEFAULT_PRECISION$})",
                            position.latitude, position.longitude
                        )
                        .into_boxed_str()
                    };

                    if !settings.opts.disable_icao {
                        // draw plane ICAO name
                        ctx.print(
                            x,
                            y + 20.0,
                            Span::styled(name.to_string(), Style::default().fg(Color::White)),
                        );
                    }

                    // draw dot on actual lat/lon
                    ctx.draw(&Points {
                        coords: &[(x, y)],
                        color: Color::Blue,
                    });
                }
            }
        });
    f.render_widget(canvas, chunks[1]);
}
