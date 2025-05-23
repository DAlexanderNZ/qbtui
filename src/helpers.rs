use crate::App;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use chrono::{DateTime};

impl App {
    /// Takes the torrent state returned from qbittorrent api and converts it to a human readable string.
    pub fn get_torrent_state(&self, torrent_state: Option<qbit_rs::model::State>) -> String {
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
    pub fn popup_area(&self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center); 
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }

    /// Convert unix timestamp to human readable string.
    pub fn timestamp_human_readable(&self, timestamp: Option<i64>) -> String {
        match timestamp  {
            Some(ts)  => {
                 if let Some(datetime) = DateTime::from_timestamp(ts, 0) {
                     datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                 } else {
                        "Invalid timestamp".to_string()
                 }
                }
            _ =>  "N/A".to_string()
        }
    }

    /// Convert bytes to human readable string.
    pub fn format_bytes (&self, bytes: i64) -> String {
        let mut bytes = bytes as f64;
        let units = ["B", "KiB", "MiB", "GiB", "TiB"];
        let mut unit = 0;
        while bytes >= 1024.0 && unit < units.len() - 1 {
            bytes /= 1024.0;
            unit += 1;
        }
        if bytes == 0.0 {
            format!("{:.0} {}", bytes, units[unit])
        } else {
            format!("{:.2} {}", bytes, units[unit])
        }
    }

    /// Convert transfer rate bytes/s to human readable string.
    pub fn format_rate(&self, rate: i64) -> String {
        // TODO: Choose if bits/s or bytes/s is more appropriate.
        let mut rate = rate as f64;
        let units = ["B/s", "KiB/s", "MiB/s", "GiB/s", "TiB/s"];
        let mut unit = 0;
        while (rate) >= 1024.0 && unit < units.len() - 1 {
            rate /= 1024.0;
            unit += 1;
        }
        if rate == 0.0 {
            format!("{:.0} {}", rate, units[unit])
        } else {
            format!("{:.2} {}", rate, units[unit])
        }
    }
}

