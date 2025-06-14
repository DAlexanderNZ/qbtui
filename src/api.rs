use crate::{signals::Message, App};
use std::fs;
use std::str::FromStr;
use color_eyre::Result;
use qbit_rs::{
    model::{AddTorrentArg, Credential, GetTorrentListArg, Sep, TorrentFile, TorrentFilter, TorrentSource}, 
    Qbit};

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

    /// Torrent contents is a vector of details about the files in a torrent.
    pub async fn get_torrent_contents(&mut self) -> Result<()> {
        let api = self.api();
        let torrent = self.torrents.get(self.state.selected().unwrap_or(0)).unwrap();
        let hash = torrent.hash.clone().unwrap();
        let content = api.get_torrent_contents(hash, None).await;
        match content {
            Ok(content) => self.torrent_content = content,
            Err(err) => {println!("Error getting torrent content {:?}", err) },
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

    pub async fn get_torrent_peers(&mut self) -> Result<()> {
        let api = self.api();
        let torrent = self.torrents.get(self.state.selected().unwrap_or(0)).unwrap();
        let hash = torrent.hash.clone().unwrap();
        // From the qBittorrent API documentation 5.0:
        // Response ID. If not provided, rid=0 will be assumed. 
        // If the given rid is different from the one of last server reply, 
        // full_update will be true (see the server reply details for more info)
        let peers = api.get_torrent_peers(hash, None).await;
        match peers {
            Ok(peers) => self.torrent_peers = Some(peers),
            Err(_err) => self.torrent_peers = None,
        }
        Ok(())
    }

    /// Takes [`App`] magnet_link and passes the magnet link to the API.
    pub async fn add_torrent_magnet(&mut self) -> Result<Message> {
        let magnet = self.magnet_link.clone();
        if magnet.is_empty() {
            return Err(color_eyre::eyre::eyre!("Magnet link is empty"));
        }
        let url = match Sep::from_str(magnet.as_str()) {
            Ok(url) => url,
            Err(_) => return Err(color_eyre::eyre::eyre!("Invalid magnet link format")),
        };
        let torrent_source = TorrentSource::Urls { urls: url };
        Ok(self.add_torrent(torrent_source).await?)
    }

    /// Takes [`App`] torrent_file_path and passes the torrent file to the API.
    pub async fn add_torrent_file(&mut self) -> Result<Message> {
        let file_path = self.torrent_file_path.clone();
        if file_path.is_empty() {
            return Err(color_eyre::eyre::eyre!("Torrent file path is empty"));
        }
        // We read and pass the raw file into API
        let file_data: Vec<u8> =  match fs::read(&file_path) {
            Ok(data) => data,
            Err(_) => return Err(color_eyre::eyre::eyre!("Failed to read torrent file")),
        };
        let torrent_file = TorrentFile {
            // Unsure if I should be truncating the path to just the file name but works as is.
            filename: file_path, 
            data: file_data,
        };
        let torrent_source = TorrentSource::TorrentFiles { torrents: vec![torrent_file] };
        Ok(self.add_torrent(torrent_source).await?)
    }

    /// Given a [`TorrentSource`], adds the torrent in qBittorrent.
    async fn add_torrent(&mut self, source: TorrentSource) -> Result<Message> {
        let api = self.api();
        let torrent = AddTorrentArg {
            source: source,
            savepath: None,
            cookie: None,
            category: None,
            tags: None,
            skip_checking: None,
            paused: None,
            root_folder: None,
            rename: None,
            up_limit: None,
            download_limit: None,
            ratio_limit: None,
            seeding_time_limit: None,
            auto_torrent_management: None,
            sequential_download: None,
            first_last_piece_priority: None,
        };
        let result = api.add_torrent(torrent).await;
        match result {
            Ok(_) => {
                return Ok(Message::DisplayAddTorrent);
            },
            Err(_err) => {}
        }
        Ok(Message::RefreshTorrents)
    }
}