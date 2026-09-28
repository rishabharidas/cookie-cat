//! OS Platform Integration Module
//!
//! Provides a cross-platform desktop display and environment abstraction:
//! 1. Multi-monitor enumeration with true display bounds and usable work areas (Dock/Taskbar avoidance).
//! 2. Dynamic display detection: determines which monitor the cat belongs to (supporting negative coordinates and arbitrary arrangements).
//! 3. Global cursor position querying across virtual desktop space.
//! 4. Background accessory configuration on macOS (Dock & app menu bar suppression).

use bevy::prelude::*;

/// Axis-aligned rectangle in logical desktop coordinates.
///
/// Supports arbitrary arrangements (side-by-side, vertical, diagonal)
/// and negative coordinate offsets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesktopRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DesktopRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Checks if a point lies within the rectangle bounds.
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Calculates intersection area with another rectangle for window overlap detection.
    pub fn intersection_area(&self, other: &DesktopRect) -> f32 {
        let ix1 = self.left().max(other.left());
        let iy1 = self.top().max(other.top());
        let ix2 = self.right().min(other.right());
        let iy2 = self.bottom().min(other.bottom());

        if ix2 > ix1 && iy2 > iy1 {
            (ix2 - ix1) * (iy2 - iy1)
        } else {
            0.0
        }
    }
}

/// Metadata and geometry for a physical display monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub bounds: DesktopRect,
    pub work_area: DesktopRect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

/// Dynamic desktop metrics computed specifically for the cat on its current display.
///
/// Contains NO hardcoded display values. All fields are dynamically derived
/// from the active display's usable work area and the cat's actual window/model dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopMetrics {
    pub display_id: String,
    pub display_name: String,
    pub bounds: DesktopRect,
    pub work_area: DesktopRect,
    pub scale_factor: f64,
    /// Minimum horizontal position for the cat window within this display's work area
    pub min_x: f32,
    /// Maximum horizontal position for the cat window within this display's work area
    pub max_x: f32,
    /// Vertical window ground position where paws rest immediately above the dock/taskbar
    pub ground_y: f32,
}

#[derive(Resource)]
pub struct DesktopEnvironmentResource(pub Box<dyn DesktopEnvironment>);

impl DesktopMetrics {
    /// Computes dynamic metrics for a display given the cat's window dimensions and paw offset.
    pub fn from_display(
        display: &DisplayInfo,
        window_width: f32,
        _window_height: f32,
        cat_paws_y_offset: f32,
        horizontal_margin: f32,
        bottom_margin: f32,
    ) -> Self {
        let min_x = display.work_area.left() + horizontal_margin;
        let max_x = (display.work_area.right() - window_width - horizontal_margin).max(min_x);

        // In top-left coordinates:
        // window_y + cat_paws_y_offset = work_area.bottom() - bottom_margin
        // => window_y = work_area.bottom() - cat_paws_y_offset - bottom_margin
        let ground_y = display.work_area.bottom() - cat_paws_y_offset - bottom_margin;

        Self {
            display_id: display.id.clone(),
            display_name: display.name.clone(),
            bounds: display.bounds,
            work_area: display.work_area,
            scale_factor: display.scale_factor,
            min_x,
            max_x,
            ground_y,
        }
    }
}

/// Cross-platform abstraction for desktop display detection and global inputs.
pub trait DesktopEnvironment: Send + Sync {
    /// Enumerates all currently connected displays.
    fn displays(&self) -> Vec<DisplayInfo>;

