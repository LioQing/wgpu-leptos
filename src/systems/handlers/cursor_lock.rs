use std::sync::Arc;

use winit::{
    error::ExternalError,
    event::{MouseButton, WindowEvent},
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};
use winit_input_helper::WinitInputHelper;

use crate::systems::Error;

/// Handler for cursor locking.
pub struct CursorLock {
    window: Arc<Window>,
    should_lock_cursor: bool,
    is_cursor_locked: bool,
}

impl CursorLock {
    pub fn new(window: Arc<Window>, should_lock_cursor: bool) -> Self {
        Self {
            window,
            should_lock_cursor,
            is_cursor_locked: false,
        }
    }

    pub fn should_lock_cursor(&self) -> bool {
        self.should_lock_cursor
    }

    pub fn set_should_lock_cursor(&mut self, should_lock_cursor: bool) -> Result<(), Error> {
        self.should_lock_cursor = should_lock_cursor;

        // Unlock cursor if should lock cursor is false and cursor is locked
        if !should_lock_cursor && self.is_cursor_locked {
            self.set_cursor_locked(false)?;
        }

        Ok(())
    }

    pub fn is_cursor_locked(&self) -> bool {
        self.is_cursor_locked
    }

    /// Lock or unlock the cursor.
    fn set_cursor_locked(&mut self, locked: bool) -> Result<(), Error> {
        if locked {
            match self.window.set_cursor_grab(CursorGrabMode::Locked) {
                Err(ExternalError::NotSupported(_)) => {
                    self.window.set_cursor_grab(CursorGrabMode::Confined)?;
                }
                Ok(_) => {}
                Err(e) => return Err(Error::DisplayLockCursor(e)),
            }
        } else {
            match self.window.set_cursor_grab(CursorGrabMode::None) {
                Ok(_) => {}
                Err(e) => return Err(Error::DisplayLockCursor(e)),
            }
        }

        self.window.set_cursor_visible(!locked);

        self.is_cursor_locked = locked;
        Ok(())
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::Focused(false) = event {
            match self.set_cursor_locked(false) {
                Ok(_) => {}
                Err(e) => log::warn!("Unable to unlock cursor on window unfocused: {e:?}"),
            }
        }
    }

    pub fn update(&mut self, input: &mut WinitInputHelper) {
        // Focus window
        if !self.is_cursor_locked()
            && self.should_lock_cursor()
            && input.mouse_pressed(MouseButton::Left)
            && input.cursor().is_some()
        {
            match self.set_cursor_locked(true) {
                Ok(_) => {}
                Err(e) => log::warn!("Unable to lock cursor on cursor grabbed: {e:?}"),
            }
        }

        // Manually unlock cursor
        if self.is_cursor_locked()
            && [KeyCode::Escape, KeyCode::Tab]
                .into_iter()
                .any(|k| input.key_pressed(k))
        {
            match self.set_cursor_locked(false) {
                Ok(_) => {}
                Err(e) => log::warn!("Unable to unlock cursor on escape pressed: {e:?}"),
            }
        }
    }
}

/// Builder of [`CursorLock`].
pub struct CursorLockBuilder<T> {
    window: T,
    should_lock_cursor: bool,
    is_cursor_locked: bool,
}

pub mod builder {
    use super::*;

    pub struct NoWindow;
    pub struct WithWindow(pub Arc<Window>);
}

impl CursorLockBuilder<builder::NoWindow> {
    pub fn new() -> Self {
        Self {
            window: builder::NoWindow,
            should_lock_cursor: false,
            is_cursor_locked: false,
        }
    }
}

impl<T> CursorLockBuilder<T> {
    pub fn with_window(self, window: Arc<Window>) -> CursorLockBuilder<builder::WithWindow> {
        CursorLockBuilder {
            window: builder::WithWindow(window),
            should_lock_cursor: self.should_lock_cursor,
            is_cursor_locked: self.is_cursor_locked,
        }
    }

    pub fn with_should_lock_cursor(mut self, should_lock_cursor: bool) -> Self {
        self.should_lock_cursor = should_lock_cursor;
        self
    }
}

impl CursorLockBuilder<builder::WithWindow> {
    pub fn build(self) -> CursorLock {
        CursorLock::new(self.window.0, self.should_lock_cursor)
    }
}
