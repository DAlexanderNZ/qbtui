use crate::{App, InputMode};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

fn clamp_cursor(new_cursor_pos: usize, input: &String) -> usize {
    new_cursor_pos.clamp(0, input.chars().count())
}

/// Stores the currently selected field being edited.
#[derive(Default, Debug, Copy, Clone)]
pub enum CurentInput {
    #[default]
    ApiUrl,
    Username,
    Password
}

impl CurentInput {
    // Return the number of fields available
    fn count() -> usize {
        3
    }

    // Convert the enum into its corresponding index.
    fn to_index(self) -> usize {
        match self {
            CurentInput::ApiUrl => 0,
            CurentInput::Username => 1,
            CurentInput::Password => 2,
        }
    }

    // Convert an index back into the enum.
    fn from_index(i: usize) -> Self {
        match i {
            0 => CurentInput::ApiUrl,
            1 => CurentInput::Username,
            2 => CurentInput::Password,
            _ => panic!("Index out of range"),
        }
    }

    /// Shift the value of the CurrentInput enum by a float value.
    /// Raps around the value if it exceeds the number of fields.
    fn shift(&mut self, delta: isize) {
        let count = CurentInput::count() as isize;
        let current_index = self.to_index() as isize;
        // Add delta and wrap around using modulo arithmetic
        let new_index = (current_index + delta).rem_euclid(count) as usize;
        *self = Self::from_index(new_index);
    }
}

/// Represents the currently selected tab for torrent information display.
#[derive(Default, Debug, Clone, Copy)]
pub enum SelectedInfoTab {
    #[default]
    Details,
    Files,
    Trackers,
    Peers,
}

impl SelectedInfoTab {
    fn to_index(self) -> usize {
        match self {
            SelectedInfoTab::Details => 0,
            SelectedInfoTab::Files => 1,
            SelectedInfoTab::Trackers => 2,
            SelectedInfoTab::Peers => 3,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => SelectedInfoTab::Details,
            1 => SelectedInfoTab::Files,
            2 => SelectedInfoTab::Trackers,
            3 => SelectedInfoTab::Peers,
            _ => panic!("Index out of range"),
        }
    }

    fn next(&mut self) {
        let current_index = self.to_index();
        let new_index = (current_index + 1) % 4; // Wrap around last tab
        *self = Self::from_index(new_index);
    }

    fn previous(&mut self) {
        let current_index = self.to_index();
        let new_index = if current_index == 0 {
            3 // Wrap around to the last tab
        } else {
            (current_index - 1) % 4
        };
        *self = Self::from_index(new_index);
    }
}