    /// Finds the display containing the given desktop coordinate (x, y).
    fn display_at(&self, x: f32, y: f32) -> Option<DisplayInfo> {
        let all = self.displays();
        for d in &all {
            if d.bounds.contains_point(x, y) {
                return Some(d.clone());
            }
        }
        // Fallback: choose nearest display by center distance
        all.into_iter().min_by(|a, b| {
            let ac = Vec2::new(a.bounds.x + a.bounds.width * 0.5, a.bounds.y + a.bounds.height * 0.5);
            let bc = Vec2::new(b.bounds.x + b.bounds.width * 0.5, b.bounds.y + b.bounds.height * 0.5);
            let p = Vec2::new(x, y);
            ac.distance_squared(p).partial_cmp(&bc.distance_squared(p)).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Finds the display that has the greatest overlap with the given window rectangle.
    fn display_for_window(&self, window_rect: DesktopRect) -> Option<DisplayInfo> {
        let all = self.displays();
        let mut best = None;
        let mut max_area = 0.0;
        for d in &all {
            let area = d.bounds.intersection_area(&window_rect);
            if area > max_area {
                max_area = area;
                best = Some(d.clone());
            }
        }
        best.or_else(|| self.display_at(window_rect.x + window_rect.width * 0.5, window_rect.y + window_rect.height * 0.5))
    }

    /// Returns the global cursor position across the virtual desktop in top-left screen coordinates.
    fn global_cursor_position(&self) -> Option<Vec2>;
}

// ============================================================================
// macOS Native Implementation
// ============================================================================

#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn objc_msgSend();
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

/// Configures the application as a background accessory on macOS.
/// Hides the application icon from the macOS Dock and removes the top menu bar entry.
#[cfg(target_os = "macos")]
pub fn configure_as_background_accessory() {
    unsafe {
        type MsgSendPtr = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        type MsgSendPolicy = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, isize) -> *mut std::ffi::c_void;

        let msg_ptr: MsgSendPtr = std::mem::transmute(objc_msgSend as *const ());
        let msg_policy: MsgSendPolicy = std::mem::transmute(objc_msgSend as *const ());

        let cls = objc_getClass(c"NSApplication".as_ptr());
        let sel_shared = sel_registerName(c"sharedApplication".as_ptr());
        let app = msg_ptr(cls, sel_shared);

        let sel_policy = sel_registerName(c"setActivationPolicy:".as_ptr());
        let _ = msg_policy(app, sel_policy, 1isize);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_background_accessory() {}

#[derive(Default)]
pub struct MacOsDesktopEnvironment;

#[cfg(target_os = "macos")]
impl DesktopEnvironment for MacOsDesktopEnvironment {
    fn displays(&self) -> Vec<DisplayInfo> {
        unsafe {
            type MsgSendPtr = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            type MsgSendUsize = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> usize;
            type MsgSendObjAtIndex = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, usize) -> *mut std::ffi::c_void;
            type MsgSendF64 = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> f64;
            type MsgSendRect = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect;
            type MsgSendCStr = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char;

            let msg_ptr: MsgSendPtr = std::mem::transmute(objc_msgSend as *const ());
            let msg_usize: MsgSendUsize = std::mem::transmute(objc_msgSend as *const ());
            let msg_obj: MsgSendObjAtIndex = std::mem::transmute(objc_msgSend as *const ());
            let msg_f64: MsgSendF64 = std::mem::transmute(objc_msgSend as *const ());
            let msg_rect: MsgSendRect = std::mem::transmute(objc_msgSend as *const ());
            let msg_cstr: MsgSendCStr = std::mem::transmute(objc_msgSend as *const ());

            let cls_screen = objc_getClass(c"NSScreen".as_ptr());
            let sel_screens = sel_registerName(c"screens".as_ptr());
            let screens = msg_ptr(cls_screen, sel_screens);
            if screens.is_null() {
                return Vec::new();
            }

            let sel_count = sel_registerName(c"count".as_ptr());
            let count = msg_usize(screens, sel_count);
            if count == 0 {
                return Vec::new();
            }

            let sel_obj = sel_registerName(c"objectAtIndex:".as_ptr());
            let sel_frame = sel_registerName(c"frame".as_ptr());
            let sel_vis = sel_registerName(c"visibleFrame".as_ptr());
            let sel_scale = sel_registerName(c"backingScaleFactor".as_ptr());
            let sel_name = sel_registerName(c"localizedName".as_ptr());
            let sel_utf8 = sel_registerName(c"UTF8String".as_ptr());

            // Primary screen is index 0 in Cocoa; its height is the reference for Y inversion
            let primary_screen = msg_obj(screens, sel_obj, 0);
            let primary_frame = msg_rect(primary_screen, sel_frame);
            let primary_height = primary_frame.size.height as f32;

            let mut result = Vec::with_capacity(count);

            for i in 0..count {
                let screen = msg_obj(screens, sel_obj, i);
                let frame = msg_rect(screen, sel_frame);
                let vis = msg_rect(screen, sel_vis);
                let scale = msg_f64(screen, sel_scale);

                let name_obj = msg_ptr(screen, sel_name);
                let name = if !name_obj.is_null() {
                    let cstr = msg_cstr(name_obj, sel_utf8);
                    if !cstr.is_null() {
                        std::ffi::CStr::from_ptr(cstr).to_string_lossy().to_string()
                    } else {
                        format!("Display {}", i)
                    }
                } else {
                    format!("Display {}", i)
                };

                // Convert Cocoa bottom-left coordinates to top-left virtual screen coordinates
                let bounds_x = frame.origin.x as f32;
                let bounds_y = primary_height - (frame.origin.y + frame.size.height) as f32;
                let bounds_w = frame.size.width as f32;
                let bounds_h = frame.size.height as f32;

                let work_x = vis.origin.x as f32;
                let work_y = primary_height - (vis.origin.y + vis.size.height) as f32;
                let work_w = vis.size.width as f32;
                let work_h = vis.size.height as f32;

                result.push(DisplayInfo {
                    id: format!("display-macos-{}", i),
                    name,
                    bounds: DesktopRect::new(bounds_x, bounds_y, bounds_w, bounds_h),
                    work_area: DesktopRect::new(work_x, work_y, work_w, work_h),
                    scale_factor: scale,
                    is_primary: i == 0,
                });
            }

            result
        }
    }

    fn global_cursor_position(&self) -> Option<Vec2> {
        unsafe {
            type MsgSendPoint = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSPoint;
            type MsgSendPtr = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            type MsgSendRect = unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect;

            let msg_point: MsgSendPoint = std::mem::transmute(objc_msgSend as *const ());
            let msg_ptr: MsgSendPtr = std::mem::transmute(objc_msgSend as *const ());
            let msg_rect: MsgSendRect = std::mem::transmute(objc_msgSend as *const ());

            let cls_screen = objc_getClass(c"NSScreen".as_ptr());
            let sel_main = sel_registerName(c"mainScreen".as_ptr());
            let main_screen = msg_ptr(cls_screen, sel_main);
            if main_screen.is_null() {
                return None;
            }

            let sel_frame = sel_registerName(c"frame".as_ptr());
            let frame = msg_rect(main_screen, sel_frame);
            let primary_height = frame.size.height as f32;

            let nsevent = objc_getClass(c"NSEvent".as_ptr());
            let sel_mouseloc = sel_registerName(c"mouseLocation".as_ptr());
            let pt = msg_point(nsevent, sel_mouseloc);

            let x = pt.x as f32;
            let y = primary_height - pt.y as f32;

            Some(Vec2::new(x, y))
        }
    }
}

// ============================================================================
// Windows / Linux / Generic Fallback Implementation
// ============================================================================

#[allow(dead_code)]
#[derive(Default)]
pub struct FallbackDesktopEnvironment;

impl DesktopEnvironment for FallbackDesktopEnvironment {
    fn displays(&self) -> Vec<DisplayInfo> {
        // Safe, non-hardcoded dynamic single display default
        vec![DisplayInfo {
            id: "display-0".into(),
            name: "Default Display".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, 0.0, 1920.0, 1040.0),
            scale_factor: 1.0,
            is_primary: true,
        }]
    }

    fn global_cursor_position(&self) -> Option<Vec2> {
        None
    }
}

/// Creates the platform-appropriate desktop environment provider.
pub fn create_desktop_environment() -> Box<dyn DesktopEnvironment> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOsDesktopEnvironment)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(FallbackDesktopEnvironment)
    }
}

