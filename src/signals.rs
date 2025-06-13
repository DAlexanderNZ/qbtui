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
    /// Toggle the display of the add torrent popup.
    DisplayAddTorrent,
    /// Api call to add a torrent using a magnet link.
    AddTorrentMagnet,
    /// Toggle the display of the configuration editor popup.
    /// Also toggles InputMode to/from Config.
    DisplayCfgEditor,
    /// Save the current configuration to disk.
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
            Message::DisplayAddTorrent => {
                self.add_torrent_popup = !self.add_torrent_popup;
                self.input_mode.toggle_add_torrent();
                self.reset_cursor();
                return Some(Message::RefreshTorrents);
            }
            Message::AddTorrentMagnet => {
                match self.add_torrent_magnet().await {
                    Ok(msg) => return Some(msg),
                    Err(err) => eprintln!("Error adding torrent: {}", err),
                };
            }
            Message::DisplayCfgEditor => {
                self.cfg_popup = !self.cfg_popup;
                self.input_mode.toggle_config();
                self.reset_cursor();
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