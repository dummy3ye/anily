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
    pub compact_scroll: usize,
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
            compact_scroll: 0,
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
        if self.screen == Screen::Home && self.searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.query.push(c);
                    return;
                }
                KeyCode::Backspace => {
                    self.query.pop();
                    return;
                }
                _ => return,
            }
        }
        match (self.screen, key.code) {
            (Screen::Home, KeyCode::Down | KeyCode::Char('j')) => {
                if self.section_title == "Search results" {
                    let columns = self.grid_columns.max(1);
                    let new_selected = self.selected + columns;
                    if new_selected < self.results.len() {
                        self.selected = new_selected;
                    }
                    return;
                }
                if self.section_title == "Trending" && self.query.is_empty() {
                    match self.home_section {
                        HomeSection::Trending => {
                            self.home_section = HomeSection::Airing;
                            self.section_selected = 0;
                            self.compact_scroll = 0;
                            return;
                        }
                        HomeSection::Airing => {
                            self.section_selected = (self.section_selected + 1)
                                .min(self.airing.len().saturating_sub(1));
                            return;
                        }
                        HomeSection::Popular => {
                            self.section_selected = (self.section_selected + 1)
                                .min(self.popular.len().saturating_sub(1));
                            return;
                        }
                        HomeSection::Favorite => {
                            self.section_selected = (self.section_selected + 1)
                                .min(self.favorite.len().saturating_sub(1));
                            return;
                        }
                    }
                }
                let columns = self.grid_columns.max(1);
                let new_selected = self.selected + columns;
                if new_selected < self.results.len() {
                    self.selected = new_selected;
                }
            }
            (Screen::Home, KeyCode::Left | KeyCode::Char('h')) => {
                if self.section_title == "Search results" {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    return;
                }
                if self.section_title == "Trending" && self.query.is_empty() {
                    match self.home_section {
                        HomeSection::Trending => {
                            self.selected = self.selected.saturating_sub(1);
                            return;
                        }
                        HomeSection::Airing => {}
                        HomeSection::Popular => {
                            self.home_section = HomeSection::Airing;
                            self.compact_scroll = 0;
                        }
                        HomeSection::Favorite => {
                            self.home_section = HomeSection::Popular;
                            self.compact_scroll = 0;
                        }
                    }
                    return;
                }
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            (Screen::Home, KeyCode::Right | KeyCode::Char('l')) => {
                if self.section_title == "Search results" {
                    self.selected = (self.selected + 1).min(self.results.len().saturating_sub(1));
                    return;
                }
                if self.section_title == "Trending" && self.query.is_empty() {
                    match self.home_section {
                        HomeSection::Trending => {
                            self.selected =
                                (self.selected + 1).min(self.results.len().saturating_sub(1));
                            return;
                        }
                        HomeSection::Airing => {
                            self.home_section = HomeSection::Popular;
                            self.compact_scroll = 0;
                        }
                        HomeSection::Popular => {
                            self.home_section = HomeSection::Favorite;
                            self.compact_scroll = 0;
                        }
                        HomeSection::Favorite => {}
                    }
                    return;
                }
                self.selected = (self.selected + 1).min(self.results.len().saturating_sub(1));
            }
            (Screen::Home, KeyCode::PageDown) => {
                if self.home_section == HomeSection::Trending {
                    self.selected = (self.selected + self.grid_columns * 3)
                        .min(self.results.len().saturating_sub(1));
                    self.scroll_row += 3;
                }
            }
            (Screen::Home, KeyCode::PageUp) => {
                if self.home_section == HomeSection::Trending {
                    self.selected = self.selected.saturating_sub(self.grid_columns * 3);
                    self.scroll_row = self.scroll_row.saturating_sub(3);
                }
            }
            (Screen::Home, KeyCode::Up | KeyCode::Char('k')) => {
                if self.section_title == "Search results" {
                    let columns = self.grid_columns.max(1);
                    if self.selected >= columns {
                        self.selected -= columns;
                    }
                    return;
                }
                if self.section_title == "Trending" && self.query.is_empty() {
                    match self.home_section {
                        HomeSection::Trending => {
                            self.selected = self.selected.saturating_sub(self.grid_columns);
                            return;
                        }
                        HomeSection::Airing => {
                            if self.section_selected == 0 {
                                self.home_section = HomeSection::Trending;
                            } else {
                                self.section_selected = self.section_selected.saturating_sub(1);
                            }
                            return;
                        }
                        HomeSection::Popular => {
                            if self.section_selected == 0 {
                                self.home_section = HomeSection::Trending;
                            } else {
                                self.section_selected = self.section_selected.saturating_sub(1);
                            }
                            return;
                        }
                        HomeSection::Favorite => {
                            if self.section_selected == 0 {
                                self.home_section = HomeSection::Trending;
                            } else {
                                self.section_selected = self.section_selected.saturating_sub(1);
                            }
                            return;
                        }
                    }
                }
                let columns = self.grid_columns.max(1);
                if self.selected >= columns {
                    self.selected -= columns;
                }
            }
            (Screen::Details | Screen::Episodes, KeyCode::Up | KeyCode::Char('k')) => {
                self.episode_selected = self.episode_selected.saturating_sub(1);
            }
            (Screen::Details | Screen::Episodes, KeyCode::Down | KeyCode::Char('j')) => {
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