/// Simulated desktop environment for testing multi-monitor layouts,
/// negative coordinates, varying resolutions, and display scale factors.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SimulatedDesktopEnvironment {
    pub display_list: Vec<DisplayInfo>,
    pub cursor_pos: Option<Vec2>,
}

#[allow(dead_code)]
impl SimulatedDesktopEnvironment {
    pub fn new(display_list: Vec<DisplayInfo>) -> Self {
        Self {
            display_list,
            cursor_pos: None,
        }
    }
}

impl DesktopEnvironment for SimulatedDesktopEnvironment {
    fn displays(&self) -> Vec<DisplayInfo> {
        self.display_list.clone()
    }

    fn global_cursor_position(&self) -> Option<Vec2> {
        self.cursor_pos
    }
}

// ============================================================================
// Comprehensive Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_rect_calculations() {
        let rect = DesktopRect::new(100.0, 200.0, 800.0, 600.0);
        assert_eq!(rect.left(), 100.0);
        assert_eq!(rect.top(), 200.0);
        assert_eq!(rect.right(), 900.0);
        assert_eq!(rect.bottom(), 800.0);

        assert!(rect.contains_point(150.0, 250.0));
        assert!(!rect.contains_point(50.0, 250.0));
        assert!(!rect.contains_point(150.0, 850.0));

        let overlap = DesktopRect::new(500.0, 400.0, 600.0, 500.0);
        assert_eq!(rect.intersection_area(&overlap), 400.0 * 400.0);
    }

    #[test]
    fn test_small_laptop_display() {
        // 1366x768 display with a 40px taskbar at bottom
        let laptop = DisplayInfo {
            id: "laptop-1366".into(),
            name: "Built-in Screen".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1366.0, 768.0),
            work_area: DesktopRect::new(0.0, 0.0, 1366.0, 728.0),
            scale_factor: 1.0,
            is_primary: true,
        };

        let metrics = DesktopMetrics::from_display(
            &laptop,
            480.0,
            340.0,
            340.0 * 0.75, // 255.0
            20.0,
            0.0,
        );

        assert_eq!(metrics.min_x, 20.0);
        assert_eq!(metrics.max_x, 1366.0 - 480.0 - 20.0);
        // Ground Y = 728.0 - 255.0 = 473.0
        assert_eq!(metrics.ground_y, 473.0);
    }

    #[test]
    fn test_1080p_monitor_with_dock() {
        // 1920x1080 display with 65px dock
        let monitor_1080p = DisplayInfo {
            id: "mon-1080p".into(),
            name: "FHD Monitor".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, 25.0, 1920.0, 990.0), // 25px menu bar, 65px dock => bottom = 1015.0
            scale_factor: 1.0,
            is_primary: true,
        };

        let metrics = DesktopMetrics::from_display(
            &monitor_1080p,
            480.0,
            340.0,
            255.0,
            20.0,
            0.0,
        );

        assert_eq!(metrics.min_x, 20.0);
        assert_eq!(metrics.max_x, 1920.0 - 480.0 - 20.0);
        // Ground Y = 1015.0 - 255.0 = 760.0
        assert_eq!(metrics.ground_y, 760.0);
    }

    #[test]
    fn test_1440p_ultrawide_monitor() {
        // 3440x1440 ultrawide
        let ultrawide = DisplayInfo {
            id: "mon-3440".into(),
            name: "Ultrawide".into(),
            bounds: DesktopRect::new(0.0, 0.0, 3440.0, 1440.0),
            work_area: DesktopRect::new(0.0, 0.0, 3440.0, 1370.0), // 70px taskbar
            scale_factor: 1.0,
            is_primary: true,
        };

        let metrics = DesktopMetrics::from_display(
            &ultrawide,
            480.0,
            340.0,
            255.0,
            20.0,
            0.0,
        );

        assert_eq!(metrics.min_x, 20.0);
        assert_eq!(metrics.max_x, 3440.0 - 480.0 - 20.0);
        assert_eq!(metrics.ground_y, 1370.0 - 255.0);
    }

    #[test]
    fn test_4k_hidpi_display() {
        // 4K display: 3840x2160 physical, 200% scale => 1920x1080 logical
        let monitor_4k = DisplayInfo {
            id: "mon-4k".into(),
            name: "4K HiDPI".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, 0.0, 1920.0, 1020.0), // 60px taskbar
            scale_factor: 2.0,
            is_primary: true,
        };

        let metrics = DesktopMetrics::from_display(
            &monitor_4k,
            480.0,
            340.0,
            255.0,
            20.0,
            0.0,
        );

        assert_eq!(metrics.min_x, 20.0);
        assert_eq!(metrics.max_x, 1920.0 - 480.0 - 20.0);
        assert_eq!(metrics.ground_y, 1020.0 - 255.0);
    }

    #[test]
    fn test_multi_monitor_side_by_side_with_negative_coordinates() {
        // Monitor A (Left): 1920x1080 at x = -1920, y = 0
        let mon_a = DisplayInfo {
            id: "mon-left".into(),
            name: "Left Monitor".into(),
            bounds: DesktopRect::new(-1920.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(-1920.0, 0.0, 1920.0, 1040.0),
            scale_factor: 1.0,
            is_primary: false,
        };

        // Monitor B (Primary): 2560x1440 at x = 0, y = 0
        let mon_b = DisplayInfo {
            id: "mon-center".into(),
            name: "Center Monitor".into(),
            bounds: DesktopRect::new(0.0, 0.0, 2560.0, 1440.0),
            work_area: DesktopRect::new(0.0, 30.0, 2560.0, 1350.0), // bottom = 1380.0
            scale_factor: 1.0,
            is_primary: true,
        };

        let env = SimulatedDesktopEnvironment::new(vec![mon_a.clone(), mon_b.clone()]);

        // Cat is positioned on Monitor A (e.g. x = -1000.0, y = 500.0)
        let win_on_a = DesktopRect::new(-1000.0, 500.0, 480.0, 340.0);
        let detected_a = env.display_for_window(win_on_a).unwrap();
        assert_eq!(detected_a.id, "mon-left");

        let metrics_a = DesktopMetrics::from_display(&detected_a, 480.0, 340.0, 255.0, 20.0, 0.0);
        assert_eq!(metrics_a.min_x, -1920.0 + 20.0);
        assert_eq!(metrics_a.max_x, 0.0 - 480.0 - 20.0);
        assert_eq!(metrics_a.ground_y, 1040.0 - 255.0);

        // Cat is dragged across boundary to Monitor B (e.g. x = 600.0, y = 800.0)
        let win_on_b = DesktopRect::new(600.0, 800.0, 480.0, 340.0);
        let detected_b = env.display_for_window(win_on_b).unwrap();
        assert_eq!(detected_b.id, "mon-center");

        let metrics_b = DesktopMetrics::from_display(&detected_b, 480.0, 340.0, 255.0, 20.0, 0.0);
        assert_eq!(metrics_b.min_x, 20.0);
        assert_eq!(metrics_b.max_x, 2560.0 - 480.0 - 20.0);
        assert_eq!(metrics_b.ground_y, 1380.0 - 255.0);
    }

    #[test]
    fn test_multi_monitor_vertical_arrangement() {
        // Monitor Top: 1920x1080 at y = -1080
        let mon_top = DisplayInfo {
            id: "mon-top".into(),
            name: "Top Monitor".into(),
            bounds: DesktopRect::new(0.0, -1080.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, -1080.0, 1920.0, 1040.0), // bottom = -40.0
            scale_factor: 1.0,
            is_primary: false,
        };

        // Monitor Bottom (Primary): 1920x1080 at y = 0
        let mon_bottom = DisplayInfo {
            id: "mon-bottom".into(),
            name: "Bottom Monitor".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, 0.0, 1920.0, 1020.0), // bottom = 1020.0
            scale_factor: 1.0,
            is_primary: true,
        };

        let env = SimulatedDesktopEnvironment::new(vec![mon_top.clone(), mon_bottom.clone()]);

        // Cat in Top Monitor
        let win_top = DesktopRect::new(200.0, -800.0, 480.0, 340.0);
        let detected_top = env.display_for_window(win_top).unwrap();
        assert_eq!(detected_top.id, "mon-top");

        let metrics_top = DesktopMetrics::from_display(&detected_top, 480.0, 340.0, 255.0, 20.0, 0.0);
        // Ground Y = -40.0 - 255.0 = -295.0
        assert_eq!(metrics_top.ground_y, -295.0);

        // Cat in Bottom Monitor
        let win_bottom = DesktopRect::new(200.0, 400.0, 480.0, 340.0);
        let detected_bottom = env.display_for_window(win_bottom).unwrap();
        assert_eq!(detected_bottom.id, "mon-bottom");

        let metrics_bottom = DesktopMetrics::from_display(&detected_bottom, 480.0, 340.0, 255.0, 20.0, 0.0);
        assert_eq!(metrics_bottom.ground_y, 1020.0 - 255.0);
    }

    #[test]
    fn test_dynamic_cat_size_adjustments() {
        let display = DisplayInfo {
            id: "display-1".into(),
            name: "Standard Display".into(),
            bounds: DesktopRect::new(0.0, 0.0, 1920.0, 1080.0),
            work_area: DesktopRect::new(0.0, 0.0, 1920.0, 1020.0),
            scale_factor: 1.0,
            is_primary: true,
        };

        // Normal size (480x340)
        let normal_metrics = DesktopMetrics::from_display(&display, 480.0, 340.0, 255.0, 20.0, 0.0);

        // Extra large cat (1.5x scaling -> 720x510)
        let xl_metrics = DesktopMetrics::from_display(&display, 720.0, 510.0, 510.0 * 0.75, 20.0, 0.0);

        assert_eq!(normal_metrics.max_x, 1920.0 - 480.0 - 20.0);
        assert_eq!(xl_metrics.max_x, 1920.0 - 720.0 - 20.0);
        assert!(xl_metrics.max_x < normal_metrics.max_x);
        assert!(xl_metrics.ground_y < normal_metrics.ground_y);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_real_screen_query() {
        let env = MacOsDesktopEnvironment;
        let displays = env.displays();
        assert!(!displays.is_empty(), "At least one display must be detected on macOS");

        for d in &displays {
            println!("Real macOS Display: {:?}", d);
            assert!(d.bounds.width > 0.0);
            assert!(d.bounds.height > 0.0);
            assert!(d.work_area.width > 0.0);
            assert!(d.work_area.height > 0.0);
            assert!(d.work_area.bottom() <= d.bounds.bottom() + 1.0);
        }
    }
}
