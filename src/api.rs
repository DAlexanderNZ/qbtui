use crate::App;
use color_eyre::Result;
use qbit_rs::{model::{Credential, GetTorrentListArg, TorrentFilter}, Qbit};

impl App {
    fn api(&self) -> Qbit {
        let credential =  Credential::new(&self.cfg.username, &self.cfg.password);
        let url: &str = &self.cfg.api_url;
        Qbit::new(url, credential)
    }

    pub async fn get_torrents(&mut self) -> Result<()> {
        let api = self.api();
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
            Ok(torrents) => self.torrents = torrents,
            // TODO: Create a popup with the error message.
            Err(_err) => {},
        }
        Ok(())
    }

    pub async fn get_torrent_trackers(&mut self) -> Result<()> {
        let api = self.api();
        let torrent  = self.torrents.get(self.state.selected().unwrap_or(0)).unwrap();
        let hash = torrent.hash.clone().unwrap();
        let trackers = api.get_torrent_trackers(hash).await;
        match trackers {
            Ok(trackers) => self.torrent_trackers = trackers,
            Err(_err) => {},
        }
        Ok(())
    }

}