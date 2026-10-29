use crate::app::state::{AppState, HomeSection};
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const CARD_WIDTH: u16 = 26;
const CARD_HEIGHT: u16 = 10;
/// Width reserved on the right side of each card for episode count badges.
const BADGE_AREA_WIDTH: u16 = 7;

pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(CARD_HEIGHT),
            Constraint::Length(1),
        ])
        .split(frame.area());
    let query = app.state.query.clone();
    let searching = app.state.searching;
    let loading = app.state.loading;
    let error = app.state.error.clone();
    let results_empty = app.state.results.is_empty();
    let search = Paragraph::new(Line::from(vec![
        Span::styled(
            "Search  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if query.is_empty() && !searching {
                "press \"/\" to search..".to_string()
            } else {
                query
            },
            Style::default().fg(if searching {
                Color::White
            } else {
                Color::DarkGray
            }),
        ),
        Span::raw(if searching { "█" } else { "" }),
    ]));
    frame.render_widget(search, chunks[0]);
    frame.render_widget(
        Paragraph::new(app.state.section_title.as_str()).style(
            Style::default()
                .fg(Color::Rgb(245, 170, 215))
                .add_modifier(Modifier::BOLD),
        ),
        Rect {
            x: chunks[0].x,
            y: chunks[0].y + 2,
            width: chunks[0].width,
            height: 1,
        },
    );

    if loading && results_empty {
        draw_skeletons(frame, chunks[1]);
    } else if loading {
        frame.render_widget(Paragraph::new("Loading…"), chunks[1]);
    } else if let Some(error) = error {
        frame.render_widget(
            Paragraph::new(error.as_str()).wrap(Wrap { trim: true }),
            chunks[1],
        );
    } else if results_empty {
        frame.render_widget(Paragraph::new("Search for an anime to browse."), chunks[1]);
    } else if app.state.section_title == "Trending" && app.state.query.is_empty() {
        draw_home_dashboard(frame, app, chunks[1]);
    } else {
        draw_cards(frame, app, chunks[1]);
    }

    fn draw_home_dashboard(frame: &mut Frame<'_>, app: &mut App, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(11), Constraint::Min(1)])
            .split(area);
        draw_cards(frame, app, split[0]);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(split[1]);
        let airing = app.state.airing.clone();
        let popular = app.state.popular.clone();
        let favorite = app.state.favorite.clone();
        draw_compact_section(frame, app, columns[0], "Top Airing", airing);
        draw_compact_section(frame, app, columns[1], "Most Popular", popular);
        draw_compact_section(frame, app, columns[2], "Most Favorite", favorite);
    }

    fn draw_compact_section(
        frame: &mut Frame<'_>,
        app: &mut App,
        area: Rect,
        heading: &str,
        anime_list: Vec<crate::providers::anilist::Anime>,
    ) {
        let section = match heading {
            "Top Airing" => HomeSection::Airing,
            "Most Popular" => HomeSection::Popular,
            _ => HomeSection::Favorite,
        };
        let active = app.state.home_section == section;
        let available_height = area.height.saturating_sub(2);
        let visible_count = ((available_height / 5) as usize).max(1);

        let scroll = if active {
            if app.state.section_selected < app.state.compact_scroll {
                app.state.compact_scroll = app.state.section_selected;
            } else if app.state.section_selected >= app.state.compact_scroll + visible_count {
                app.state.compact_scroll = app.state.section_selected + 1 - visible_count;
            }
            app.state.compact_scroll
        } else {
            0
        };

        frame.render_widget(
            Paragraph::new(if active {
                format!("▶ {heading}")
            } else {
                heading.to_string()
            })
            .style(
                Style::default()
                    .fg(if active {
                        Color::White
                    } else {
                        Color::Rgb(245, 170, 215)
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Rect { height: 1, ..area },
        );
        for (visible_idx, anime) in anime_list.iter().skip(scroll).enumerate() {
            let index = scroll + visible_idx;
            let y = area.y + 2 + visible_idx as u16 * 5;
            if y + 4 > area.bottom() {
                break;
            }
            app.start_cover(anime);
            let selected = app.state.home_section == section && app.state.section_selected == index;
            frame.render_widget(
                Paragraph::new("").style(if selected {
                    Style::default().bg(Color::Rgb(35, 45, 75))
                } else {
                    Style::default()
                }),
                Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: 4,
                },
            );
            let image_area = Rect {
                x: area.x,
                y,
                width: 8.min(area.width),
                height: 4,
            };
            if let Some(protocol) = app.cover_protocol(anime, image_area.width, image_area.height) {
                frame.render_widget(ratatui_image::Image::new(&protocol), image_area);
            } else {
                frame.render_widget(Paragraph::new("░░░░░░░░"), image_area);
            }
            frame.render_widget(
                Paragraph::new(truncate(&anime.title, 22))
                    .wrap(Wrap { trim: true })
                    .style(if selected {
                        Style::default().fg(Color::Rgb(245, 170, 215))
                    } else {
                        Style::default()
                    }),
                Rect {
                    x: image_area.right() + 1,
                    y,
                    width: area.width.saturating_sub(9),
                    height: 2,
                },
            );
            frame.render_widget(
                Paragraph::new(format!(
                    "{}  ·  {}",
                    anime.status.to_lowercase(),
                    anime
                        .episodes
                        .map_or_else(|| "? eps".to_string(), |n| format!("{n} eps"))
                ))
                .style(Style::default().fg(if selected {
                    Color::White
                } else {
                    Color::Gray
                })),
                Rect {
                    x: image_area.right() + 1,
                    y: y + 2,
                    width: area.width.saturating_sub(9),
                    height: 1,
                },
            );
        }
    }

    frame.render_widget(
        Paragraph::new(if results_empty || searching {
            "Enter search   q quit"
        } else {
            "←/→/↑/↓ browse   Enter episodes   / search   q quit"
        }),
        chunks[2],
    );
}

fn draw_skeletons(frame: &mut Frame<'_>, area: Rect) {
    let columns = ((area.width / CARD_WIDTH).max(1)) as usize;
    let rows = ((area.height / CARD_HEIGHT).max(1)) as usize;
    let skeleton = Style::default().fg(Color::Rgb(45, 48, 65));
    for index in 0..columns * rows {
        let row = index / columns;
        let column = index % columns;
        let card = Rect {
            x: area.x + column as u16 * CARD_WIDTH,
            y: area.y + row as u16 * CARD_HEIGHT,
            width: CARD_WIDTH.saturating_sub(1),
            height: CARD_HEIGHT.saturating_sub(1),
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
                Line::styled("████████████████████████", skeleton),
            ]),
            card,
        );
    }
}

