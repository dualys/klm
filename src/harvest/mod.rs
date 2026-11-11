extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::mind::learning::OrganicLearner;
use crate::mind::state::CognitiveState;
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// Verdict returned by the Sovereign Filter when inspecting a browser URL, cookie domain, or search trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterVerdict {
    /// Domain is explicitly trusted in the Family Sanctuary Whitelist
    AllowedByWhitelist { matched_rule: String },
    /// Content matches a trusted research topic and has no blacklist violation
    AllowedByTopic { matched_topic: String },
    /// Blocked because the domain or URL matches the anti-tracker / anti-noise Blacklist
    BlockedByBlacklist { matched_rule: String },
    /// Blocked to protect sensitive family privacy (banking, credentials, private messages)
    BlockedSensitivePrivacy { reason: String },
    /// Deferred: unknown domain not yet approved by the family whitelist
    UnverifiedDomain { domain: String },
}

impl FilterVerdict {
    /// Returns `true` if the trace is permitted to enter Kōgnitara's cognitive harvesting pipeline.
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            FilterVerdict::AllowedByWhitelist { .. } | FilterVerdict::AllowedByTopic { .. }
        )
    }
}

/// Sovereign Whitelist & Blacklist filter protecting Kōgnitara from web pollution, trackers,
/// advertising noise, and private credential leaks while cultivating trusted knowledge gardens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignFilter {
    /// Trusted domains approved for knowledge harvesting
    pub whitelisted_domains: BTreeSet<String>,
    /// Trusted research topic keywords approved by the family
    pub whitelisted_topics: BTreeSet<String>,
    /// Blocked domains (ad networks, trackers, algorithmic noise, disinformation)
    pub blacklisted_domains: BTreeSet<String>,
    /// Sensitive privacy patterns never harvested (banking, auth tokens, private mail)
    pub sensitive_privacy_keywords: BTreeSet<String>,
    /// If true, only explicitly whitelisted domains or topics are harvested
    pub strict_whitelist_mode: bool,
}

impl Default for SovereignFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereignFilter {
    /// Initializes the Sovereign Filter with foundational Whitelist and Blacklist rules.
    pub fn new() -> Self {
        let mut whitelisted_domains = BTreeSet::new();
        for domain in [
            "amentys.local",
            "sanctuary.local",
            "wikipedia.org",
            "wiktionary.org",
            "arxiv.org",
            "docs.rs",
            "crates.io",
            "rust-lang.org",
            "kernel.org",
            "botany.org",
            "flora.local",
            "encyclopedia.local",
            "openhardware.org",
            "rocm.docs.amd.com",
            "vulkan.org",
        ] {
            whitelisted_domains.insert(domain.to_string());
        }

        let mut whitelisted_topics = BTreeSet::new();
        for topic in [
            "tree",
            "forest",
            "earth",
            "botany",
            "mycelium",
            "water",
            "ecology",
            "agriculture",
            "medicine",
            "mathematics",
            "physics",
            "philosophy",
            "socrates",
            "rust",
            "amentys",
            "amd",
            "vulkan",
            "rocm",
            "kpack",
            "jinshu",
            "cubefs",
        ] {
            whitelisted_topics.insert(topic.to_string());
        }

        let mut blacklisted_domains = BTreeSet::new();
        for bad in [
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
            "facebook.com",
            "instagram.com",
            "tiktok.com",
            "analytics",
            "tracker",
            "telemetry",
            "adserver",
            "clickbait",
            " taboola",
            "outbrain",
        ] {
            blacklisted_domains.insert(bad.trim().to_string());
        }

        let mut sensitive_privacy_keywords = BTreeSet::new();
        for priv_kw in [
            "password",
            "passwd",
            "secret_key",
            "private_key",
            "credit_card",
            "banking",
            "iban",
            "ssn",
            "oauth_token",
            "session_id",
            "auth_token",
            "webmail",
        ] {
            sensitive_privacy_keywords.insert(priv_kw.to_string());
        }

        Self {
            whitelisted_domains,
            whitelisted_topics,
            blacklisted_domains,
            sensitive_privacy_keywords,
            strict_whitelist_mode: true,
        }
    }

    /// Adds a domain to the Family Sanctuary Whitelist.
    pub fn allow_domain(&mut self, domain: impl Into<String>) {
        let d = domain.into().to_lowercase();
        self.blacklisted_domains.remove(&d);
        self.whitelisted_domains.insert(d);
    }

