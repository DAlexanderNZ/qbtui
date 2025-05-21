use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},  
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
            let area = self.popup_area(frame.area(), 50, 25);
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

        if self.torrent_popup == true {
            let area = self.popup_area(frame.area(), 80, 80);
            self.render_selected_torrent(frame, area);
        }

        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        self.render_torrents_table(frame, rects[0]);
        self.render_footer(frame, rects[1]);      
        
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

    /// Takes the torrent state returned from qbittorrent api and converts it to a human readable string.
    fn get_torrent_state(&self, torrent_state: Option<qbit_rs::model::State>) -> String {
        let mut display_state = String::new();
        match torrent_state {
            Some(qbit_rs::model::State::Error) => display_state = "Error".to_string() ,
            Some(qbit_rs::model::State::MissingFiles) => display_state = "Missing Files".to_string(),
            Some(qbit_rs::model::State::Uploading
                | qbit_rs::model::State::StalledUP
                | qbit_rs::model::State::ForcedUP) => display_state = "Seeding".to_string(),
            Some(qbit_rs::model::State::CheckingUP
                | qbit_rs::model::State::CheckingDL
                | qbit_rs::model::State::CheckingResumeData) => display_state = "Checking".to_string(),
            Some(qbit_rs::model::State::PausedUP) => display_state = "Completed".to_string(),
            Some(qbit_rs::model::State::QueuedUP) => display_state = "Queued".to_string(),
            Some(qbit_rs::model::State::Allocating) => display_state = "Allocating".to_string(),
            Some(qbit_rs::model::State::Downloading
                | qbit_rs::model::State::MetaDL
                | qbit_rs::model::State::ForcedDL) => display_state = "Downloading".to_string(),
            Some(qbit_rs::model::State::PausedDL) => display_state = "Paused".to_string(),
            Some(qbit_rs::model::State::StalledDL) => display_state = "Stalled".to_string(),
            Some(qbit_rs::model::State::Moving) => display_state = "Moving".to_string(),
            Some(qbit_rs::model::State::Unknown) => display_state = "Unknown".to_string(),
            _ => display_state.push_str("Very Unknown"),
        }
        display_state
    }

    /// Helper to return a centered rect given x and y percentages.
    fn popup_area(&self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center); 
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

