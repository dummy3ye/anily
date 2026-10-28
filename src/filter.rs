use crate::genre;
use crate::providers::anilist::Anime;

/// Checks if any word token in the given title matches an excluded term.
/// Words are split on non-alphanumeric characters to prevent false positives
/// (e.g., "country" won't match "ntr").
pub fn title_contains_excluded(title: &str, exclude_list: &[String]) -> bool {
    if exclude_list.is_empty() || title.trim().is_empty() {
        return false;
    }

    let words = title.split(|c: char| !c.is_alphanumeric());

    for word in words {
        if word.is_empty() {
            continue;
        }
        for excluded in exclude_list {
            if genre::matches_genre_or_tag(word, excluded) {
                return true;
            }
        }
    }
    false
}

/// Determines if an anime should be excluded based on its genres/tags OR its title.
pub fn should_exclude_anime(anime: &Anime, exclude_list: &[String]) -> bool {
    if exclude_list.is_empty() {
        return false;
    }

    if genre::should_exclude(&anime.genres, exclude_list) {
        return true;
    }

    if title_contains_excluded(&anime.title, exclude_list) {
        return true;
    }

    false
}

/// Filters a list of anime based on both genres/tags and title words.
pub fn filter_anime(items: Vec<Anime>, exclude_list: &[String]) -> Vec<Anime> {
    if exclude_list.is_empty() {
        return items;
    }

    items
        .into_iter()
        .filter(|anime| !should_exclude_anime(anime, exclude_list))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_contains_excluded_word() {
        let exclude = vec!["Ecchi".to_string(), "ntr".to_string()];

        assert!(title_contains_excluded("Ecchi Everyday", &exclude));
        assert!(title_contains_excluded("My Life: NTR Edition", &exclude));
        assert!(title_contains_excluded("A Netorare Story", &exclude));
        assert!(!title_contains_excluded("Normal Romance", &exclude));
    }

    #[test]
    fn test_title_no_false_positives_for_substrings() {
        let exclude = vec!["ntr".to_string()];

        // "country", "entry", "intro" contain n-t-r consecutively or similarly
        assert!(!title_contains_excluded(
            "Country Life in Another World",
            &exclude
        ));
        assert!(!title_contains_excluded("Entry Level Hero", &exclude));
        assert!(!title_contains_excluded("Introduction to Magic", &exclude));
    }

    #[test]
    fn test_should_exclude_anime_either_genre_or_title() {
        let exclude = vec!["Ecchi".to_string()];

        let anime_by_genre = Anime {
            id: 1,
            title: "Safe Title".to_string(),
            cover: String::new(),
            cover_large: None,
            episodes: None,
            status: "RELEASING".to_string(),
            genres: vec!["Ecchi".to_string()],
            sub_episodes: None,
            dub_episodes: None,
            cover_art: None,
        };

        let anime_by_title = Anime {
            id: 2,
            title: "Super Ecchi Adventure".to_string(),
            cover: String::new(),
            cover_large: None,
            episodes: None,
            status: "RELEASING".to_string(),
            genres: vec!["Comedy".to_string()],
            sub_episodes: None,
            dub_episodes: None,
            cover_art: None,
        };

        let anime_clean = Anime {
            id: 3,
            title: "Clean Anime".to_string(),
            cover: String::new(),
            cover_large: None,
            episodes: None,
            status: "RELEASING".to_string(),
            genres: vec!["Comedy".to_string(), "Action".to_string()],
            sub_episodes: None,
            dub_episodes: None,
            cover_art: None,
        };

        assert!(should_exclude_anime(&anime_by_genre, &exclude));
        assert!(should_exclude_anime(&anime_by_title, &exclude));
        assert!(!should_exclude_anime(&anime_clean, &exclude));
    }
}