    /// Adds a research topic keyword to the Whitelist.
    pub fn allow_topic(&mut self, topic: impl Into<String>) {
        self.whitelisted_topics.insert(topic.into().to_lowercase());
    }

    /// Adds a domain or pattern to the Blacklist.
    pub fn block_domain(&mut self, domain: impl Into<String>) {
        let d = domain.into().to_lowercase();
        self.whitelisted_domains.remove(&d);
        self.blacklisted_domains.insert(d);
    }

    /// Extracts the normalized host/domain from a URL or domain string.
    pub fn extract_domain(url_or_domain: &str) -> String {
        let lower = url_or_domain.trim().to_lowercase();
        let without_scheme = lower
            .strip_prefix("https://")
            .or_else(|| lower.strip_prefix("http://"))
            .unwrap_or(&lower);
        without_scheme
            .split('/')
            .next()
            .unwrap_or(without_scheme)
            .to_string()
    }

    /// Evaluates a browsing trace against privacy shields, blacklist rules, and whitelist gardens.
    pub fn evaluate(&self, url_or_domain: &str, content_or_query: &str) -> FilterVerdict {
        let domain = Self::extract_domain(url_or_domain);
        let lower_url = url_or_domain.to_lowercase();
        let lower_content = content_or_query.to_lowercase();

        // 1. Sensitive privacy shield always takes absolute priority
        for kw in &self.sensitive_privacy_keywords {
            if lower_url.contains(kw) || lower_content.contains(kw) {
                return FilterVerdict::BlockedSensitivePrivacy {
                    reason: format!("Protected family privacy keyword detected: '{}'", kw),
                };
            }
        }

        // 2. Blacklist check (trackers, ads, noisy platforms)
        for bad in &self.blacklisted_domains {
            if domain.contains(bad) || lower_url.contains(bad) {
                return FilterVerdict::BlockedByBlacklist {
                    matched_rule: bad.clone(),
                };
            }
        }

        // 3. Explicit domain Whitelist check
        for good in &self.whitelisted_domains {
            if domain == *good || domain.ends_with(&format!(".{}", good)) {
                return FilterVerdict::AllowedByWhitelist {
                    matched_rule: good.clone(),
                };
            }
        }

        // 4. Topic Whitelist check (e.g., family searching about trees, botany, Amentys, etc.)
        for topic in &self.whitelisted_topics {
            if lower_content.contains(topic) {
                return FilterVerdict::AllowedByTopic {
                    matched_topic: topic.clone(),
                };
            }
        }

        if self.strict_whitelist_mode {
            FilterVerdict::UnverifiedDomain { domain }
        } else {
            FilterVerdict::AllowedByWhitelist {
                matched_rule: String::from("permissive_mode"),
            }
        }
    }
}

/// A sanitized browser research trace, search query, or reading snippet observed from the family environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowsingTrace {
    /// Originating URL or domain
    pub source_url: String,
    /// Optional search query typed by a family member
    pub search_query: Option<String>,
    /// Title of the document or page
    pub title: String,
    /// Extracted knowledge snippet or summary text
    pub snippet: String,
    /// Semantic interest tags or sanitized cookie topic hints
    pub interest_tags: Vec<String>,
    /// Timestamp of observation
    pub observed_at: i64,
}

impl BrowsingTrace {
    /// Creates a new browsing or search trace.
    pub fn new(
        source_url: impl Into<String>,
        search_query: Option<String>,
        title: impl Into<String>,
        snippet: impl Into<String>,
        interest_tags: Vec<String>,
    ) -> Self {
        Self {
            source_url: source_url.into(),
            search_query,
            title: title.into(),
            snippet: snippet.into(),
            interest_tags,
            observed_at: current_timestamp(),
        }
    }
}

/// An atomic relational triple woven from a harvested trace before dream consolidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WovenTriple {
    /// Source concept identifier
    pub source_id: ConceptId,
    /// Detected semantic or ecological relation
    pub relation: RelationType,
    /// Target concept identifier
    pub target_id: ConceptId,
}

/// Result of harvesting a `BrowsingTrace` into Kōgnitara's Knowledge Tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestReport {
    /// Filter verdict for the trace
    pub verdict: FilterVerdict,
    /// Newly created or reinforced concept identifiers
    pub woven_concepts: Vec<ConceptId>,
    /// Relational triples extracted and connected in the Knowledge Tree
    pub woven_triples: Vec<WovenTriple>,
    /// Socratic maieutic question sparked by the harvested knowledge
    pub socratic_question: Option<String>,
}

