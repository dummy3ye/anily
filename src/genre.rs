use crate::providers::anilist::Anime;

/// Expands an excluded genre or tag to include known aliases (e.g., "ntr" -> "NTR", "Netorare").
pub fn expand_tag_aliases(name: &str) -> Vec<String> {
    let lower = name.trim().to_lowercase();
    let mut list = vec![name.trim().to_string()];
    if lower == "ntr" {
        list.push("Netorare".to_string());
    } else if lower == "netorare" {
        list.push("NTR".to_string());
    }
    list
}

/// Determines if a genre or tag string matches an exclusion pattern.
/// Matches are case-insensitive and handle aliases (e.g. "ntr" <-> "netorare").
pub fn matches_genre_or_tag(target: &str, pattern: &str) -> bool {
    let t = target.trim().to_lowercase();
    let p = pattern.trim().to_lowercase();

    if t.is_empty() || p.is_empty() {
        return false;
    }

    if t == p {
        return true;
    }

    // Common aliases
    if (p == "ntr" && t == "netorare") || (p == "netorare" && t == "ntr") {
        return true;
    }

    false
}

/// Checks whether an anime's genre/tag list matches any pattern in `exclude_list`.
pub fn should_exclude(genres_and_tags: &[String], exclude_list: &[String]) -> bool {
    if exclude_list.is_empty() || genres_and_tags.is_empty() {
        return false;
    }

    genres_and_tags.iter().any(|item| {
        exclude_list
            .iter()
            .any(|excluded| matches_genre_or_tag(item, excluded))
    })
}

/// Filters a list of `Anime` items, removing any that match the `exclude_list`.
pub fn filter_anime(items: Vec<Anime>, exclude_list: &[String]) -> Vec<Anime> {
    if exclude_list.is_empty() {
        return items;
    }

    items
        .into_iter()
        .filter(|anime| !should_exclude(&anime.genres, exclude_list))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_matching() {
        assert!(matches_genre_or_tag("Ecchi", "ecchi"));
        assert!(matches_genre_or_tag("HENTAI", "hentai"));
        assert!(!matches_genre_or_tag("Action", "Romance"));
    }

    #[test]
    fn test_alias_ntr_netorare() {
        assert!(matches_genre_or_tag("Netorare", "ntr"));
        assert!(matches_genre_or_tag("NTR", "netorare"));
        assert!(matches_genre_or_tag("ntr", "NTR"));
    }

    #[test]
    fn test_should_exclude() {
        let exclude = vec!["Ecchi".to_string(), "ntr".to_string()];
        let tags_safe = vec!["Action".to_string(), "Adventure".to_string()];
        let tags_ecchi = vec!["Action".to_string(), "ecchi".to_string()];
        let tags_netorare = vec!["Drama".to_string(), "Netorare".to_string()];

        assert!(!should_exclude(&tags_safe, &exclude));
        assert!(should_exclude(&tags_ecchi, &exclude));
        assert!(should_exclude(&tags_netorare, &exclude));
    }

    #[test]
    fn test_empty_exclude_list() {
        let exclude: Vec<String> = vec![];
        let tags = vec!["Ecchi".to_string(), "Hentai".to_string()];
        assert!(!should_exclude(&tags, &exclude));
    }
}
