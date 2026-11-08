//! Pure, DPI-aware window positioning. No config or platform access here.
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub primary: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub reason: &'static str,
    pub monitor_index: usize,
    pub physical_width: u32,
    pub physical_height: u32,
}

/// Preserve the right edge using measured physical outer widths, including
/// negative desktop coordinates and DPI rounding performed by the window host.
pub fn right_anchored_x(x: i32, old_width: u32, new_width: u32) -> i32 {
    (i64::from(x) + i64::from(old_width) - i64::from(new_width))
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// Prefer the existing right edge, then the left edge, then fit inside the
/// current monitor work area. All arguments are physical outer coordinates.
pub fn fit_resize_x(
    x: i32,
    old_width: u32,
    new_width: u32,
    work_left: i32,
    work_width: u32,
) -> i32 {
    let left = i64::from(work_left);
    let max_x = (left + i64::from(work_width) - i64::from(new_width)).max(left);
    let right_anchored = i64::from(x) + i64::from(old_width) - i64::from(new_width);
    let x = i64::from(x);
    let fits = |candidate| candidate >= left && candidate <= max_x;
    let placed = if new_width > work_width {
        left
    } else if fits(right_anchored) {
        right_anchored
    } else if fits(x) {
        x
    } else {
        right_anchored.clamp(left, max_x)
    };
    placed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[derive(Debug, PartialEq)]
pub struct PanelPlacement {
    pub y: i32,
    pub panels_above: bool,
}

pub fn player_strip_y(y: i32, outer_height: u32, strip_height: u32, panels_above: bool) -> i32 {
    let offset = if panels_above {
        outer_height.saturating_sub(strip_height)
    } else {
        0
    };
    (i64::from(y) + i64::from(offset)).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub fn place_panel(
    strip_y: i32,
    strip_height: u32,
    outer_height: u32,
    work_top: i32,
    work_height: u32,
) -> PanelPlacement {
    if outer_height <= strip_height {
        return PanelPlacement {
            y: strip_y,
            panels_above: false,
        };
    }
    let top = i64::from(work_top);
    let bottom = top + i64::from(work_height);
    let strip = i64::from(strip_y);
    let height = i64::from(outer_height);
    let up = strip - i64::from(outer_height - strip_height);
    let below_fits = strip >= top && strip + height <= bottom;
    let above_fits = up >= top && strip + i64::from(strip_height) <= bottom;
    let panels_above =
        !below_fits && (above_fits || strip - top > bottom - strip - i64::from(strip_height));
    let y = if panels_above { up } else { strip };
    PanelPlacement {
        y: y.clamp(top, (bottom - height).max(top))
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        panels_above,
    }
}

pub fn plan_position(
    saved: Option<(i32, i32)>,
    monitors: &[MonitorArea],
    logical: (f64, f64),
) -> Option<Placement> {
    if !logical.0.is_finite() || !logical.1.is_finite() || logical.0 <= 0. || logical.1 <= 0. {
        return None;
    }
    let valid =
        |m: &&MonitorArea| m.width > 0 && m.height > 0 && m.scale.is_finite() && m.scale > 0.;
    let chosen = saved.and_then(|(x, y)| {
        monitors.iter().enumerate().find(|(_, m)| {
            valid(m)
                && i64::from(x) >= i64::from(m.x)
                && i64::from(y) >= i64::from(m.y)
                && i64::from(x) < i64::from(m.x) + i64::from(m.width)
                && i64::from(y) < i64::from(m.y) + i64::from(m.height)
        })
    });
    let (index, m, restore) = if let Some((index, monitor)) = chosen {
        (index, monitor, true)
    } else {
        let (index, monitor) = monitors
            .iter()
            .enumerate()
            .find(|(_, m)| valid(m) && m.primary)
            .or_else(|| monitors.iter().enumerate().find(|(_, m)| valid(m)))?;
        (index, monitor, false)
    };
    let width = (logical.0 * m.scale).round().clamp(1., u32::MAX as f64) as u32;
    let height = (logical.1 * m.scale).round().clamp(1., u32::MAX as f64) as u32;
    let left = i64::from(m.x);
    let top = i64::from(m.y);
    let max_x = (left + i64::from(m.width) - i64::from(width)).max(left);
    let max_y = (top + i64::from(m.height) - i64::from(height)).max(top);
    let (x, y, reason) = if restore {
        let (x, y) = saved.unwrap();
        let clamped_x = i64::from(x).clamp(left, max_x);
        let clamped_y = i64::from(y).clamp(top, max_y);
        (
            clamped_x,
            clamped_y,
            if clamped_x == i64::from(x) && clamped_y == i64::from(y) {
                "restored"
            } else {
                "clamped"
            },
        )
    } else {
        (
            left + (max_x - left) / 2,
            top + (max_y - top) / 2,
            if saved.is_some() {
                "centered-offscreen"
            } else {
                "centered-default"
            },
        )
    };
    Some(Placement {
        x: x.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        y: y.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        reason,
        monitor_index: index,
        physical_width: width,
        physical_height: height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn primary() -> MonitorArea {
        MonitorArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale: 1.,
            primary: true,
        }
    }
    #[test]
    fn compact_resize_preserves_right_edge_and_roundtrips_at_multiple_dpi() {
        for (full, compact) in [(725, 102), (1088, 153), (1450, 204)] {
            for x in [200, -1500] {
                let collapsed = right_anchored_x(x, full, compact);
                assert_eq!(
                    i64::from(collapsed) + i64::from(compact),
                    i64::from(x) + i64::from(full)
                );
                assert_eq!(right_anchored_x(collapsed, compact, full), x);
                assert_eq!(right_anchored_x(x, full, full), x);
            }
        }
    }
    #[test]
    fn compact_moved_to_monitor_right_edge_expands_inward() {
        for right in [1920, 0, -1920] {
            let x = right - 153;
            let expanded = right_anchored_x(x, 153, 1088);
            assert_eq!(expanded, right - 1088);
            assert_eq!(right_anchored_x(expanded, 1088, 153), x);
        }
        assert_eq!(right_anchored_x(i32::MIN, 1, u32::MAX), i32::MIN);
        assert_eq!(right_anchored_x(i32::MAX, u32::MAX, 1), i32::MAX);
    }

    #[test]
    fn panels_use_monitor_work_area_and_keep_the_strip_anchor() {
        for (strip, settings, eq, bottom) in [
            (36, 258, 257, 1040),
            (54, 387, 386, 1560),
            (72, 516, 514, 2080),
        ] {
            let y = bottom - strip as i32;
            for height in [settings, eq] {
                let placed = place_panel(y, strip, height, 0, bottom as u32);
                assert!(placed.panels_above);
                assert_eq!(placed.y + height as i32, bottom);
                assert_eq!(player_strip_y(placed.y, height, strip, true), y);
                assert_eq!(
                    place_panel(y, strip, strip, 0, bottom as u32),
                    PanelPlacement {
                        y,
                        panels_above: false
                    }
                );
            }
        }
        assert_eq!(
            place_panel(100, 36, 258, 0, 1040),
            PanelPlacement {
                y: 100,
                panels_above: false
            }
        );
        // The collapsed window includes its 13px non-client frame. Only
        // the 333 extra client pixels are panel content at 150% DPI.
        let framed = place_panel(2021, 67, 400, 0, 2088);
        assert_eq!(
            framed,
            PanelPlacement {
                y: 1688,
                panels_above: true
            }
        );
        assert_eq!(player_strip_y(framed.y, 387, 54, true), 2021);
        let negative = place_panel(-76, 36, 258, -1080, 1040);
        assert_eq!(
            negative,
            PanelPlacement {
                y: -298,
                panels_above: true
            }
        );
        let constrained = place_panel(80, 36, 258, 0, 160);
        assert_eq!(constrained.y, 0);
    }

    #[test]
    fn expansion_chooses_the_fitting_horizontal_edge_at_multiple_dpi() {
        for (compact, full) in [(102, 725), (175, 1110), (204, 1450)] {
            for left in [0, -3840, 48] {
                let width = 3840u32;
                assert_eq!(fit_resize_x(left, compact, full, left, width), left);
                let compact_at_right = left + (width - compact) as i32;
                assert_eq!(
                    fit_resize_x(compact_at_right, compact, full, left, width),
                    left + (width - full) as i32
                );
                let middle = left + 1600;
                assert_eq!(
                    fit_resize_x(middle, compact, full, left, width),
                    right_anchored_x(middle, compact, full)
                );
                assert_eq!(
                    fit_resize_x(left, full, compact, left, width),
                    right_anchored_x(left, full, compact)
                );
            }
        }
    }
    #[test]
    fn horizontal_fallback_keeps_resizable_windows_in_the_work_area() {
        for left in [-1920, 0, 1920] {
            for x in (left - 200..left + 2120).step_by(37) {
                for width in [725, 1088, 1800, 1920] {
                    let placed = fit_resize_x(x, 153, width, left, 1920);
                    assert!(placed >= left);
                    assert!(i64::from(placed) + i64::from(width) <= i64::from(left) + 1920);
                }
            }
        }
        assert_eq!(fit_resize_x(900, 100, 1800, 0, 1920), 0);
        assert_eq!(fit_resize_x(100, 100, 2400, -1920, 1920), -1920);
        assert_eq!(fit_resize_x(i32::MIN, 1, 725, 0, 1920), 0);
        assert_eq!(fit_resize_x(i32::MAX, u32::MAX, 725, 0, 1920), 1195);
    }

    #[test]
    fn absent_geometry_centers_before_show() {
        let p = plan_position(None, &[primary()], (725., 36.)).unwrap();
        assert_eq!((p.x, p.y, p.reason), (597, 522, "centered-default"));
    }
    #[test]
    fn valid_geometry_is_retained() {
        let p = plan_position(Some((120, 100)), &[primary()], (725., 36.)).unwrap();
        assert_eq!((p.x, p.y, p.reason), (120, 100, "restored"));
    }
    #[test]
    fn offscreen_and_extreme_positions_center_safely() {
        for saved in [(1_000_000, -1_000_000), (i32::MAX, i32::MIN)] {
            let p = plan_position(Some(saved), &[primary()], (725., 36.)).unwrap();
            assert_eq!((p.x, p.y, p.reason), (597, 522, "centered-offscreen"));
        }
    }
    #[test]
    fn partial_offscreen_bar_is_clamped_without_changing_size() {
        let p = plan_position(Some((1900, 1050)), &[primary()], (725., 36.)).unwrap();
        assert_eq!(
            (p.x, p.y, p.reason, p.physical_width),
            (1195, 1044, "clamped", 725)
        );
    }
    #[test]
    fn negative_monitor_and_mixed_dpi_are_supported() {
        let left = MonitorArea {
            x: -1920,
            scale: 1.5,
            primary: false,
            ..primary()
        };
        let monitors = [primary(), left];
        let p = plan_position(Some((-1500, 100)), &monitors, (725., 36.)).unwrap();
        assert_eq!(
            (p.x, p.y, p.physical_width, p.physical_height),
            (-1500, 100, 1088, 54)
        );
        assert_eq!(p.reason, "restored");
        let p = plan_position(Some((-900, 100)), &monitors, (725., 36.)).unwrap();
        assert_eq!((p.x, p.reason), (-1088, "clamped"));
    }
    #[test]
    fn oversized_and_absent_monitors_do_not_overflow() {
        let p = plan_position(None, &[primary()], (3000., 36.)).unwrap();
        assert_eq!(p.x, 0);
        assert!(plan_position(None, &[], (725., 36.)).is_none());
        assert!(plan_position(None, &[primary()], (f64::NAN, 36.)).is_none());
    }
}
