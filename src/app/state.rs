use crate::providers::anilist::Anime;
use crate::providers::hianime::Episode;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Details,
    Episodes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeSection {
    Trending,
    Airing,
    Popular,
    Favorite,
}

#[derive(Debug)]
pub struct AppState {
    pub screen: Screen,
    pub query: String,
    pub searching: bool,
    pub home_section: HomeSection,
    pub section_selected: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub results: Vec<Anime>,
    pub airing: Vec<Anime>,
    pub popular: Vec<Anime>,
    pub favorite: Vec<Anime>,
    pub selected: usize,
    pub selected_anime: Option<Anime>,
    pub details: Option<crate::providers::anilist::AnimeDetails>,
    pub episodes: Vec<Episode>,
    pub episode_loading: bool,
    pub episode_error: Option<String>,
    pub episode_selected: usize,
    pub now_playing: Option<usize>,
    pub hianime_id: Option<String>,
    pub dubbed: bool,
    pub grid_columns: usize,
    pub section_title: String,
    pub scroll_row: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            query: String::new(),
            searching: false,
            home_section: HomeSection::Trending,
            section_selected: 0,
            loading: true,
            error: None,
            results: Vec::new(),
            airing: Vec::new(),
            popular: Vec::new(),
            favorite: Vec::new(),
            selected: 0,
            selected_anime: None,
            details: None,
            episodes: Vec::new(),
            episode_loading: false,
            episode_error: None,
            episode_selected: 0,
            now_playing: None,
            hianime_id: None,
            dubbed: false,
            grid_columns: 6,
            section_title: "Trending".to_string(),
            scroll_row: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match (self.screen, key.code) {
            (Screen::Home, KeyCode::Char(c)) if self.searching => self.query.push(c),
            (Screen::Home, KeyCode::Backspace) if self.searching => {
                self.query.pop();
            }
            (Screen::Home, KeyCode::Down | KeyCode::Char('j')) => match self.home_section {
                HomeSection::Trending => {
                    self.home_section = HomeSection::Airing;
                    self.section_selected = 0;
                }
                _ => self.section_selected = self.section_selected.saturating_add(1).min(4),
            },
            (Screen::Home, KeyCode::Left | KeyCode::Char('h')) => match self.home_section {
                HomeSection::Trending => self.selected = self.selected.saturating_sub(1),
                HomeSection::Airing => {}
                HomeSection::Popular => self.home_section = HomeSection::Airing,
                HomeSection::Favorite => self.home_section = HomeSection::Popular,
            },
            (Screen::Home, KeyCode::Right | KeyCode::Char('l')) => match self.home_section {
                HomeSection::Trending => {
                    self.selected = (self.selected + 1).min(self.results.len().saturating_sub(1))
                }
                HomeSection::Airing => self.home_section = HomeSection::Popular,
                HomeSection::Popular => self.home_section = HomeSection::Favorite,
                HomeSection::Favorite => {}
            },
            (Screen::Home, KeyCode::PageDown) => {
                self.selected = (self.selected + self.grid_columns * 3)
                    .min(self.results.len().saturating_sub(1));
                self.scroll_row += 3;
            }
            (Screen::Home, KeyCode::PageUp) => {
                self.selected = self.selected.saturating_sub(self.grid_columns * 3);
                self.scroll_row = self.scroll_row.saturating_sub(3);
            }
            (Screen::Home, KeyCode::Up | KeyCode::Char('k')) => match self.home_section {
                HomeSection::Trending => {
                    self.selected = self.selected.saturating_sub(self.grid_columns)
                }
                _ => {
                    self.home_section = HomeSection::Trending;
                    self.selected = 0;
                }
            },
            (Screen::Details | Screen::Episodes, KeyCode::Up) => {
                self.episode_selected = self.episode_selected.saturating_sub(1);
            }
            (Screen::Details | Screen::Episodes, KeyCode::Down) => {
                self.episode_selected =
                    (self.episode_selected + 1).min(self.episodes.len().saturating_sub(1));
            }
            (Screen::Details | Screen::Episodes, KeyCode::Char('d')) => self.dubbed = !self.dubbed,
            _ => {}
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
