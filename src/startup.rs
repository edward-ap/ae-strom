//! Pure, DPI-aware startup positioning. No config or platform access here.
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
