//! The main menu: the background, the two halves of the logo, the
//! buttons and the version text, built in draw order. Display Options
//! opens the same Options pages the game's Esc menu shows, on the Video
//! page.

use crate::ui::front_end::control::{vk, Action, Control, ControlKind};
use crate::ui::front_end::flow::Trigger;
use crate::ui::front_end::screen::{FrontCtx, Screen};
use crate::ui::front_end::{Registry, MAIN_MENU};

/// Background descriptor 8's cel (global, loaded by the preload):
/// `gameselectscreenEXP` when the expansion is installed, else `TitleScreen`.
pub const BACKGROUND: &str = r"FrontEnd\gameselectscreenEXP";
pub const BACKGROUND_CLASSIC: &str = r"FrontEnd\TitleScreen";
/// Button cels (all exist in the install).
pub const WIDE: &str = r"FrontEnd\WideButtonBlank";
pub const SHORT: &str = r"CharSelect\ShortButtonBlank";
/// Logo halves: the black base cel (the fire overlay is drawn
/// additively over it by the host).
pub const LOGO_LEFT: &str = r"FrontEnd\D2logoBlackLeft";
pub const LOGO_RIGHT: &str = r"FrontEnd\D2logoBlackRight";
/// The version line's box: frame x, bottom edge, width and height.
pub const VERSION_X: i32 = 0;
pub const VERSION_Y: i32 = 599;
pub const VERSION_W: u16 = 200;
pub const VERSION_H: u16 = 40;
/// Font16 (descriptor 0x115, record).
pub const VERSION_FONT: u16 = 1;

/// The main menu's top version line: the D2RUST build version.
pub fn version_text() -> String {
    format!("D2RUST {}", crate::launch::VERSION)
}

/// Button string ids.
pub const STR_SINGLE_PLAYER: u32 = 5106;
pub const STR_EXIT: u32 = 5109;
pub const STR_CREDITS: u32 = 5110;
/// The Display Options button: D2RUST's own, a wide button like Single
/// Player and Exit Game. Its label is written in capitals as the other
/// buttons' labels are.
pub const OPTIONS_LABEL: &str = "DISPLAY OPTIONS";
pub const OPTIONS_Y: i32 = 486;
pub const OPTIONS_H: u16 = 35;

struct MainMenu;

fn button(
    art: &'static str,
    x: i32,
    y: i32,
    w: u16,
    h: u16,
    string: u32,
    action: Action,
) -> Control {
    Control::new(ControlKind::Button, x, y, w, h)
        .with_art(art)
        .with_string(string)
        .with_action(action)
}

/// Frame to draw for a resting, pressed or disabled button (r3 art): the same
/// rule the host applies.
pub fn button_frame(c: &Control, tile: u32, pressed: bool) -> u32 {
    let tiles = crate::ui::front_end::control::button_tiles(c.w, c.h);
    crate::ui::front_end::control::button_frame(tile, tiles, pressed, c.enabled, false)
}

impl Screen for MainMenu {
    fn build(&mut self, ctx: &mut FrontCtx) -> Vec<Control> {
        let up = if ctx.expansion { 0 } else { 100 };
        let bg = if ctx.expansion {
            BACKGROUND
        } else {
            BACKGROUND_CLASSIC
        };
        vec![
            Control::new(ControlKind::Image, 0, 599, 800, 600).with_art(bg),
            Control::new(ControlKind::AnimImage, 400, 120, 181, 170).with_art(LOGO_LEFT),
            Control::new(ControlKind::AnimImage, 400, 120, 188, 177).with_art(LOGO_RIGHT),
            button(
                WIDE,
                264,
                324 - up,
                272,
                35,
                STR_SINGLE_PLAYER,
                Action::Trigger(Trigger::SinglePlayer),
            ),
            button(
                SHORT,
                332,
                528,
                135,
                25,
                STR_CREDITS,
                Action::Trigger(Trigger::Credits),
            ),
            button(
                WIDE,
                264,
                568,
                272,
                35,
                STR_EXIT,
                Action::Trigger(Trigger::Exit),
            )
            .with_hotkey(vk::ESC),
            button(
                WIDE,
                264,
                OPTIONS_Y,
                272,
                OPTIONS_H,
                0,
                Action::Trigger(Trigger::Options),
            )
            .with_text(OPTIONS_LABEL, 0),
            Control::new(ControlKind::Text, VERSION_X, VERSION_Y, VERSION_W, VERSION_H)
                .with_font(VERSION_FONT, 2)
                .with_text(version_text(), 0),
        ]
    }
}

pub fn register(reg: &mut Registry) {
    reg.register(MAIN_MENU, Box::new(MainMenu));
}