fn draw_cards(frame: &mut Frame<'_>, app: &mut App, area: Rect) {
    let columns = ((area.width / CARD_WIDTH).max(1)) as usize;
    app.state.grid_columns = columns;
    let rows = ((area.height / CARD_HEIGHT).max(1)) as usize;
    let max_row = app.state.results.len().saturating_sub(1) / columns;
    if app.state.selected < app.state.scroll_row * columns {
        app.state.scroll_row = app.state.selected / columns;
    } else if app.state.selected >= (app.state.scroll_row + rows) * columns {
        app.state.scroll_row = app.state.selected / columns + 1 - rows;
    }
    app.state.scroll_row = app.state.scroll_row.min(max_row);
    let first = app.state.scroll_row * columns;
    let last = (first + rows * columns).min(app.state.results.len());
    app.start_visible_covers(first, last);
    let selected_index = app.state.selected;
    for (position, anime) in app.state.results[first..last].to_vec().iter().enumerate() {
        let index = first + position;
        let row = position / columns;
        let column = position % columns;
        let card_area = Rect {
            x: area.x + column as u16 * CARD_WIDTH,
            y: area.y + row as u16 * CARD_HEIGHT,
            width: CARD_WIDTH.saturating_sub(1),
            height: CARD_HEIGHT.saturating_sub(1),
        };
        let selected = index == selected_index && app.state.home_section == HomeSection::Trending;
        let accent = [
            Color::Rgb(45, 50, 90),
            Color::Rgb(65, 45, 90),
            Color::Rgb(35, 75, 90),
            Color::Rgb(85, 55, 45),
        ][index % 4];
        let cover_area = Rect {
            x: card_area.x,
            y: card_area.y,
            width: card_area.width,
            height: 7,
        };
        frame.render_widget(
            Paragraph::new("").style(if selected {
                Style::default().bg(Color::Rgb(35, 45, 75))
            } else {
                Style::default()
            }),
            card_area,
        );
        // Define the badge area on the right side of the cover
        let badge_area = Rect {
            x: cover_area.x + cover_area.width,
            y: cover_area.y,
            width: BADGE_AREA_WIDTH,
            height: 7,
        };
        // Adjust cover width to leave space for badges
        let adjusted_cover_area = Rect {
            x: cover_area.x,
            y: cover_area.y,
            width: cover_area.width - BADGE_AREA_WIDTH,
            height: 7,
        };
        // Render cover image
        if let Some(protocol) =
            app.cover_protocol(anime, adjusted_cover_area.width, adjusted_cover_area.height)
        {
            frame.render_widget(ratatui_image::Image::new(&protocol), adjusted_cover_area);
        } else {
            frame.render_widget(
                Paragraph::new("cover unavailable").style(Style::default().fg(accent)),
                adjusted_cover_area,
            );
        }
        // Render badges
        let mut badge_spans = Vec::new();
        if let Some(sub) = anime.sub_episodes {
            badge_spans.push(Span::styled(
                format!(" CC {sub} "),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(180, 240, 190))
                    .add_modifier(Modifier::BOLD),
            ));
        }
        if let Some(dub) = anime.dub_episodes {
            badge_spans.push(Span::raw(" "));
            // Use simple "mic" glyph instead of the emoji
            badge_spans.push(Span::styled(
                format!(" mic {dub} "),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(190, 225, 250))
                    .add_modifier(Modifier::BOLD),
            ));
        }
        // Fallback when no sub/dub info
        if badge_spans.is_empty() {
            if let Some(episodes) = anime.episodes {
                badge_spans.push(Span::styled(
                    format!(" {episodes} eps "),
                    Style::default().fg(Color::Gray),
                ));
            }
        }
        let badge_line = Line::from(badge_spans);
        frame.render_widget(
            Paragraph::new(badge_line).alignment(ratatui::layout::Alignment::Center),
            badge_area,
        );

        frame.render_widget(
            Paragraph::new(anime.title.clone())
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::White).add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                })),
            Rect {
                x: card_area.x,
                y: card_area.y + 7,
                width: card_area.width,
                height: 2,
            },
        );
        let mut meta_spans = vec![Span::styled(
            anime.status.to_lowercase(),
            Style::default().fg(Color::Gray),
        )];
        meta_spans.push(Span::raw(" "));
        meta_spans.push(Span::styled(
            anime
                .episodes
                .map_or_else(|| "? eps".to_string(), |n| format!("{n} eps")),
            Style::default().fg(Color::Gray),
        ));

        let metadata = Line::from(meta_spans);
        frame.render_widget(
            Paragraph::new(metadata).style(if selected {
                Style::default().bg(Color::Rgb(35, 45, 75))
            } else {
                Style::default()
            }),
            Rect {
                x: card_area.x,
                y: card_area.y + 9,
                width: card_area.width,
                height: 1,
            },
        );
    }
}

