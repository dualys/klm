use crate::{night_review, reflection_log};
use klm::{Kognitara, current_timestamp};
use serde::Serialize;
use serde_json::{Value, json};
use std::path::Path;

const PAGE_SIZE: usize = 50;

struct Query {
    page: usize,
    text: String,
}

fn decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err("Invalid URL encoding".into());
                }
                if !bytes[i + 1].is_ascii_hexdigit() || !bytes[i + 2].is_ascii_hexdigit() {
                    return Err("Invalid URL encoding".into());
                }
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .map_err(|_| "Invalid URL encoding")?;
                result.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid URL encoding")?);
                i += 3;
            }
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            byte => {
                result.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in query".into())
}

fn parse_query(raw: &str) -> Result<Query, String> {
    if raw.len() > 2048 {
        return Err("Search query is too long".into());
    }
    let mut page = None;
    let mut text = None;
    for pair in raw.split('&').filter(|part| !part.is_empty()) {
        let (key, value) = pair.split_once('=').ok_or("Invalid query parameter")?;
        match key {
            "page" if page.is_none() => {
                let number: usize = value.parse().map_err(|_| "Invalid page number")?;
                if number == 0 {
                    return Err("Page number must be positive".into());
                }
                page = Some(number);
            }
            "q" if text.is_none() => {
                let value = decode(value)?;
                if value.chars().count() > 200 || value.chars().any(char::is_control) {
                    return Err("Search text must contain at most 200 printable characters".into());
                }
                text = Some(value.trim().to_lowercase());
            }
            _ => return Err("Unknown or duplicate query parameter".into()),
        }
    }
    Ok(Query {
        page: page.unwrap_or(1),
        text: text.unwrap_or_default(),
    })
}

fn matches(text: &str, query: &str) -> bool {
    query.is_empty() || text.to_lowercase().contains(query)
}

fn paginated<T: Serialize>(items: &[T], requested: usize) -> Value {
    let total = items.len();
    let pages = total.div_ceil(PAGE_SIZE).max(1);
    let page = requested.min(pages);
    let start = (page - 1) * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(total);
    json!({ "page": page, "pages": pages, "total": total, "items": &items[start..end] })
}

#[derive(Serialize)]
struct HypothesisItem<'a> {
    hypothesis: &'a klm::Hypothesis,
    examinations: Option<u64>,
    examined_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value")]
enum NightItem<'a> {
    Hypothesis(&'a night_review::HypothesisReview),
    Concept(&'a night_review::ConceptReview),
}

pub(crate) fn page(
    mind: &Kognitara,
    directory: &Path,
    path: &str,
    raw_query: &str,
) -> Result<Value, (u16, String)> {
    let query = parse_query(raw_query).map_err(|error| (400, error))?;
    let night = if path != "/api/reflections" {
        night_review::read(directory).map_err(|error| (503, error))?
    } else {
        None
    };
    match path {
        "/api/hypotheses" => {
            let items: Vec<_> = mind
                .hypotheses
                .hypotheses
                .iter()
                .filter(|hypothesis| {
                    matches(&hypothesis.statement, &query.text)
                        || hypothesis
                            .evidence
                            .iter()
                            .filter(|e| e.revision == hypothesis.revision())
                            .any(|e| {
                                matches(&e.source, &query.text) || matches(&e.note, &query.text)
                            })
                })
                .map(|hypothesis| {
                    let review = night
                        .as_ref()
                        .and_then(|state| {
                            state.hypotheses.get(&format!(
                                "{}:{}",
                                hypothesis.id,
                                hypothesis.revision()
                            ))
                        })
                        .filter(|review| review.statement == hypothesis.statement);
                    HypothesisItem {
                        hypothesis,
                        examinations: review.map(|review| review.examinations),
                        examined_at: review.map(|review| review.examined_at),
                    }
                })
                .collect();
            Ok(paginated(&items, query.page))
        }
        "/api/reflections" => {
            let mut items = reflection_log::recent(mind, current_timestamp());
            items.reverse();
            items.retain(|entry| {
                matches(&entry.reflection.maieutic_question, &query.text)
                    || matches(&entry.reflection.interior_monologue, &query.text)
                    || matches(&entry.reflection.examined_hypothesis, &query.text)
            });
            Ok(paginated(&items, query.page))
        }
        "/api/night" => {
            let mut items = Vec::new();
            if let Some(state) = &night {
                let mut reviews: Vec<_> = state.hypotheses.values().collect();
                reviews.sort_by(|a, b| b.examined_at.cmp(&a.examined_at));
                for review in reviews {
                    if matches(&review.statement, &query.text)
                        || review
                            .questions
                            .iter()
                            .chain(&review.contradictions)
                            .any(|s| matches(s, &query.text))
                        || review.wikipedia.as_ref().is_some_and(|wiki| {
                            matches(&wiki.source, &query.text)
                                || matches(&wiki.excerpt, &query.text)
                        })
                    {
                        items.push(NightItem::Hypothesis(review));
                    }
                }
                for concept in &state.concepts {
                    if matches(&concept.label, &query.text)
                        || matches(&concept.question, &query.text)
                        || concept
                            .contradictions
                            .iter()
                            .any(|s| matches(s, &query.text))
                    {
                        items.push(NightItem::Concept(concept));
                    }
                }
            }
            Ok(paginated(&items, query.page))
        }
        _ => Err((404, "Not found".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_validates_encoding_and_page() {
        let query = parse_query("page=2&q=For%C3%AAt+primaire").unwrap();
        assert_eq!(query.page, 2);
        assert_eq!(query.text, "forêt primaire");
        for invalid in [
            "page=0",
            "page=-1",
            "q=%ZZ",
            "q=%+1",
            "q=%FF",
            "page=1&page=2",
        ] {
            assert!(parse_query(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn results_are_bounded_and_page_is_clamped() {
        let items: Vec<_> = (0..121).collect();
        let first = paginated(&items, 1);
        let last = paginated(&items, 99);
        assert_eq!(first["items"].as_array().unwrap().len(), 50);
        assert_eq!(last["page"], 3);
        assert_eq!(last["items"].as_array().unwrap().len(), 21);
        assert_eq!(paginated::<u8>(&[], 2)["page"], 1);
    }
}
