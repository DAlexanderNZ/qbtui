use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{
    layout::{Constraint, Layout},  
    widgets::TableState, 
    DefaultTerminal, Frame
};
use qbit_rs::{model::{GetTorrentListArg, TorrentFilter}, Qbit};
use qbit_rs::model::Credential;
use serde::{Serialize, Deserialize};
use confy;
// Local imports
mod input;
use input::CurentInput;
mod elements;
mod helpers;

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

async fn get_torrents(credential: Credential, api_url: &str) -> Result<Vec<qbit_rs::model::Torrent>> {
    
    let api = Qbit::new(api_url, credential);
    let arg = GetTorrentListArg {
        filter: Some(TorrentFilter::All),
        category: None,
        tag: None,
        sort: None,
        reverse: None,
        limit: Some(10),
        offset: None,
        hashes: None,
    };
    let torrents = api.get_torrent_list(arg).await;
    match torrents {
        Ok(torrents) => Ok(torrents),
        Err(e) => Err(e.into())
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

#[derive(Debug)]
enum InputMode {
    Normal,
    Config,
}

impl ::std::default::Default for InputMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    event_stream: EventStream,
    state: TableState,
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
    refresh_torrents: bool,
    torrent_popup: bool,
}


impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        //self.input = String::new();
        self.charcter_index = 0;
        self.cfg = confy::load("qbtui", None)?;
        self.input = self.cfg.clone();
        self.get_torrents().await?;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
            // TODO: With time delay regularly refresh the torrents.
            if self.refresh_torrents {
                self.get_torrents().await?;
                self.refresh_torrents = false;
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let rects;
        let footer: usize;
        // Split frame area depending on whether the torrent info section is active.
        if self.torrent_popup == true {
            let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(14), Constraint::Length(4)]);
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
            self.render_selected_torrent(frame, rects[1]);
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
                self.cfg = self.input.clone();
                self.save_cfg = false;
                match confy::store("qbtui", None, &self.cfg) {
                    Ok(_) => self.cfg_popup = false,
                    Err(err) => eprintln!("Error creating config file: {}", err),
                }
            }
        } 
    }


    /// Fetches torrent list from qbittorrent api.
    pub async fn get_torrents(&mut self) -> Result<()> {
        let credential = Credential::new(&self.cfg.username, &self.cfg.password);
        let api_url = &self.cfg.api_url;
        let torrents = get_torrents(credential, api_url).await;
        match torrents {
            Ok(torrents) => self.torrents = torrents,
            // TODO: Create a popup with the error message.
            Err(_err) => {},
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