fn truncate(value: &str, width: usize) -> String {
    let mut result = value.chars().take(width).collect::<String>();
    if value.chars().count() > width {
        result.pop();
        result.push('…');
    }
    format!("{result:<width$}")
}

pub fn draw_episodes(frame: &mut Frame<'_>, state: &AppState) {
    let title = state
        .selected_anime
        .as_ref()
        .map_or("Episodes", |anime| anime.title.as_str());
    let items = state
        .episodes
        .iter()
        .map(|episode| ListItem::new(format!("Episode {}", episode.number)));
    let list = List::new(items).highlight_style(Style::default().bg(Color::Blue));
    let mut selected = ListState::default();
    selected.select(Some(state.episode_selected));
    let area = frame.area();
    frame.render_stateful_widget(list, area, &mut selected);
    frame.render_widget(
        Paragraph::new(format!(
            "{title}   ↑/↓ select   Enter play   d {}   Esc quit",
            if state.dubbed { "dub" } else { "sub" }
        )),
        Rect {
            x: 0,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        },
    );
}

pub fn draw_details(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();
    let details = app.state.details.clone().unwrap_or_default();
    let cover_area = Rect {
        x: 2,
        y: 1,
        width: 22,
        height: 14,
    };
    if let Some(anime) = app.state.selected_anime.clone() {
        if let Some(protocol) = app.cover_protocol(&anime, cover_area.width, cover_area.height) {
            frame.render_widget(ratatui_image::Image::new(&protocol), cover_area);
        }
    }

    frame.render_widget(
        Paragraph::new(details.title).style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Rect {
            x: 27,
            y: 1,
            width: area.width.saturating_sub(29),
            height: 2,
        },
    );
    let description = format!(
        "{}\n\n{} · {} · {} episodes\nScore: {}  ·  {} min\n{}\n{}\n{}",
        details.native_title,
        details.format,
        details.status.to_lowercase(),
        details
            .episodes
            .map_or_else(|| "?".to_string(), |n| n.to_string()),
        details
            .score
            .map_or_else(|| "?".to_string(), |n| n.to_string()),
        details
            .duration
            .map_or_else(|| "?".to_string(), |n| n.to_string()),
        details.description,
        if details.genres.is_empty() {
            "Genres: —".to_string()
        } else {
            format!("Genres: {}", details.genres.join("  ·  "))
        },
        if details.studios.is_empty() {
            "Studios: —".to_string()
        } else {
            format!("Studios: {}", details.studios.join(", "))
        },
    );
    frame.render_widget(
        Paragraph::new(description).wrap(Wrap { trim: true }),
        Rect {
            x: 27,
            y: 3,
            width: area.width.saturating_sub(29),
            height: 12,
        },
    );

    frame.render_widget(
        Paragraph::new("Episodes").style(
            Style::default()
                .fg(Color::Rgb(245, 170, 215))
                .add_modifier(Modifier::BOLD),
        ),
        Rect {
            x: 2,
            y: 17,
            width: area.width.saturating_sub(4),
            height: 1,
        },
    );
    let episode_area = Rect {
        x: 2,
        y: 19,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(21),
    };
    if app.state.episode_loading {
        frame.render_widget(Paragraph::new("Loading episodes…"), episode_area);
    } else if app.state.episodes.is_empty() {
        frame.render_widget(
            Paragraph::new(
                app.state
                    .episode_error
                    .as_deref()
                    .unwrap_or("No playable episodes were found."),
            )
            .wrap(Wrap { trim: true }),
            episode_area,
        );
    } else {
        let items = app
            .state
            .episodes
            .iter()
            .enumerate()
            .map(|(index, episode)| {
                let status = if app.state.now_playing == Some(index) {
                    "  Now playing..."
                } else {
                    ""
                };
                ListItem::new(format!("Episode {}{status}", episode.number))
            });
        let list = List::new(items).highlight_symbol("▶ ").highlight_style(
            Style::default()
                .fg(Color::Rgb(245, 170, 215))
                .add_modifier(Modifier::BOLD),
        );
        let mut selected = ListState::default();
        selected.select(Some(
            app.state
                .episode_selected
                .min(app.state.episodes.len().saturating_sub(1)),
        ));
        frame.render_stateful_widget(list, episode_area, &mut selected);
    }
    frame.render_widget(
        Paragraph::new("↑/↓ choose episode   Enter play   d sub/dub   Esc home")
            .style(Style::default().fg(Color::Gray)),
        Rect {
            x: 2,
            y: area.bottom().saturating_sub(1),
            width: area.width.saturating_sub(4),
            height: 1,
        },
    );
}
