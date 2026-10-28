use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Anime {
    pub id: u32,
    pub title: String,
    pub cover: String,
    pub cover_large: Option<String>,
    pub episodes: Option<u32>,
    pub status: String,
    pub genres: Vec<String>,
    pub sub_episodes: Option<u32>,
    pub dub_episodes: Option<u32>,
    pub cover_art: Option<CoverArt>,
}

#[derive(Debug)]
pub struct HomeFeed {
    pub trending: Vec<Anime>,
    pub airing: Vec<Anime>,
    pub popular: Vec<Anime>,
    pub favorite: Vec<Anime>,
}

#[derive(Debug, Clone, Default)]
pub struct AnimeDetails {
    pub title: String,
    pub native_title: String,
    pub description: String,
    pub genres: Vec<String>,
    pub studios: Vec<String>,
    pub producers: Vec<String>,
    pub format: String,
    pub status: String,
    pub score: Option<u32>,
    pub duration: Option<u32>,
    pub episodes: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CoverArt {
    pub image: image::DynamicImage,
}

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "Page")]
    page: Option<Page>,
    trending: Option<Page>,
    airing: Option<Page>,
    popular: Option<Page>,
    favorite: Option<Page>,
}

#[derive(Deserialize)]
struct Page {
    media: Vec<Media>,
}

#[derive(Deserialize)]
struct Media {
    id: u32,
    title: Titles,
    #[serde(rename = "coverImage")]
    cover_image: Cover,
    episodes: Option<u32>,
    status: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Deserialize)]
struct Tag {
    name: String,
}

#[derive(Deserialize)]
struct Titles {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Deserialize)]
struct Cover {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
    large: Option<String>,
}

pub async fn search(client: &Client, query: &str) -> Result<Vec<Anime>> {
    let response = client
        .post("https://graphql.anilist.co")
        .json(&serde_json::json!({
            "query": "query ($search: String!) { Page(perPage: 24) { media(search: $search, type: ANIME, sort: SEARCH_MATCH) { id title { romaji english native } coverImage { extraLarge large } episodes status genres tags { name } } } }",
            "variables": { "search": query }
        }))
        .send()
        .await
        .context("requesting AniList search")?
        .error_for_status()
        .context("AniList search returned an error")?
        .json()
        .await
        .context("decoding AniList response")?;

    media_to_anime(response).await
}

pub async fn trending(client: &Client) -> Result<Vec<Anime>> {
    let response = client
        .post("https://graphql.anilist.co")
        .json(&serde_json::json!({
            "query": "query { Page(perPage: 24) { media(type: ANIME, sort: TRENDING_DESC) { id title { romaji english native } coverImage { extraLarge large } episodes status genres tags { name } } } }"
        }))
        .send()
        .await
        .context("requesting AniList trending anime")?
        .error_for_status()
        .context("AniList trending request returned an error")?
        .json()
        .await
        .context("decoding AniList trending response")?;

    media_to_anime(response).await
}

pub async fn home_feed(client: &Client, exclude_list: &[String]) -> Result<HomeFeed> {
    let mut tags_not_in: Vec<String> = Vec::new();
    let mut genres_not_in: Vec<String> = Vec::new();

    for item in exclude_list {
        for alias in crate::genre::expand_tag_aliases(item) {
            tags_not_in.push(alias.clone());
            genres_not_in.push(alias);
        }
    }

    let query = if exclude_list.is_empty() {
        "query { \
            trending: Page(perPage: 16) { media(type: ANIME, sort: TRENDING_DESC) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            airing: Page(perPage: 10) { media(type: ANIME, status: RELEASING, sort: UPDATED_AT_DESC) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            popular: Page(perPage: 10) { media(type: ANIME, sort: POPULARITY_DESC) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            favorite: Page(perPage: 10) { media(type: ANIME, sort: FAVOURITES_DESC) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
        }".to_string()
    } else {
        "query ($genres: [String], $tags: [String]) { \
            trending: Page(perPage: 16) { media(type: ANIME, sort: TRENDING_DESC, genre_not_in: $genres, tag_not_in: $tags) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            airing: Page(perPage: 10) { media(type: ANIME, status: RELEASING, sort: UPDATED_AT_DESC, genre_not_in: $genres, tag_not_in: $tags) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            popular: Page(perPage: 10) { media(type: ANIME, sort: POPULARITY_DESC, genre_not_in: $genres, tag_not_in: $tags) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
            favorite: Page(perPage: 10) { media(type: ANIME, sort: FAVOURITES_DESC, genre_not_in: $genres, tag_not_in: $tags) { id title { romaji english native } coverImage { extraLarge large } episodes status genres } } \
        }".to_string()
    };

    let variables = serde_json::json!({
        "genres": genres_not_in,
        "tags": tags_not_in,
    });

    let response: Response = client
        .post("https://graphql.anilist.co")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables,
        }))
        .send()
        .await
        .context("requesting AniList home feed")?
        .error_for_status()
        .context("AniList home feed returned an error")?
        .json()
        .await
        .context("decoding AniList home feed")?;

    let counts = crate::providers::hianime::scrape_home_counts(client).await;
    let attach_counts = |mut list: Vec<Anime>| -> Vec<Anime> {
        for anime in list.iter_mut() {
            let norm = crate::providers::hianime::normalize_title(&anime.title);
            if let Some(c) = counts.get(&norm) {
                anime.sub_episodes = c.sub;
                anime.dub_episodes = c.dub;
            }
        }
        list
    };

    Ok(HomeFeed {
        trending: attach_counts(page_to_anime(
            response.data.trending.context("missing trending feed")?,
        )),
        airing: attach_counts(page_to_anime(
            response.data.airing.context("missing airing feed")?,
        )),
        popular: attach_counts(page_to_anime(
            response.data.popular.context("missing popular feed")?,
        )),
        favorite: attach_counts(page_to_anime(
            response.data.favorite.context("missing favorite feed")?,
        )),
    })
}

