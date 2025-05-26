use crate::{App, CurentInput};
use ratatui::{
    layout::{Constraint, Alignment, Position, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Scrollbar, ScrollbarOrientation,  Gauge},
    Frame
};

const TABLE_ITEM_HEIGHT: usize = 2;
const INFO_TEXT: [&str; 2] = [
    "(Esc) quit | (Tab) details | (↑) move up | (↓) move down | (←) move left | (→) move right",
    "(Ctrl + e) edit cfg | (r) refresh | (k) move up | (j) move down | (h) move left | (l) move right",
];

impl App {
     /// Takes the INFO_TEXT and renders it as a widget.
    pub fn render_footer(&self, frame: &mut Frame, area: Rect) {
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
    pub fn render_cfg_popup(&self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical(
            [Constraint::Length(5), Constraint::Length(4)]
        );
        let rects = vertical.split(area);
        let block = Block::bordered().style(Style::new().fg(Color::White).bg(Color::Black));
        let rendered_password: String = std::iter::repeat("*")
            .take(self.input.password.len()).collect();
        let cfg_text = vec![
            Line::from(format!("API URL: {}", self.input.api_url.as_str())),
            Line::from(format!("Username: {}", self.input.username.as_str())),
            Line::from(format!("Password: {}", rendered_password.as_str())),
        ];
        let cfg_paragraph = Paragraph::new(cfg_text)
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone().title(" Edit config ").title_alignment(Alignment::Center))
            .alignment(Alignment::Left);
        frame.render_widget(cfg_paragraph, rects[0]);
        let cfg_save_text = vec![
            Line::from("Press (Ctrl + e) to close this popup (without saving)."),
            Line::from("Press (Ctrl + s) to save the config."),
        ];
        let cfg_save_paragraph = Paragraph::new(cfg_save_text)
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone())
            .alignment(Alignment::Left);
        frame.render_widget(cfg_save_paragraph, rects[1]);

