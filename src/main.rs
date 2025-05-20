use color_eyre::Result;
use crossterm::event::EventStream;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect}, 
    style::{Color, Modifier, Style, Stylize}, 
    text::{Line, Text}, 
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState}, 
    DefaultTerminal, Frame
};
use qbit_rs::{model::{GetTorrentListArg, TorrentFilter}, Qbit};
use qbit_rs::model::Credential;
use serde::{Serialize, Deserialize};
use confy;

mod input;

const TABLE_ITEM_HEIGHT: usize = 2;
const INFO_TEXT: [&str; 2] = [
    "(Esc) quit | (↑) move up | (↓) move down | (←) move left | (→) move right",
    "(Ctrl + e) edit cfg | (r) refresh | (k) move up | (j) move down | (h) move left | (l) move right",
];

#[derive(Debug, Serialize, Deserialize)]
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
    input: String,
    // Position of the cursor in the input field
    charcter_index: usize,
    input_mode: InputMode,
    // Config handling
    cfg_popup: bool,
    save_cfg: bool,
    cfg: AppConfig,
    // Torrent data storage
    torrents: Vec<qbit_rs::model::Torrent>,
}


impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.input = String::new();
        self.charcter_index = 0;
        self.cfg = confy::load("qbtui", None)?;
        self.get_torrents().await?;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
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
            let area = self.popup_area(frame.area(), 50, 25);
            self.render_cfg_popup(frame, area);

            if self.save_cfg == true {
                self.save_cfg = false;
                match confy::store("qbtui", None, &self.cfg) {
                    Ok(_) => self.cfg_popup = false,
                    Err(err) => eprintln!("Error creating config file: {}", err),
                }
            }
        }

        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        self.render_torrents_table(frame, rects[0]);
        self.render_footer(frame, rects[1]);      
        
    }

    /// Takes the INFO_TEXT and renders it as a widget.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .centered()
            .block(Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::White).bg(Color::Black)));
        frame.render_widget(info, area);
    }

    /// Renders the config popup.
    /// Takes user input for api_url, username and password.
    fn render_cfg_popup(&self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical(
            [Constraint::Length(5), Constraint::Length(5), Constraint::Length(4)]
        );
        let rects = vertical.split(area);
        let block = Block::bordered();
        let cfg_text = vec![
            Line::from(format!("API URL: {}", self.input.as_str())),
            Line::from("Username:"),
            Line::from("Password:"),
        ];
        let cfg_paragraph = Paragraph::new(cfg_text)
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone().title(" Edit config ").title_alignment(Alignment::Center))
            .alignment(Alignment::Left);
        frame.render_widget(cfg_paragraph, rects[0]);
        let cfg_input = vec![
            Line::from(self.cfg.api_url.clone()),
            Line::from(self.cfg.username.clone()),
            Line::from(std::iter::repeat("•").take(self.cfg.password.len()).collect::<String>())
        ];
        let cfg_input_paragraph = Paragraph::new(cfg_input)
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone())
            .alignment(Alignment::Left);
        frame.render_widget(cfg_input_paragraph, rects[1]);
        let cfg_save_text = vec![
            Line::from("Press (Ctrl + e) to close this popup."),
            Line::from("Press (Ctrl + s) to save the config."),
        ];
        let cfg_save_paragraph = Paragraph::new(cfg_save_text)
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone())
            .alignment(Alignment::Left);
        frame.render_widget(cfg_save_paragraph, rects[2]);
    }

    /// Renders the torrents table in the following format:
    /// | Name | Size | Bytes Downloaded | Progress | State | DL Speed | UL Speed | ETA | Ratio |
    /// | name | size | downloaded | progress | state | dlspeed | upspeed | eta | ratio |
    fn render_torrents_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = ["Name", "Size", "Bytes Downloaded", "Progress", "State" ,"DL Speed", "UL Speed", "ETA (Min)", "Ratio"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().bold().fg(Color::White).bg(Color::Black))
            .height(1);

        let selected_row_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightBlue)
            .fg(Color::Black);
        let selected_col_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightBlue)
            .fg(Color::Black);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Blue)
            .fg(Color::Black);

        let mut rows = vec![];
        for (i, torrent) in self.torrents.iter().enumerate() {
            let color = if i % 2 == 0 {
                Color::DarkGray
            } else {
                Color::Black
            };

            let size = torrent.size.unwrap_or(-1) / (1024 * 1024); // Convert to MiB
            let downloaded = torrent.downloaded.unwrap_or(-1) / (1024 * 1024); // Convert to MiB
            //TODO: Create a progress bar from the percentage
            // Unsure if this can be done due to Cell only accepting strings and widgets::Gauge 
            // not supporting being rendered as text.
            let progress = torrent.progress.unwrap_or_else(|| -1.0) * 100.0; // Convert to percentage 
            let display_state = self.get_torrent_state(torrent.state.clone());                              
            let dlspeed = torrent.dlspeed.unwrap_or(-1) / 1024; // Convert to KiB/s
            let upspeed = torrent.upspeed.unwrap_or(-1) / 1024; // Convert to KiB/s
            let eta = 
                if torrent.eta.unwrap_or(-1) == 8640000 { 0 } // Default value when completed
                else { torrent.eta.unwrap_or(-1) / 60}; // Convert to minutes
            let ratio = torrent.ratio.unwrap_or(-1.0);

            let item: Row<'_> = [
                torrent.name.clone().unwrap_or_else(|| String::from("")),
                format!("{:?} MiB", size),
                format!("{:?} MiB", downloaded),
                format!("{:.2}%", progress),
                display_state,
                format!("{:?} KiB/s", dlspeed),
                format!("{:?} KiB/s", upspeed),
                format!("{:?}", eta),
                format!("{:.4}", ratio),
            ]
            .into_iter()
            .map(|content| Cell::new(content))
            .collect::<Row>()
            .style(Style::default().fg(Color::White).bg(color))
            .height(TABLE_ITEM_HEIGHT as u16);
            rows.push(item);
        }

        let witdths = [
            Constraint::Percentage(27), // Name
            Constraint::Percentage(10), // Size
            Constraint::Percentage(13), // Bytes Downloaded
            Constraint::Percentage(6), // Progress
            Constraint::Percentage(8), // State
            Constraint::Percentage(9), // DL Speed
            Constraint::Percentage(9), // UL Speed
            Constraint::Percentage(10), // ETA
            Constraint::Percentage(10), // Ratio
        ];

        let t = Table::new(rows,witdths)
            .header(header)
            .block(Block::default().borders(Borders::ALL))
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style);

        frame.render_stateful_widget(t, area, &mut self.state);
    }

    /// Fetches torrent list from qbittorrent api.
    async fn get_torrents(&mut self) -> Result<()> {
        let credential = Credential::new(&self.cfg.username, &self.cfg.password);
        let api_url = &self.cfg.api_url;
        let torrents = get_torrents(credential, api_url).await;
        match torrents {
            Ok(torrents) => self.torrents = torrents,
            Err(err) => eprintln!("Error: {}", err),
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

