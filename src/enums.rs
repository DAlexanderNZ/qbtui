use crate::Message;

#[derive(Debug, Default)]
pub enum ScrollContext {
    #[default]
    TorrentsTable,
    InfoTab,
} 

/// Represents the currently selected tab for torrent information display.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

    pub fn next(&mut self) -> Option<Message> {
        let current_index = self.to_index();
        let new_index = (current_index + 1) % 4; // Wrap around last tab
        *self = Self::from_index(new_index);
        self.update_selected()
    }

    pub fn previous(&mut self) -> Option<Message> {
        let current_index = self.to_index();
        let new_index = if current_index == 0 {
            3 // Wrap around to the last tab
        } else {
            (current_index - 1) % 4
        };
        *self = Self::from_index(new_index);
        self.update_selected()
    }

    /// Return a message for updating the newly selected tab.
    pub fn update_selected(self) -> Option<Message> {
        match self {
            SelectedInfoTab::Files => Some(Message::TorrentFiles),
            SelectedInfoTab::Trackers => Some(Message::TorrentTrackers),
            SelectedInfoTab::Peers => Some(Message::TorrentPeers),
            _ => None,
        }            
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SelectedAddTorrentTab {
    #[default]
    MagnetLink,
    File,
}

impl SelectedAddTorrentTab {
    pub fn toggle(&mut self) {
        match self {
            SelectedAddTorrentTab::MagnetLink => *self = SelectedAddTorrentTab::File,
            SelectedAddTorrentTab::File => *self = SelectedAddTorrentTab::MagnetLink,
        };
    }
}