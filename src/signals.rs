use crate::App;

pub enum Message {
    RefreshTorrents,
    TorrentTrackers,
    TorrentPeers,
}


impl App {
    pub async fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::RefreshTorrents => {
                let _ = self.get_torrents().await;
            }
            Message::TorrentTrackers => {
                let _ = self.get_torrent_trackers().await;
            }
            Message::TorrentPeers => {
                let _ = self.get_torrent_peers().await;
            }
        }
        None
    }
}