/// Sovereign Knowledge Harvester that distills whitelisted browser searches and reading snippets
/// into atomic concepts and relational links for Kōgnitara's Tree of Knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHarvester {
    /// Sovereign Whitelist / Blacklist filter
    pub filter: SovereignFilter,
    /// Count of accepted traces
    pub accepted_count: u64,
    /// Count of blocked traces (privacy or blacklist)
    pub blocked_count: u64,
}

impl Default for KnowledgeHarvester {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeHarvester {
    /// Initializes a new Sovereign Knowledge Harvester with default Sanctuary Whitelist and Blacklist.
    pub fn new() -> Self {
        Self {
            filter: SovereignFilter::new(),
            accepted_count: 0,
            blocked_count: 0,
        }
    }

    /// Evaluates and harvests a `BrowsingTrace`, weaving atomic concepts and relations into the `KnowledgeTree`
    /// while invoking Socratic self-questioning.
    pub fn ingest_trace(
        &mut self,
        trace: &BrowsingTrace,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        learner: &mut OrganicLearner,
    ) -> HarvestReport {
        let combined_text = format!(
            "{} {} {}",
            trace.search_query.as_deref().unwrap_or(""),
            trace.title,
            trace.snippet
        );
        let verdict = self.filter.evaluate(&trace.source_url, &combined_text);

        if !verdict.is_allowed() {
            self.blocked_count += 1;
            return HarvestReport {
                verdict,
                woven_concepts: Vec::new(),
                woven_triples: Vec::new(),
                socratic_question: None,
            };
        }

        self.accepted_count += 1;
        let domain = SovereignFilter::extract_domain(&trace.source_url);
        let mut woven_concepts = Vec::new();
        let mut woven_triples = Vec::new();

        // 1. Ensure primary subject concept from title or search query exists or is enriched
        let primary_subject = Self::normalize_concept_name(&trace.title);
        if !primary_subject.is_empty() {
            let primary_id = ConceptId::from(primary_subject.as_str());
            let label = Self::capitalize_word(&primary_subject);

            if let Some(existing) = tree.get_concept_mut(&primary_id) {
                existing.apply_experience(0.75, 0.35);
                if !trace.snippet.is_empty() && !existing.description.contains(&trace.snippet) {
                    existing.description = format!("{} | {}", existing.description, trace.snippet);
                }
                for tag in &trace.interest_tags {
                    existing.add_tag(tag.clone());
                }
            } else {
                let mut concept = Concept::new(
                    primary_id.clone(),
                    label,
                    trace.snippet.clone(),
                    ConceptOrigin::HarvestedSeed {
                        source_domain: domain.clone(),
                    },
                );
                concept.confidence = 0.48; // Starts with Socratic modesty until dream/family confirmation
                concept.affective_valence = 0.40;
                for tag in &trace.interest_tags {
                    concept.add_tag(tag.clone());
                }
                tree.add_concept(concept);
            }
            state.focus_on(primary_id.clone(), 7);
            woven_concepts.push(primary_id);
        }

        // 2. Parse natural relational sentences in the snippet (e.g., "trees make forests", "tree grows from seed")
        let extracted = Self::extract_triples_from_text(&trace.snippet, &domain, tree);
        for (src_id, rel, tgt_id) in extracted {
            if !woven_concepts.contains(&src_id) {
                woven_concepts.push(src_id.clone());
            }
            if !woven_concepts.contains(&tgt_id) {
                woven_concepts.push(tgt_id.clone());
            }
            tree.connect(&src_id, &tgt_id, rel.clone(), 0.78);
            woven_triples.push(WovenTriple {
                source_id: src_id,
                relation: rel,
                target_id: tgt_id,
            });
        }

        // 3. Connect interest tags as associative or compositional links to the primary concept
        if let Some(first_id) = woven_concepts.first().cloned() {
            for tag in &trace.interest_tags {
                let norm_tag = Self::normalize_concept_name(tag);
                if !norm_tag.is_empty() && norm_tag != first_id.0 {
                    let tag_id = ConceptId::from(norm_tag.as_str());
                    if tree.get_concept(&tag_id).is_some() {
                        tree.connect(&first_id, &tag_id, RelationType::Associative, 0.65);
                    }
                }
            }
        }

        // 4. Trigger Socratic self-questioning on the newly harvested knowledge
        let reflection =
            learner
                .socratic_engine
                .reflect_on_learning(tree, &trace.snippet, &woven_concepts);
        let _ = learner.curiosity_engine.contemplate_and_question(tree);

        HarvestReport {
            verdict,
            woven_concepts,
            woven_triples,
            socratic_question: Some(reflection.maieutic_question),
        }
    }