async fn media_to_anime(response: Response) -> Result<Vec<Anime>> {
    Ok(page_to_anime(
        response.data.page.context("missing search page")?,
    ))
}

fn page_to_anime(page: Page) -> Vec<Anime> {
    page.media
        .into_iter()
        .map(|media| {
            let mut genres = media.genres;
            for tag in media.tags {
                if !genres.iter().any(|g| g.eq_ignore_ascii_case(&tag.name)) {
                    genres.push(tag.name);
                }
            }

            Anime {
                id: media.id,
                title: media
                    .title
                    .english
                    .or(media.title.romaji)
                    .or(media.title.native)
                    .unwrap_or_else(|| "Untitled".to_string()),
                cover: media.cover_image.extra_large.unwrap_or_default(),
                cover_large: media.cover_image.large,
                episodes: media.episodes,
                status: media.status.unwrap_or_else(|| "UNKNOWN".to_string()),
                genres,
                sub_episodes: None,
                dub_episodes: None,
                cover_art: None,
            }
        })
        .collect()
}

pub async fn fetch_cover(client: &Client, url: &str) -> Option<CoverArt> {
    let bytes = client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .bytes()
        .await
        .ok()?;
    Some(CoverArt {
        image: image::load_from_memory(&bytes).ok()?,
    })
}

pub async fn details(client: &Client, id: u32) -> Result<AnimeDetails> {
    #[derive(Deserialize)]
    struct DetailResponse {
        data: DetailData,
    }
    #[derive(Deserialize)]
    struct DetailData {
        #[serde(rename = "Media")]
        media: DetailMedia,
    }
    #[derive(Deserialize)]
    struct DetailMedia {
        title: Titles,
        description: Option<String>,
        genres: Vec<String>,
        studios: NamedCollection,
        format: Option<String>,
        status: Option<String>,
        #[serde(rename = "averageScore")]
        average_score: Option<u32>,
        duration: Option<u32>,
        episodes: Option<u32>,
    }
    #[derive(Deserialize)]
    struct NamedCollection {
        nodes: Vec<Named>,
    }
    #[derive(Deserialize)]
    struct Named {
        name: String,
    }

    let response: DetailResponse = client
        .post("https://graphql.anilist.co")
        .json(&serde_json::json!({
            "query": "query ($id: Int!) { Media(id: $id, type: ANIME) { title { romaji english native } description genres studios { nodes { name } } format status averageScore duration episodes } }",
            "variables": { "id": id }
        }))
        .send()
        .await
        .context("requesting AniList anime details")?
        .error_for_status()
        .context("AniList details returned an error")?
        .json()
        .await
        .context("decoding AniList anime details")?;
    let media = response.data.media;
    Ok(AnimeDetails {
        title: media
            .title
            .english
            .or(media.title.romaji)
            .unwrap_or_default(),
        native_title: media.title.native.unwrap_or_default(),
        description: media.description.unwrap_or_default().replace("<br>", " "),
        genres: media.genres,
        studios: media
            .studios
            .nodes
            .into_iter()
            .map(|node| node.name)
            .collect(),
        producers: Vec::new(),
        format: media.format.unwrap_or_else(|| "Unknown".to_string()),
        status: media.status.unwrap_or_else(|| "Unknown".to_string()),
        score: media.average_score,
        duration: media.duration,
        episodes: media.episodes,
    })
}

pub async fn title(client: &Client, id: u32) -> Result<Anime> {
    search(client, &id.to_string())
        .await?
        .into_iter()
        .find(|anime| anime.id == id)
        .ok_or_else(|| anyhow!("AniList anime {id} was not found"))
}
