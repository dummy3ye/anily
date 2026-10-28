pub mod state;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Size;
use ratatui_image::{picker::Picker, protocol::Protocol, Resize};
use reqwest::Client;
use state::{AppState, HomeSection, Screen};
use std::collections::HashMap;
use std::process::Child;
use tokio::task::JoinHandle;

type EpisodeLookup = Result<(String, Vec<crate::providers::hianime::Episode>)>;

pub struct App {
    pub state: AppState,
    client: Client,
    pub picker: Picker,
    pub cover_protocols: HashMap<(u32, u16, u16), Protocol>,
    pub cover_tasks: HashMap<u32, JoinHandle<Option<crate::providers::anilist::CoverArt>>>,
    home_task: Option<JoinHandle<Result<crate::providers::anilist::HomeFeed>>>,
    episode_task: Option<JoinHandle<EpisodeLookup>>,
    mpv_process: Option<Child>,
    discord: Option<crate::rpc::discord::DiscordRpc>,
    pub exit_prompt: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            exit_prompt: false,
            state: AppState::new(),
            client: Client::builder().user_agent("anily/0.1").build()?,
            picker: Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks()),
            cover_protocols: HashMap::new(),
            cover_tasks: HashMap::new(),
            home_task: Some(tokio::spawn({
                let client = Client::builder().user_agent("anily/0.1").build()?;
                async move { crate::providers::anilist::home_feed(&client).await }
            })),
            episode_task: None,
            mpv_process: None,
            discord: crate::config::Config::load()
                .ok()
                .and_then(|cfg| cfg.discord_client_id)
                .and_then(|id| crate::rpc::discord::DiscordRpc::connect(&id)),
        })
    }

    pub async fn poll_background(&mut self) {
        if let Some(process) = self.mpv_process.as_mut() {
            if process.try_wait().ok().flatten().is_some() {
                self.mpv_process = None;
                self.state.now_playing = None;
                if let Some(discord) = self.discord.as_mut() {
                    let _ = discord.clear();
                }
            }
        }
        if self.home_task.as_ref().is_some_and(JoinHandle::is_finished) {
            if let Some(task) = self.home_task.take() {
                match task.await {
                    Ok(Ok(feed)) => {
                        self.state.results = feed.trending;
                        self.state.airing = feed.airing;
                        self.state.popular = feed.popular;
                        self.state.favorite = feed.favorite;
                        self.state.loading = false;
                    }
                    Ok(Err(error)) => {
                        self.state.error = Some(error.to_string());
                        self.state.loading = false;
                    }
                    Err(error) => {
                        self.state.error = Some(error.to_string());
                        self.state.loading = false;
                    }
                }
            }
        }
        let finished: Vec<u32> = self
            .cover_tasks
            .iter()
            .filter_map(|(id, task)| task.is_finished().then_some(*id))
            .collect();
        for id in finished {
            if let Some(task) = self.cover_tasks.remove(&id) {
                match task.await {
                    Ok(Some(cover)) => {
                        for anime in self.all_home_anime_mut() {
                            if anime.id == id {
                                anime.cover_art = Some(cover.clone());
                            }
                        }
                    }
                    Ok(None) => {
                        let retries: Vec<(u32, String)> = {
                            let mut urls = Vec::new();
                            for anime in self.all_home_anime_mut() {
                                if anime.id == id {
                                    if let Some(url) =
                                        anime.cover_large.as_ref().filter(|u| !u.is_empty())
                                    {
                                        urls.push((id, url.clone()));
                                    }
                                }
                            }
                            urls
                        };
                        for (retry_id, url) in retries {
                            let client = self.client.clone();
                            self.cover_tasks.insert(
                                retry_id,
                                tokio::spawn(async move {
                                    crate::providers::anilist::fetch_cover(&client, &url).await
                                }),
                            );
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        if self
            .episode_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            if let Some(task) = self.episode_task.take() {
                match task.await {
                    Ok(Ok((id, episodes))) => {
                        self.state.hianime_id = Some(id);
                        self.state.episodes = episodes;
                        self.state.episode_loading = false;
                    }
                    Ok(Err(error)) => {
                        self.state.episode_error = Some(error.to_string());
                        self.state.episode_loading = false;
                    }
                    Err(error) => {
                        self.state.episode_error = Some(error.to_string());
                        self.state.episode_loading = false;
                    }
                }
            }
        }
    }

    pub fn start_visible_covers(&mut self, start: usize, end: usize) {
        for anime in self.state.results[start..end].iter() {
            if anime.cover_art.is_none() && !self.cover_tasks.contains_key(&anime.id) {
                let url = if !anime.cover.is_empty() {
                    Some(anime.cover.clone())
                } else {
                    anime
                        .cover_large
                        .as_ref()
                        .filter(|u| !u.is_empty())
                        .cloned()
                };
                if let Some(url) = url {
                    let client = self.client.clone();
                    let id = anime.id;
                    self.cover_tasks.insert(
                        id,
                        tokio::spawn(async move {
                            crate::providers::anilist::fetch_cover(&client, &url).await
                        }),
                    );
                }
            }
        }
    }

    pub fn start_cover(&mut self, anime: &crate::providers::anilist::Anime) {
        if anime.cover_art.is_none() && !self.cover_tasks.contains_key(&anime.id) {
            let url = if !anime.cover.is_empty() {
                Some(anime.cover.clone())
            } else {
                anime
                    .cover_large
                    .as_ref()
                    .filter(|u| !u.is_empty())
                    .cloned()
            };
            if let Some(url) = url {
                let client = self.client.clone();
                let id = anime.id;
                self.cover_tasks.insert(
                    id,
                    tokio::spawn(async move {
                        crate::providers::anilist::fetch_cover(&client, &url).await
                    }),
                );
            }
        }
    }

    fn all_home_anime_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut crate::providers::anilist::Anime> {
        self.state
            .results
            .iter_mut()
            .chain(self.state.airing.iter_mut())
            .chain(self.state.popular.iter_mut())
            .chain(self.state.favorite.iter_mut())
    }

    pub fn cover_protocol(
        &mut self,
        anime: &crate::providers::anilist::Anime,
        width: u16,
        height: u16,
    ) -> Option<Protocol> {
        let cache_key = (anime.id, width, height);
        if let Some(protocol) = self.cover_protocols.get(&cache_key) {
            return Some(protocol.clone());
        }
        let image = anime.cover_art.as_ref()?.image.clone();
        let protocol = self
            .picker
            .new_protocol(image, Size::new(width, height), Resize::Fit(None))
            .ok()?;
        self.cover_protocols.insert(cache_key, protocol.clone());
        Some(protocol)
    }

    pub async fn load_home(&mut self) -> Result<()> {
        self.state.loading = true;
        self.state.error = None;
        self.state.results = crate::providers::anilist::trending(&self.client).await?;
        self.state.loading = false;
        Ok(())
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.exit_prompt {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.exit_prompt = false;
                    return Ok(false);
                }
                _ => return Ok(false),
            }
        }

        if key.code == KeyCode::Esc {
            if self.state.screen != Screen::Home {
                self.state.screen = Screen::Home;
                self.state.selected_anime = None;
                self.state.details = None;
                self.state.episodes.clear();
                self.state.episode_selected = 0;
                self.state.now_playing = None;
                self.state.episode_error = None;
                self.state.episode_loading = false;
                self.state.searching = false;
                self.state.section_title = "Trending".to_string();
                self.state.query.clear();
                self.state.home_section = crate::app::state::HomeSection::Trending;
                self.state.section_selected = 0;
                self.state.selected = 0;
                self.state.scroll_row = 0;
                self.state.loading = true;
                self.state.error = None;
                if let Ok(feed) = crate::providers::anilist::home_feed(&self.client).await {
                    self.state.results = feed.trending;
                    self.state.airing = feed.airing;
                    self.state.popular = feed.popular;
                    self.state.favorite = feed.favorite;
                }
                self.state.loading = false;
                return Ok(false);
            } else if self.state.screen == Screen::Home {
                if self.state.section_title == "Search results" {
                    self.state.searching = false;
                    self.state.section_title = "Trending".to_string();
                    self.state.query.clear();
                    self.state.home_section = crate::app::state::HomeSection::Trending;
                    self.state.section_selected = 0;
                    self.state.selected = 0;
                    self.state.scroll_row = 0;
                    self.state.loading = true;
                    self.state.error = None;
                    if let Ok(feed) = crate::providers::anilist::home_feed(&self.client).await {
                        self.state.results = feed.trending;
                        self.state.airing = feed.airing;
                        self.state.popular = feed.popular;
                        self.state.favorite = feed.favorite;
                    }
                    self.state.loading = false;
                    return Ok(false);
                }
                if self.state.searching {
                    self.state.searching = false;
                    self.state.section_title = "Trending".to_string();
                    self.state.query.clear();
                    self.state.home_section = crate::app::state::HomeSection::Trending;
                    self.state.section_selected = 0;
                    self.state.selected = 0;
                    self.state.scroll_row = 0;
                    self.state.loading = true;
                    self.state.error = None;
                    if let Ok(feed) = crate::providers::anilist::home_feed(&self.client).await {
                        self.state.results = feed.trending;
                        self.state.airing = feed.airing;
                        self.state.popular = feed.popular;
                        self.state.favorite = feed.favorite;
                    }
                    self.state.loading = false;
                    return Ok(false);
                }
                if !self.state.query.is_empty() {
                    self.state.query.clear();
                    self.state.section_title = "Trending".to_string();
                    self.state.home_section = crate::app::state::HomeSection::Trending;
                    self.state.section_selected = 0;
                    self.state.selected = 0;
                    self.state.scroll_row = 0;
                    self.state.loading = true;
                    self.state.error = None;
                    if let Ok(feed) = crate::providers::anilist::home_feed(&self.client).await {
                        self.state.results = feed.trending;
                        self.state.airing = feed.airing;
                        self.state.popular = feed.popular;
                        self.state.favorite = feed.favorite;
                    }
                    self.state.loading = false;
                    return Ok(false);
                }
            }
        }

        match key.code {
            KeyCode::Char('q') if (!self.state.searching || self.state.query.is_empty()) => {
                self.exit_prompt = true;
                Ok(false)
            }
            KeyCode::Enter => {
                match self.state.screen {
                    Screen::Home if self.state.searching => {
                        if self.state.query.trim().is_empty() {
                            return Ok(false);
                        }
                        self.state.searching = false;
                        self.state.loading = true;
                        self.state.error = None;
                        self.state.section_title = "Search results".to_string();
                        self.state.home_section = crate::app::state::HomeSection::Trending;
                        self.state.section_selected = 0;
                        self.state.results =
                            crate::providers::anilist::search(&self.client, &self.state.query)
                                .await?;
                        self.state.loading = false;
                        self.state.selected = 0;
                        self.state.scroll_row = 0;
                    }
                    Screen::Home => self.load_episodes().await?,
                    Screen::Details | Screen::Episodes => self.play_selected().await?,
                }
                Ok(false)
            }
            KeyCode::Char('/') if self.state.screen == Screen::Home => {
                self.state.searching = true;
                self.state.query.clear();
                self.state.section_title = "Search".to_string();
                Ok(false)
            }
            _ => {
                self.state.handle_key(key);
                Ok(false)
            }
        }
    }

    async fn load_episodes(&mut self) -> Result<()> {
        let anime = match self.state.home_section {
            HomeSection::Trending => self.state.results.get(self.state.selected),
            HomeSection::Airing => self.state.airing.get(self.state.section_selected),
            HomeSection::Popular => self.state.popular.get(self.state.section_selected),
            HomeSection::Favorite => self.state.favorite.get(self.state.section_selected),
        }
        .cloned()
        .context("no anime selected")?;

        self.state.selected_anime = Some(anime.clone());
        self.state.loading = true;
        self.state.error = None;
        self.state.details =
            Some(crate::providers::anilist::details(&self.client, anime.id).await?);
        self.state.episode_selected = 0;
        self.state.now_playing = None;
        self.state.episode_error = None;
        self.state.episode_loading = true;
        self.state.screen = Screen::Details;
        self.state.loading = false;

        self.state.episodes.clear();
        self.state.hianime_id = None;
        let client = self.client.clone();
        let title = anime.title.clone();
        self.episode_task = Some(tokio::spawn(async move {
            let matches = crate::providers::hianime::search(&client, &title).await?;
            let match_item = matches
                .first()
                .context("anime was not found on playback provider")?;
            let episodes = crate::providers::hianime::episodes(&client, &match_item.id).await?;
            Ok((match_item.id.clone(), episodes))
        }));
        Ok(())
    }

    async fn play_selected(&mut self) -> Result<()> {
        if self.state.episodes.is_empty() {
            return Err(anyhow::anyhow!(
                "No playable episodes are available from the playback provider"
            ));
        }
        let episode = self
            .state
            .episodes
            .get(self.state.episode_selected)
            .cloned()
            .context("no episode selected")?;
        let stream =
            crate::providers::hianime::stream(&self.client, &episode, self.state.dubbed).await?;
        if let Some(mut process) = self.mpv_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        let selected = self.state.episode_selected;
        let mut command = std::process::Command::new("mpv");
        command
            .arg("--really-quiet")
            .arg("--no-terminal")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg(format!("--referrer={}", stream.referer));
        if let Some(subtitles) = stream.subtitles {
            command.arg(format!("--sub-file={subtitles}"));
        }
        command.arg(stream.video);
        self.mpv_process = Some(command.spawn().context("starting mpv")?);
        self.state.now_playing = Some(selected);
        if let Some(discord) = self.discord.as_mut() {
            let cover_url = self.state.selected_anime.as_ref().and_then(|anime| {
                if anime.cover.is_empty() {
                    anime.cover_large.as_deref()
                } else {
                    Some(anime.cover.as_str())
                }
            });
            discord.set_episode(
                self.state
                    .selected_anime
                    .as_ref()
                    .map_or("Anime", |anime| anime.title.as_str()),
                &episode.number,
                cover_url,
            )?;
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(mut process) = self.mpv_process.take() {
            let _ = process.kill();
        }
    }
}
