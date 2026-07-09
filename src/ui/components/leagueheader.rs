use alloc::ffi::CString;
use pebble::layer::AsLayer;
use pebble::types::{GBitmap, GColor, GCompOp, GCornerMask, GPoint, GRect, GSize, GTextAlignment, GTextOverflowMode};
use pebble::system::fonts::{GFont, FontKey};
use pebble::platform::is_rect;
use pebble::{GContext, RawLayer};
use taconite::state::Snap;
use taconite::{State};
use taconite::layer::Draw;


const STATUS_BAR_LAYER_HEIGHT: i16 = 16;
const HEADER_HEIGHT: i16 = 28 + STATUS_BAR_LAYER_HEIGHT;

pub struct HeaderData {
    pub title: Snap<CString>,
    pub icon: Option<Snap<GBitmap>>,
    pub under_status_bar: bool,
}

pub struct HeaderLayer {
    internal: Draw,
}

impl AsLayer for HeaderLayer {
    fn as_raw(&self) -> *mut RawLayer {
        self.internal.as_raw()
    }
}

impl HeaderLayer {
    pub fn new(window_bounds: GRect, state: impl Into<State<HeaderData>>) -> Self {
        let state = state.into();
        let header_bounds = GRect {
            origin: window_bounds.origin,
            size: GSize { w: window_bounds.size.w, h: HEADER_HEIGHT },
        };

        let dl = Draw::new(header_bounds, state, move |ctx, data, frame| {
            let bounds = GRect { origin: GPoint { x: 0, y: 0 }, size: frame.size };
            // pbl_log!("drawing league header, name = %s", data.title.clone());
            if is_rect() {
                draw_rect(ctx, data, bounds);
            } else {
                draw_circle(ctx, data, bounds);
            }
        });

        HeaderLayer { internal: dl }
    }

    pub fn render(&self) {
        self.internal.render();
    }
}

fn draw_rect(ctx: &mut GContext, data: &HeaderData, bounds: GRect) {
    ctx.set_fill_color(GColor::DukeBlue);
    ctx.fill_rect(bounds, 0, GCornerMask::GCornerNone);

    let font = GFont::get_system(FontKey::GOTHIC_18_BOLD);
    let title_size = ctx.measure_text(&data.title, &font, bounds.size, None);
    let icon_padding = if data.icon.is_some() { 32 } else { 8 };
    let title_bounds = GRect {
        origin: GPoint { x: icon_padding, y: STATUS_BAR_LAYER_HEIGHT },
        size: GSize { w: title_size.w, h: 18 },
    };
    ctx.draw_text(&data.title, &font, title_bounds, GTextOverflowMode::TrailingEllipsis, GTextAlignment::Center);

    if let Some(icon) = &data.icon {
        let icon_bounds = GRect {
            origin: GPoint { x: 8, y: 4 + STATUS_BAR_LAYER_HEIGHT },
            size: GSize { w: 16, h: 16 },
        };
        ctx.set_compositing_mode(GCompOp::GCompOpSet);
        ctx.draw_bitmap_in_rect(&icon, icon_bounds);
    }
}

fn draw_circle(ctx: &mut GContext, data: &HeaderData, bounds: GRect) {
    let menu_bottom = if data.under_status_bar { 28 + STATUS_BAR_LAYER_HEIGHT } else { 32 };

    // Mask any overflowing menu items behind the header
    let mask_bounds = GRect {
        origin: GPoint { x: 0, y: 0 },
        size: GSize { w: bounds.size.w, h: menu_bottom },
    };
    ctx.set_fill_color(GColor::White);
    ctx.fill_rect(mask_bounds, 0, GCornerMask::GCornerNone);

    ctx.set_fill_color(GColor::DukeBlue);
    let radius = bounds.size.w;
    let horz_center = bounds.size.w / 2;
    // vert_center can be negative (circle extends above the layer); cast through u16 preserves the two's-complement bit pattern expected by the C SDK's int16_t GPoint fields
    let vert_center = menu_bottom - radius;
    ctx.fill_circle(GPoint { x: horz_center, y: vert_center }, radius as u16);

    let font = GFont::get_system(FontKey::GOTHIC_18_BOLD);
    let title_size = ctx.measure_text(&data.title, &font, bounds.size, None);
    let icon_padding = if data.icon.is_some() { 8 } else { 0 };
    let title_x = (bounds.size.w / 2).wrapping_sub(title_size.w / 2).wrapping_add(icon_padding);
    let title_y = if data.under_status_bar { 2 + STATUS_BAR_LAYER_HEIGHT } else { 4 };
    let title_bounds = GRect {
        origin: GPoint { x: title_x, y: title_y },
        size: GSize { w: title_size.w, h: 18 },
    };
    ctx.draw_text(&data.title, &font, title_bounds, GTextOverflowMode::TrailingEllipsis, GTextAlignment::Center);

    if let Some(icon) = &data.icon {
        let icon_x = (bounds.size.w / 2).wrapping_sub(title_size.w / 2).wrapping_sub(12);
        let icon_y = if data.under_status_bar { 6 + STATUS_BAR_LAYER_HEIGHT } else { 8 };
        let icon_bounds = GRect {
            origin: GPoint { x: icon_x, y: icon_y },
            size: GSize { w: 16, h: 16 },
        };
        ctx.set_compositing_mode(GCompOp::GCompOpSet);
        ctx.draw_bitmap_in_rect(&icon, icon_bounds);
    }
}
