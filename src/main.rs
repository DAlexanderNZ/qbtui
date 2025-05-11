use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{
    layout::{Constraint, Rect}, 
    style::{Color, Style, Stylize}, 
    text::{Line, Text}, 
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState}, 
    DefaultTerminal, Frame
};
use qbit_rs::{model::{GetTorrentListArg, TorrentFilter}, Qbit};
use qbit_rs::model::Credential;

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
    //get_torrents().await;
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    // Event stream.
    event_stream: EventStream,
    //state: TableState,
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
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        //  Render fallback text if no torrents are available.
        if self.torrents.is_empty() {
            let text = "Hello, Ratatui!\n\n\
                Created using https://github.com/ratatui/templates\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.";
            frame.render_widget(
                Paragraph::new(text)
                    .block(Block::bordered().title(title))
                    .centered(),
                frame.area(),
            )
        } else {
            let area = frame.area();
            self.render_torrents_table(frame, area);       
        }
    }

    /// Renders the torrents table in the following format:
    /// | Name | Size | Bytes Downloaded | Progress | State | DL Speed | UL Speed | ETA | Ratio |
    /// | name | size | downloaded | progress | state | dlspeed | upspeed | eta | ratio |
    fn render_torrents_table(&self, frame: &mut Frame, area: Rect) {
        let header = ["Name", "Size", "Bytes Downloaded", "Progress", "DL Speed", "UL Speed", "ETA", "Ratio"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .height(1);

        let mut rows = vec![];
        for torrent in &self.torrents {
            let size = torrent.size.unwrap_or(-1) / 1048576; // Convert to MiB

            let item = [
                torrent.name.clone().unwrap_or_else(|| String::from("")),
                format!("{:?} MiB", size.to_string()),
                torrent.downloaded.unwrap_or(-1).to_string(),
                torrent.progress.unwrap_or_else(|| -1.0).to_string(),
                //state = torrent.state.clone(),
                torrent.dlspeed.unwrap_or(-1).to_string(),
                torrent.upspeed.unwrap_or(-1).to_string(),
                torrent.eta.unwrap_or(-1).to_string(),
                torrent.ratio.unwrap().to_string(),
            ]
            .into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .height(2);
            rows.push(item);
        }

        let witdths = [
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ];

        let t = Table::new(rows,witdths)
            .header(header)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(t, area);
    }

    async fn get_torrents(&mut self) -> Result<()> {
        let credential = Credential::new("admin", "password");
        let api_url = "http://localhost:8080";
        let torrents = get_torrents(credential, api_url).await;
        match torrents {
            Ok(torrents) => self.torrents = torrents,
            Err(err) => eprintln!("Error: {}", err),
        }
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self) -> Result<()> {
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (KeyModifiers::NONE, KeyCode::Char('r')) => {
                // Refresh the torrents list.
            },
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

