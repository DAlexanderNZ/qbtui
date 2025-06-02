use crate::App;

pub enum Message {
    /// Refresh the list of torrents and other displayed torrent data.
    RefreshTorrents,
    /// Api call to get the contents of the selected torrent.
    TorrentFiles,
    /// Api call to get/refresh the selected torrent trackers.
    TorrentTrackers,
    /// Api call to get/refresh the selected torrent peers.
    TorrentPeers,
    /// Toggle the display of the torrent info popup.
    DisplayTorrentInfo,
    /// Toggle the display of the configuration editor popup.
    /// Also toggles InputMode to/from Config.
    DisplayCfgEditor,
    SaveCfg,
    /// Quit and exit the application.
    Quit,
}


impl App {
    pub async fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::RefreshTorrents => {
                let _ = self.get_torrents().await;
                // Chain messages to refresh other displayed data.
                if self.torrent_popup {
                    return self.info_tab.update_selected();
                }
            }
            Message::TorrentFiles => {
                let _ = self.get_torrent_contents().await;
            }
            Message::TorrentTrackers => {
                let _ = self.get_torrent_trackers().await;
            }
            Message::TorrentPeers => {
                let _ = self.get_torrent_peers().await;
            }
            Message::DisplayTorrentInfo => {
                self.torrent_popup = !self.torrent_popup;
            }
            Message::DisplayCfgEditor => {
                self.cfg_popup = !self.cfg_popup;
                self.input_mode.toggle_config();
                return Some(Message::RefreshTorrents);
            }
            Message::SaveCfg => {
                self.cfg = self.input.clone();
                match confy::store("qbtui", None, &self.input) {
                    Ok(_) => {},
                    Err(err) => eprintln!("Error creating config file: {}", err),
                }
                return Some(Message::DisplayCfgEditor);
            }
            // Set running to false to quit the application.
            Message::Quit => {
                self.running = false;
            }
        }
        None
    }
}