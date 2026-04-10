//! TV Input Manager for Cosmic Climber
//!
//! Handles D-pad/remote input from the TV Platform Abstraction Layer (PAL).
//! Maps TV actions (Up/Down/Left/Right/Action/Back) to game controls.

#[cfg(target_arch = "wasm32")]
use std::cell::UnsafeCell;

/// TV action types corresponding to PAL key mappings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvAction {
    Up,
    Down,
    Left,
    Right,
    Action, // OK/Select - Jump/Shoot/Confirm
    Back,   // Return/Back - Pause
}

/// Tracks the pressed state of TV actions
#[derive(Debug, Default)]
pub struct TvInputManager {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub action: bool, // Jump/Shoot/Confirm
    pub back: bool,   // Pause
    up_prev: bool,
    down_prev: bool,
    left_prev: bool,
    right_prev: bool,
    action_prev: bool,
    back_prev: bool,
}

impl TvInputManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an action's pressed state (called from WASM JS bridge)
    pub fn set_action(&mut self, action: TvAction, pressed: bool) {
        match action {
            TvAction::Up => self.up = pressed,
            TvAction::Down => self.down = pressed,
            TvAction::Left => self.left = pressed,
            TvAction::Right => self.right = pressed,
            TvAction::Action => self.action = pressed,
            TvAction::Back => self.back = pressed,
        }
    }

    /// Held Action (OK) — mirrors `is_key_down` where needed.
    pub fn is_action_held(&self) -> bool {
        self.action
    }

    pub fn action_just_pressed(&self) -> bool {
        self.action && !self.action_prev
    }

    pub fn back_just_pressed(&self) -> bool {
        self.back && !self.back_prev
    }

    pub fn up_just_pressed(&self) -> bool {
        self.up && !self.up_prev
    }

    pub fn down_just_pressed(&self) -> bool {
        self.down && !self.down_prev
    }

    pub fn left_just_pressed(&self) -> bool {
        self.left && !self.left_prev
    }

    pub fn right_just_pressed(&self) -> bool {
        self.right && !self.right_prev
    }

    /// Call once at end of each `Game::update` after all input is read.
    pub fn sync_prev_from_current(&mut self) {
        self.up_prev = self.up;
        self.down_prev = self.down;
        self.left_prev = self.left;
        self.right_prev = self.right;
        self.action_prev = self.action;
        self.back_prev = self.back;
    }
}

// ============================================================================
// WASM Exports - these functions are called from JavaScript/PAL
// ============================================================================

#[cfg(target_arch = "wasm32")]
struct TvInputGlobal(UnsafeCell<Option<TvInputManager>>);

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TvInputGlobal {}

#[cfg(target_arch = "wasm32")]
static TV_INPUT_GLOBAL: TvInputGlobal = TvInputGlobal(UnsafeCell::new(None));

/// Initialize the TV input manager (call once at startup on WASM)
#[cfg(target_arch = "wasm32")]
pub fn init_tv_input_manager() {
    unsafe {
        *TV_INPUT_GLOBAL.0.get() = Some(TvInputManager::new());
    }
}

/// Get immutable reference to TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager() -> Option<&'static TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_ref() }
}

/// Get mutable reference to TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager_mut() -> Option<&'static mut TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_mut() }
}

// WASM exported functions - these are called from JS via PAL
#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_up"]
pub extern "C" fn mq_handle_up(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Up, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_down"]
pub extern "C" fn mq_handle_down(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Down, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_left"]
pub extern "C" fn mq_handle_left(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Left, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_right"]
pub extern "C" fn mq_handle_right(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Right, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_action"]
pub extern "C" fn mq_handle_action(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Action, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_back"]
pub extern "C" fn mq_handle_back(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Back, pressed != 0);
        }
    }
}

// Non-WASM stubs
#[cfg(not(target_arch = "wasm32"))]
pub fn init_tv_input_manager() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_tv_input_manager() -> Option<&'static TvInputManager> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_tv_input_manager_mut() -> Option<&'static mut TvInputManager> {
    None
}