impl App {
    /// Reads the crossterm events and updates the state of [`App`].
    pub async fn handle_crossterm_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                match event {
                    Some(Ok(evt)) => {
                        match evt {
                            Event::Key(key)
                                if key.kind == KeyEventKind::Press
                                    => self.on_key_event(key),
                            Event::Mouse(_) => {}
                            Event::Resize(_, _) => {}
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Sleep for a short duration to avoid busy waiting.
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // Global keys
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            _ => {}
        }

        // Mode specific keys
        match self.input_mode {
            InputMode::Normal => {
                match (key.modifiers, key.code) {
                    (_, KeyCode::Esc) => self.quit(),
                    (_, KeyCode::Char('r')) => {
                        self.refresh_torrents = true;
                    },
                    // Open/Close edit config popup
                    (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                        self.cfg_popup = !self.cfg_popup;
                        self.input_mode = InputMode::Config;
                        self.reset_cursor();        
                    },
                    (_, KeyCode::Tab) => self.torrent_popup = !self.torrent_popup,
                    // Moving about the table
                    (_, KeyCode::Char('j') | KeyCode::Down) => self.next_row(),
                    (_, KeyCode::Char('k') | KeyCode::Up) => self.previous_row(),
                    (_, KeyCode::Char('h') | KeyCode::Left) => self.previous_column(),
                    (_, KeyCode::Char('l') | KeyCode::Right) => self.next_column(),
                    // Delete input char
                    (_, KeyCode::Backspace) => self.delete_char(),            
                    _ => {}
                }
            },
            InputMode::Config => {
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('e')) 
                    | (_, KeyCode::Esc) => {
                        self.cfg_popup = !self.cfg_popup;
                        self.input_mode = InputMode::Normal;
                        self.input = self.cfg.clone();
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        self.save_cfg = true;
                        self.refresh_torrents = true;
                        self.input_mode = InputMode::Normal;
                    },
                    (_, KeyCode::Char(to_insert)) => self.enter_char(to_insert),
                    (_, KeyCode::Backspace) => self.delete_char(),
                    (_, KeyCode::Down | KeyCode::Enter) => self.next_row(),
                    (_, KeyCode::Up) => self.previous_row(),
                    (_, KeyCode::Left) => self.previous_column(),
                    (_, KeyCode::Right) => self.next_column(),
                    _ => {}   
                }
            },
        }
    }

    /// Move the selection down in the InputMode context.
    /// In Normal mode, it moves down the torrent table.
    /// In Config mode, it moves down the config inputs.
    fn next_row(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                let i =  match self.state.selected() {
                    Some(i) => {
                        if i >= self.torrents.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
                self.scroll_state = self.scroll_state.position(i);
            },
            InputMode::Config => {
                self.current_input.shift(1);
                let input = self.current_input();
                self.charcter_index = clamp_cursor(input.len(), input);
            }
        }
    }

    /// Move the selection up in the InputMode context.
    /// In Normal mode, it moves up the torrent table.
    /// In Config mode, it moves up the config inputs.
    fn previous_row(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                let i = match self.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.torrents.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
                self.scroll_state = self.scroll_state.position(i);
            },
            InputMode::Config => {
                self.current_input.shift(-1);
                let input = self.current_input();
                self.charcter_index = clamp_cursor(input.len(), input);
            }
        }
    }

    /// Move the selection right in the InputMode context.
    /// In Normal mode, it moves right the torrent table.
    /// In Config mode, it moves right the config inputs.
    fn next_column(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                self.info_tab.next();
            },
            InputMode::Config => { 
                let input = self.current_input();
                let cursor_moved_right = self.charcter_index.saturating_add(1);
                self.charcter_index = clamp_cursor(cursor_moved_right, input); 
            }
        }
    }

    /// Move the selection left in the InputMode context.
    /// In Normal mode, it moves left the torrent table.
    /// In Config mode, it moves left the config inputs.
    fn previous_column(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                if self.torrent_popup == true {
                    self.info_tab.previous();
                }
            },
            InputMode::Config => { 
                let input = self.current_input();
                let cursor_moved_left = self.charcter_index.saturating_sub(1);
                self.charcter_index = clamp_cursor(cursor_moved_left, input);
            }
        }
    }

    /// Returns a static reference to the currently selected input field.
    fn current_input(&self) -> &String {
        match self.current_input {
            CurentInput::ApiUrl => &self.input.api_url,
            CurentInput::Username => &self.input.username,
            CurentInput::Password => &self.input.password
        }
    }

    /// Returns a mutable reference to the currently selected input field.
    fn current_input_mut(&mut self) -> &mut String {
        match self.current_input {
            CurentInput::ApiUrl => &mut self.input.api_url,
            CurentInput::Username => &mut self.input.username,
            CurentInput::Password => &mut self.input.password
        }
    }

    /// Inserts a char at the current cursor position in the current input field.
    fn enter_char(&mut self, c: char) {
        let index = self.byte_index();
        let input = self.current_input_mut();
        input.insert(index, c);
        self.next_column();
    }


    fn byte_index(&self) -> usize {
        let input = self.current_input();
        input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.charcter_index)
            .unwrap_or(input.len())
    }

    /// Removes char at the current cursor position from the current input field.
    fn delete_char(&mut self)  {
        if self.charcter_index != 0 {
            let current_index = self.charcter_index;
            let input = self.current_input_mut();
            let before_chars = input.chars().take(current_index - 1);
            let after_chars = input.chars().skip(current_index);
            *input = before_chars.chain(after_chars).collect();
            self.charcter_index = clamp_cursor(current_index - 1, input);
        }
    }

    /// Resets the charcter index (cursor) to the end of the current input field.
    pub fn reset_cursor(&mut self) {
        self.charcter_index = self.current_input().chars().count();
    }
}