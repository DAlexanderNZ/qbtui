use crate::{signals::Message, App, SelectedInfoTab, ScrollContext};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use chrono::DateTime;

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

    /// Takes the tracker state returned from qbittorrent api and converts it to a human readable string.
    pub fn get_tracker_status(&self, status: qbit_rs::model::TrackerStatus) -> String {
        let display_state = match status {
            qbit_rs::model::TrackerStatus::Disabled => "Disabled".to_string(),
            qbit_rs::model::TrackerStatus::NotContacted => "Not Contacted".to_string(),
            qbit_rs::model::TrackerStatus::Working => "Working".to_string(),
            qbit_rs::model::TrackerStatus::Updating => "Updating".to_string(),
            qbit_rs::model::TrackerStatus::NotWorking => "Not Working".to_string(),
        };
        display_state   
    }

    /// Takes the torrent priority returned from qbittorrent api and converts it to a human readable string.
    pub fn format_priority(&self, priority: qbit_rs::model::Priority) -> String {
        match priority {
            qbit_rs::model::Priority::DoNotDownload => "Do Not Download".to_string(),
            qbit_rs::model::Priority::Normal => "Normal".to_string(),
            qbit_rs::model::Priority::Mixed => "Medium".to_string(),
            qbit_rs::model::Priority::High => "High".to_string(),
            qbit_rs::model::Priority::Maximal => "Max".to_string(),
        }
    }

    /// Helper to return a centered rect given x and y percentages.
    pub fn popup_area(&self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center); 
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }

    /// Update the scrollbar state for the current info tabs content length.
    pub fn info_tab_scrollbar(&mut self, length: usize, viewport: usize) {
        self.info_tab_scroll_state = self.info_tab_scroll_state.content_length(length).viewport_content_length(viewport);
        // Update the ScollContext to InfoTab.
        self.scroll_context = ScrollContext::InfoTab;
    }

    /// Returns the length of the number of elelments in the info tab.
    /// This is used to give the correct length to the scrollbar.
    fn info_tab_elements_length(&self) -> usize {
        match self.info_tab {
            SelectedInfoTab::Trackers => self.torrent_trackers.len(),
            SelectedInfoTab::Peers => self.torrent_peers.as_ref().unwrap().peers.as_ref().unwrap().len(),
            SelectedInfoTab::Files => self.torrent_content.len(),
            SelectedInfoTab::Details => 0 // Details tab does not have elements 
        }
    }

    /// Move the scrollbar state down by one position in the current ScrollContext.
    /// Returns an optional message if update is needed.
    pub fn scroll_down(&mut self) -> Option<Message> {
        match self.scroll_context {
            ScrollContext::TorrentsTable => {
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
                if self.torrent_popup {
                    return self.info_tab.update_selected();
                }
            },
            ScrollContext::InfoTab => {
                let i = match self.info_tab_state.selected() {
                    Some(i) => {
                        if i >= self.info_tab_elements_length() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.info_tab_state.select(Some(i));
                self.info_tab_scroll_state = self.info_tab_scroll_state.position(i);
            }
        }
        None
    }

    /// Move the scrollbar state up by one position in the current ScrollContext.
    /// Returns an optional message if update is needed.
    pub fn scroll_up(&mut self) -> Option<Message>{
        match self.scroll_context {
            ScrollContext::TorrentsTable => {

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
                if self.torrent_popup {
                    return self.info_tab.update_selected();
                }
            },
            ScrollContext::InfoTab => {
                let i = match self.info_tab_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.info_tab_elements_length() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.info_tab_state.select(Some(i));
                self.info_tab_scroll_state = self.info_tab_scroll_state.position(i);
            }
        }
        None            
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

    /// Convert seconds elapsed to formated string.
    /// Format: 1W:2D:3H:4M:5S
    pub fn format_seconds(&self, mut seconds: i64) -> String {
        // ETA returns 8640000 when the torrent is complete
        if seconds == 8640000 {
            return "0".to_string();
        }
        let weeks = seconds / 604800;
        seconds %= 604800;
        let days = seconds / 86400;
        seconds %= 86400;
        let hours = seconds / 3600;
        seconds %= 3600;
        let minutes = seconds / 60;
        seconds %= 60;
        //format!("{:02}D:{:02}H:{:02}M:{:02}S", days, hours, minutes, seconds)
        let formatted = [
            (weeks, "W"),
            (days, "D"),
            (hours, "H"),
            (minutes, "M"),
            (seconds, "S"),
        ]
        .iter()
        .filter(|(v, _)| *v > 0)
        .map(|(v, s)| format!("{}{}", v, s))
        .collect::<Vec<_>>()
        .join(":");
        if formatted.is_empty() {
            "0".to_string()
        } else {
            formatted
        }
    }
}