        // Render the input cursor
        let (label, line_index) = match self.current_input {
            CurentInput::ApiUrl => ("API URL: ", 1),
            CurentInput::Username => ("Username: ", 2),
            CurentInput::Password => ("Password: ", 3),
        };
        // Get the cordinates of the required cursor location
        let x = rects[0].x + label.len() as u16 + self.charcter_index as u16 + 1;
        let y = rects[0].y + line_index as u16;
        frame.set_cursor_position(Position::new(x, y));
    }

    /// Renders the torrents table in the following format:
    /// | Name | Size | Bytes Downloaded | Progress | State | DL Speed | UL Speed | ETA | Ratio |
    /// | name | size | downloaded | progress | state | dlspeed | upspeed | eta | ratio |
    pub fn render_torrents_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = ["Name", "Size", "Bytes Downloaded", "Progress", "State" ,"DL Speed", "UL Speed", "ETA", "Ratio"]
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

            let size = self.format_bytes(torrent.size.unwrap_or(0));
            let downloaded = self.format_bytes(torrent.downloaded.unwrap_or(0));
            //TODO: Create a progress bar from the percentage
            // Unsure if this can be done due to Cell only accepting strings and widgets::Gauge 
            // not supporting being rendered as text.
            let progress = torrent.progress.unwrap_or_else(|| -1.0) * 100.0; // Convert to percentage 
            let display_state = self.get_torrent_state(torrent.state.clone());                              
            let dlspeed = self.format_rate(torrent.dlspeed.unwrap_or(0));
            let upspeed = self.format_rate(torrent.upspeed.unwrap_or(0));
            let eta = self.format_seconds(torrent.eta.unwrap_or(0));
            let ratio = torrent.ratio.unwrap_or(-1.0);

            let item: Row<'_> = [
                torrent.name.clone().unwrap_or_else(|| String::from("")),
                format!("{}", size),
                format!("{}", downloaded),
                format!("{:.2}%", progress),
                display_state,
                format!("{}", dlspeed),
                format!("{}", upspeed),
                format!("{}", eta),
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

        // Render the scrollbar on the right side of the table
        self.scroll_state = self.scroll_state.content_length(self.torrents.len()).viewport_content_length(TABLE_ITEM_HEIGHT);
        frame.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut self.scroll_state);
    }

    /// Renders detailed information about the selected torrent in a footer.
    /// The popup contains a progress bar, torrent transfer info, and file/torrent info.
    pub fn render_selected_torrent(&self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical(
            [Constraint::Length(3), Constraint::Length(7), Constraint::Length(4)]
        );
        let rects = vertical.split(area);
        let block = Block::bordered().style(Style::new().fg(Color::White).bg(Color::Black));
        let selected_torrent = self.torrents.get(self.state.selected().unwrap_or(0)).unwrap();
        let torrent_name = selected_torrent.name.clone().unwrap_or_else(|| String::from(""));
        // Progress bar
        let progress = Gauge::default()
            .style(Style::new().fg(Color::White).bg(Color::Black))
            .block(block.clone().title(torrent_name).title_alignment(Alignment::Center))
            .gauge_style(Style::new().fg(Color::Green).bg(Color::Black))
            .percent((selected_torrent.progress.unwrap_or_else(|| 0.0) * 100.0) as u16);
        frame.render_widget(progress, rects[0]);

        // Verbose torrent transfer info
        let mut rows = vec![];
        let eta = 
                if selected_torrent.eta.unwrap_or(-1) == 8640000 { 0 } // Default value when completed
                else { selected_torrent.eta.unwrap_or(-1) / 60};
        let row_one: Row<'_> = [
            format!("Time Active: {}", self.format_seconds(selected_torrent.time_active.unwrap_or(0))),
            format!("Eta: {}", self.format_seconds(eta)),
            format!("Connections: {:?}", selected_torrent.num_complete.unwrap_or(-1))
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows.push(row_one);
        let row_two: Row<'_> = [
            format!("Downloaded: {}", self.format_bytes(selected_torrent.downloaded.unwrap_or(0))),
            format!("Uploaded: {}", self.format_bytes(selected_torrent.uploaded.unwrap_or(0))),
            format!("Seeds: {:?}", selected_torrent.num_seeds.unwrap_or(-1))
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows.push(row_two);
        let row_three: Row<'_> = [
            format!("Download Speed: {}", self.format_rate(selected_torrent.dlspeed.unwrap_or(0))),
            format!("Upload Speed: {}", self.format_rate(selected_torrent.upspeed.unwrap_or(0))),
            format!("Peers: {:?}", selected_torrent.num_incomplete.unwrap_or(-1))
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows.push(row_three);
        let row_four: Row<'_> = [
            format!("Download Limit: {:?}", selected_torrent.dl_limit.unwrap_or(-1)),
            format!("Upload Limit: {:?}", selected_torrent.up_limit.unwrap_or(-1)),
            format!("Sequential Dl: {:?}", selected_torrent.seq_dl.unwrap())
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows.push(row_four);
        let row_five: Row<'_> = [
            format!("Share Ratio: {:.6}", selected_torrent.ratio.unwrap_or(-1.0)),
            format!("Status: {}", self.get_torrent_state(selected_torrent.state.clone())),
            format!("Last Seen Complete: {}", self.timestamp_human_readable(selected_torrent.last_activity))
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows.push(row_five);

        let widths = [
            Constraint::Percentage(33), 
            Constraint::Percentage(33),
            Constraint::Percentage(33), 
        ];
        let t = Table::new(rows, &widths)
        .block(block.clone().title("Transfer").title_alignment(Alignment::Center));
        frame.render_widget(t, rects[1]);

        // File/torrent info
        let mut rows_two = vec![];
        let row_one = [
            format!("Total Size: {}", self.format_bytes(selected_torrent.size.unwrap_or(0))),
            format!("Hash: {}", selected_torrent.hash.clone().unwrap()),
            format!("Save Path: {}", selected_torrent.save_path.clone().unwrap())
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows_two.push(row_one);
        let row_two = [
            format!("Added On: {}", self.timestamp_human_readable(selected_torrent.added_on)),
            format!("Completed On: {}", self.timestamp_human_readable(selected_torrent.completion_on)),
            format!("Tracker: {}", selected_torrent.tracker.clone().unwrap())
        ]
        .into_iter()
        .map(|content| Cell::new(content))
        .collect::<Row>();
        rows_two.push(row_two);
        let t = Table::new(rows_two, widths)
        .block(block.clone().title("Information").title_alignment(Alignment::Center));
        frame.render_widget(t, rects[2]);
        
    }
}