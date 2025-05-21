use crate::{App, InputMode, CurentInput};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

fn clamp_cursor(new_cursor_pos: usize, input: &String) -> usize {
    new_cursor_pos.clamp(0, input.chars().count())
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
        match self.input_mode {
            InputMode::Normal => {
                match (key.modifiers, key.code) {
                    (_, KeyCode::Esc | KeyCode::Char('q'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                    (_, KeyCode::Char('r')) => {
                        // TODO: Refresh the torrents list.
                    },
                    // Open/Close edit config popup
                    (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                        self.cfg_popup = !self.cfg_popup;
                        self.input_mode = InputMode::Config;
                    },
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
                    (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                    (KeyModifiers::CONTROL, KeyCode::Char('e')) 
                    | (_, KeyCode::Esc) => {
                        self.cfg_popup = !self.cfg_popup;
                        self.input_mode = InputMode::Normal;
                        self.reset_cursor();
                        self.input = self.cfg.clone();
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        self.save_cfg = true;
                        self.input_mode = InputMode::Normal;
                        self.reset_cursor();
                    },
                    (_, KeyCode::Char(to_insert)) => self.enter_char(to_insert),
                    (_, KeyCode::Backspace) => self.delete_char(),
                    (_, KeyCode::Down) => self.next_row(),
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
                //self.scroll_state.position(i * TABLE_ITEM_HEIGHT);
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
                //self.scroll_state.position(i * TABLE_ITEM_HEIGHT);
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
            InputMode::Normal => self.state.select_next_column(),
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
            InputMode::Normal => self.state.select_previous_column(),
            InputMode::Config => { 
                let input = self.current_input();
                let cursor_moved_left = self.charcter_index.saturating_sub(1);
                self.charcter_index = clamp_cursor(cursor_moved_left, input);
            }
        }
    }

    fn current_input(&self) -> &String {
        match self.current_input {
            CurentInput::ApiUrl => &self.input.api_url,
            CurentInput::Username => &self.input.username,
            CurentInput::Password => &self.input.password
        }
    }

    fn current_input_mut(&mut self) -> &mut String {
        match self.current_input {
            CurentInput::ApiUrl => &mut self.input.api_url,
            CurentInput::Username => &mut self.input.username,
            CurentInput::Password => &mut self.input.password
        }
    }

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

    /// Removes char at the current cursor position from self.input
    fn delete_char(&mut self)  {
        if self.charcter_index != 0 {
            let current_index = self.charcter_index;
            let input = self.current_input_mut();
            let before_chars = input.chars().take(current_index - 1);
            let after_chars = input.chars().skip(current_index);
            *input = before_chars.chain(after_chars).collect();
            self.charcter_index = clamp_cursor(current_index, input);
        }
    }

    const fn reset_cursor(&mut self) {
        self.charcter_index = 0;
    }
}