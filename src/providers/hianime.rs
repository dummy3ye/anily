use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use scraper::{Html, Selector};

const BASE: &str = "https://hianime.at";

#[derive(Debug, Clone)]
pub struct Match {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub number: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Stream {
    pub video: String,
    pub audio: Option<String>,
    pub subtitles: Option<String>,
    pub referer: String,
}

#[derive(Debug, Clone, Default)]
pub struct EpisodeCounts {
    pub sub: Option<u32>,
    pub dub: Option<u32>,
}

pub fn normalize_title(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn scrape_home_counts(
    client: &Client,
) -> std::collections::HashMap<String, EpisodeCounts> {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let Ok(resp) = client.get(format!("{BASE}/home")).send().await else {
        return map;
    };
    let Ok(html) = resp.text().await else {
        return map;
    };

    let document = Html::parse_document(&html);
    let item_sel = Selector::parse(".film-detail, .flw-item, li").expect("valid selector");
    let name_sel =
        Selector::parse(".film-name, .film-title, .dynamic-name").expect("valid selector");
    let sub_sel = Selector::parse(".tick-sub").expect("valid selector");
    let dub_sel = Selector::parse(".tick-dub").expect("valid selector");

    for item in document.select(&item_sel) {
        if let Some(name_el) = item.select(&name_sel).next() {
            let name = name_el.text().collect::<String>().trim().to_string();
            let sub = item
                .select(&sub_sel)
                .next()
                .and_then(|el| el.text().collect::<String>().trim().parse::<u32>().ok());
            let dub = item
                .select(&dub_sel)
                .next()
                .and_then(|el| el.text().collect::<String>().trim().parse::<u32>().ok());

            if sub.is_some() || dub.is_some() {
                let norm = normalize_title(&name);
                if !norm.is_empty() {
                    map.entry(norm).or_insert(EpisodeCounts { sub, dub });
                }
            }
        }
    }

    map
}

pub async fn search(client: &Client, query: &str) -> Result<Vec<Match>> {
    let html = client
        .get(format!("{BASE}/search"))
        .query(&[("keyword", query)])
        .send()
        .await
        .context("requesting HiAnime search")?
        .error_for_status()
        .context("HiAnime search returned an error")?
        .text()
        .await?;
    let document = Html::parse_document(&html);
    let item = Selector::parse("div.film-detail").expect("valid selector");
    let link = Selector::parse("h3.film-name a").expect("valid selector");
    Ok(document
        .select(&item)
        .filter_map(|detail| {
            let anchor = detail.select(&link).next()?;
            let href = anchor.value().attr("href")?;
            let id = href.rsplit('/').next()?.to_string();
            Some(Match {
                id,
                title: anchor
                    .value()
                    .attr("title")
                    .unwrap_or("Untitled")
                    .to_string(),
            })
        })
        .collect())
}

pub async fn episodes(client: &Client, anime_id: &str) -> Result<Vec<Episode>> {
    let anime_id = anime_id
        .rsplit('-')
        .next()
        .filter(|value| value.chars().all(|character| character.is_ascii_digit()))
        .unwrap_or(anime_id);
    let response = client
        .get(format!("{BASE}/api/theme/episode/list/{anime_id}"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let html = response
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|json| {
            json.get("html")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or(response);
    let document = Html::parse_document(&html);
    let item = Selector::parse(".ep-item").expect("valid selector");
    Ok(document
        .select(&item)
        .filter_map(|node| {
            Some(Episode {
                number: node.value().attr("data-number")?.to_string(),
                id: node.value().attr("data-id")?.to_string(),
            })
        })
        .collect())
}

pub async fn stream(client: &Client, episode: &Episode, dubbed: bool) -> Result<Stream> {
    let response = client
        .get(format!("{BASE}/api/theme/episode/servers"))
        .query(&[("episodeId", episode.id.as_str())])
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let html = response
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|json| {
            json.get("html")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or(response);
    let document = Html::parse_document(&html);
    let selector = Selector::parse(".server-item").expect("valid selector");
    let kind = if dubbed { "dub" } else { "sub" };
    let server = document
        .select(&selector)
        .find(|node| {
            node.value().attr("data-type") == Some(kind)
                && node.value().attr("data-server-name") == Some("ZokoAnime")
        })
        .ok_or_else(|| anyhow!("no {kind} stream server is available"))?;
    let encoded = server
        .value()
        .attr("data-hash")
        .ok_or_else(|| anyhow!("stream server returned no embed hash"))?;
    let embed = String::from_utf8(STANDARD.decode(encoded)?)?;
    let referer = format!(
        "{}/",
        embed.split('/').take(3).collect::<Vec<_>>().join("/")
    );
    let page = client
        .get(&embed)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let blob = page
        .split("window.__P=\"")
        .nth(1)
        .and_then(|value| value.split('"').next())
        .ok_or_else(|| anyhow!("stream embed returned no configuration"))?;
    let json = xor_blob(blob)?;
    let video = json
        .split("\"src\":\"")
        .nth(1)
        .and_then(|value| value.split('"').next())
        .ok_or_else(|| anyhow!("stream configuration returned no video"))?
        .replace("\\/", "/");
    let subtitles = json
        .split("\"subtitles\":[")
        .nth(1)
        .and_then(|value| {
            value
                .split("},{")
                .find(|entry| entry.contains("\"default\":true"))
        })
        .and_then(|entry| entry.split("\"src\":\"").nth(1))
        .and_then(|value| value.split('"').next())
        .map(|value| value.replace("\\/", "/"));
    Ok(Stream {
        video,
        audio: None,
        subtitles,
        referer,
    })
}

fn xor_blob(blob: &str) -> Result<String> {
    let bytes = STANDARD.decode(blob)?;
    let key = b"otaku-embed-v1";
    Ok(bytes
        .into_iter()
        .enumerate()
        .map(|(index, byte)| (byte ^ key[index % key.len()]) as char)
        .collect())
}