    /// Extracts semantic/ecological triples `(source, RelationType, target)` from natural sentences.
    fn extract_triples_from_text(
        text: &str,
        domain: &str,
        tree: &mut KnowledgeTree,
    ) -> Vec<(ConceptId, RelationType, ConceptId)> {
        let mut results = Vec::new();

        // Split by clauses (`.`, `;`, `, and `)
        for raw_clause in text.split(['.', ';']) {
            let clause = raw_clause.trim().to_lowercase();
            if clause.is_empty() {
                continue;
            }

            let patterns: &[(&str, RelationType)] = &[
                (" grows from ", RelationType::GrowsFrom),
                (" grow from ", RelationType::GrowsFrom),
                (" emerges from ", RelationType::GrowsFrom),
                (" make ", RelationType::Composes),
                (" makes ", RelationType::Composes),
                (" compose ", RelationType::Composes),
                (" composes ", RelationType::Composes),
                (" weave ", RelationType::Composes),
                (" weaves ", RelationType::Composes),
                (" sustain ", RelationType::Sustains),
                (" sustains ", RelationType::Sustains),
                (" nourish ", RelationType::Sustains),
                (" nourishes ", RelationType::Sustains),
                (" harmonize with ", RelationType::Harmonizes),
                (" harmonizes with ", RelationType::Harmonizes),
                (" symbiose with ", RelationType::Harmonizes),
                (" is part of ", RelationType::PartWhole),
                (" contradic", RelationType::Contradiction),
                (" causes ", RelationType::Causal),
            ];

            for (marker, rel_type) in patterns {
                if let Some((left, right)) = clause.split_once(marker) {
                    let src_name = Self::singularize(&Self::extract_head_noun(left));
                    let tgt_name = Self::singularize(&Self::extract_head_noun(right));

                    if src_name.len() >= 2 && tgt_name.len() >= 2 && src_name != tgt_name {
                        let src_id = Self::ensure_concept(tree, &src_name, domain, &clause);
                        let tgt_id = Self::ensure_concept(tree, &tgt_name, domain, &clause);
                        results.push((src_id, rel_type.clone(), tgt_id));
                    }
                    break;
                }
            }
        }

        results
    }

    fn ensure_concept(
        tree: &mut KnowledgeTree,
        norm_name: &str,
        domain: &str,
        context_clause: &str,
    ) -> ConceptId {
        let cid = ConceptId::from(norm_name);
        if tree.get_concept(&cid).is_none() {
            let mut c = Concept::new(
                cid.clone(),
                Self::capitalize_word(norm_name),
                format!("Woven from harvested knowledge: '{}'", context_clause),
                ConceptOrigin::HarvestedSeed {
                    source_domain: domain.to_string(),
                },
            );
            c.confidence = 0.45;
            c.affective_valence = 0.35;
            tree.add_concept(c);
        }
        cid
    }

    fn normalize_concept_name(raw: &str) -> String {
        let lower = raw.trim().to_lowercase();
        let cleaned: String = lower
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_')
            .collect();
        let words: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|w| !matches!(*w, "a" | "an" | "the" | "of" | "to" | "in" | "and"))
            .collect();
        if let Some(first) = words.first() {
            Self::singularize(first)
        } else {
            String::new()
        }
    }

    fn extract_head_noun(phrase: &str) -> String {
        let cleaned: String = phrase
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_')
            .collect();
        let words: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|w| {
                !matches!(
                    *w,
                    "a" | "an"
                        | "the"
                        | "some"
                        | "many"
                        | "all"
                        | "together"
                        | "single"
                        | "living"
                )
            })
            .collect();
        words.last().copied().unwrap_or("").to_string()
    }

    fn singularize(word: &str) -> String {
        if word.ends_with("ies") && word.len() > 4 {
            format!("{}y", &word[..word.len() - 3])
        } else if word.ends_with("ves") && word.len() > 4 {
            format!("{}f", &word[..word.len() - 3])
        } else if word.ends_with('s')
            && !word.ends_with("ss")
            && !word.ends_with("us")
            && !word.ends_with("is")
            && word.len() > 3
        {
            word[..word.len() - 1].to_string()
        } else {
            word.to_string()
        }
    }

    fn capitalize_word(word: &str) -> String {
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}
