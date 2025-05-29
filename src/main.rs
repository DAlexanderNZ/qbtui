use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{
    layout::{Constraint, Layout},  
    widgets::{TableState, ScrollbarState}, 
    DefaultTerminal, Frame
};
use qbit_rs::model::Tracker;
use serde::{Serialize, Deserialize};
use confy;
// Local imports
mod input;
use input::{CurentInput, SelectedInfoTab, InputMode};
mod elements;
mod helpers;
mod api;
mod signals;
use signals::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    api_url: String,
    username: String,
    password: String,
}

impl ::std::default::Default for AppConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8080".into(),
            username: "admin".into(),
            password: "".into(),
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    //let cfg: AppConfig = confy::load("qbtui", None)?;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    event_stream: EventStream,
    state: TableState,
    scroll_state: ScrollbarState,
    //scroll_state: ScrollbarState,
    // Input
    // Current value of the input field
    input: AppConfig,
    current_input: CurentInput,
    // Position of the cursor in the input field
    charcter_index: usize,
    input_mode: InputMode,
    // Config handling
    cfg_popup: bool,
    first_cfg: bool,
    save_cfg: bool,
    cfg: AppConfig,
    // Torrent data storage
    torrents: Vec<qbit_rs::model::Torrent>,
    torrent_trackers: Vec<Tracker>,
    torrent_peers: Option<qbit_rs::model::PeerSyncData>,
    // Torrent info popup
    torrent_popup: bool, 
    info_tab: SelectedInfoTab,
}


impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.charcter_index = 0;
        self.cfg = confy::load("qbtui", None)?;
        self.input = self.cfg.clone();
        self.get_torrents().await?;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            let mut msg = self.handle_crossterm_events().await?;
            // TODO: With time delay regularly refresh the torrents.
            while msg.is_some() {
                msg = self.update(msg.unwrap()).await;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let rects;
        let footer: usize;
        // Split frame area depending on whether the torrent info section is active.
        if self.torrent_popup == true {
            let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(17), Constraint::Length(4)]);
            rects = vertical.split(frame.area());
            footer = 2;
        } else {
            let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
            rects = vertical.split(frame.area());
            footer = 1;
        }

        self.render_torrents_table(frame, rects[0]);
        self.render_footer(frame, rects[footer]);      

        // Show torrent info footer
        if self.torrent_popup == true  && self.torrents.len() > 0 {
            self.render_torrent_into(frame, rects[1]);
        }  else {
            self.torrent_popup = false;
        }
        
        // Show cfg popup on first run or user input.
        if self.cfg.password == "" || self.cfg_popup == true {
            // TODO: Make this a less ugly check for first run config.
            if self.cfg.password == "" {
                if self.first_cfg == false {
                    self.input_mode = InputMode::Config;
                    self.reset_cursor();
                }
                self.cfg_popup = true;
            }
            let area = self.popup_area(frame.area(), 50, 50);
            self.render_cfg_popup(frame, area);

            if self.save_cfg == true {
                
            }
        } 
    }
}

