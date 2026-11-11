extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::council::assembly::CouncilDeliberation;
use crate::dream::journal::DreamOracle;
use crate::family::sanctuary::Sanctuary;
use crate::mind::socratic::SocraticReflection;
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// A structured entry in Kōgnitara's Living Manual (used by external `kog` CLI and Offline Web Ark).
/// Dynamically woven from the `KnowledgeTree`, `DreamOracle` insights, `Sanctuary` values, and `CouncilOfElders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingManualEntry {
    /// Canonical concept identifier
    pub concept_id: ConceptId,
    /// Human-readable concept title
    pub label: String,
    /// Concise one-line synopsis for the `NAME` section
    pub synopsis: String,
    /// Ontological filiation chain (e.g., `["earth", "seed", "tree", "forest", "water"]`)
    pub filiation_chain: Vec<String>,
    /// Dynamically woven prose describing the essence, relations, and purpose of the concept
    pub woven_essence: String,
    /// Epistemic confidence percentage (`0..=100`)
    pub confidence_percent: u8,
    /// Number of experiential and oniric engagements
    pub experience_count: u64,
    /// Concepts from which this concept grows or originates (`GrowsFrom`, incoming `Filiation`)
    pub grows_from: Vec<String>,
    /// Larger wholes that this concept composes (`Composes`, `PartWhole`)
    pub composes: Vec<String>,
    /// Systems or beings that this concept sustains or causes (`Sustains`, `Causal`)
    pub sustains: Vec<String>,
    /// Symbiotic partners, emotional resonances, and dream analogies (`Harmonizes`, `Analogy`, `EmotionalResonance`)
    pub harmonizes: Vec<String>,
    /// Concepts in dialectical tension or opposition (`Contradiction`)
    pub contradicts: Vec<String>,
    /// Notes from Family teachings, Dream Oracles, and the Council of Elders
    pub family_and_elders_notes: Vec<String>,
    /// Latest Socratic maieutic question inviting deeper reflection
    pub socratic_inquiry: Option<String>,
    /// Related concepts for cross-navigation (`kog <concept>`)
    pub see_also: Vec<String>,
}

/// Generator for Kōgnitara's Living Manual (`kog` man-of-life pages) and the Resilient Offline Web Ark.
#[derive(Debug, Clone, Default)]
pub struct LivingManual;

impl LivingManual {
    /// Synthesizes a `LivingManualEntry` for `query` from the provided `KnowledgeTree` and cognitive records.
    pub fn generate_entry(
        tree: &KnowledgeTree,
        sanctuary: &Sanctuary,
        oracles: &[DreamOracle],
        reflections: &[SocraticReflection],
        deliberations: &[CouncilDeliberation],
        query: &str,
    ) -> Option<LivingManualEntry> {
        let concept = tree.resolve_concept(query)?;
        Some(Self::build_entry_for_concept(
            concept,
            tree,
            sanctuary,
            oracles,
            reflections,
            deliberations,
        ))
    }

    fn build_entry_for_concept(
        concept: &Concept,
        tree: &KnowledgeTree,
        sanctuary: &Sanctuary,
        oracles: &[DreamOracle],
        reflections: &[SocraticReflection],
        deliberations: &[CouncilDeliberation],
    ) -> LivingManualEntry {
        let id = &concept.id;
        let filiation_chain = tree.trace_filiation_chain(id);

        let mut grows_from = Vec::new();
        let mut composes = Vec::new();
        let mut sustains = Vec::new();
        let mut harmonizes = Vec::new();
        let mut contradicts = Vec::new();
        let mut see_also_set = BTreeSet::new();

        // Outgoing edges from `concept`
        for (target, edge) in tree.neighbors(id) {
            let name = target.id.0.clone();
            see_also_set.insert(name.clone());
            match edge.relation_type {
                RelationType::GrowsFrom => Self::push_unique(&mut grows_from, name),
                RelationType::Composes | RelationType::PartWhole => {
                    Self::push_unique(&mut composes, name)
                }
                RelationType::Sustains | RelationType::Causal => {
                    Self::push_unique(&mut sustains, name)
                }
                RelationType::Harmonizes
                | RelationType::Analogy
                | RelationType::EmotionalResonance
                | RelationType::Associative => Self::push_unique(&mut harmonizes, name),
                RelationType::Contradiction => Self::push_unique(&mut contradicts, name),
                RelationType::Filiation | RelationType::TemporalSequence => {
                    Self::push_unique(&mut sustains, name)
                }
            }
        }

        // Incoming edges pointing to `concept`
        for (source, edge) in tree.incoming_neighbors(id) {
            let name = source.id.0.clone();
            see_also_set.insert(name.clone());
            match edge.relation_type {
                RelationType::Filiation | RelationType::Causal => {
                    Self::push_unique(&mut grows_from, name)
                }
                RelationType::Composes | RelationType::PartWhole => {
                    Self::push_unique(&mut grows_from, name)
                }
                RelationType::Harmonizes
                | RelationType::Analogy
                | RelationType::EmotionalResonance => Self::push_unique(&mut harmonizes, name),
                RelationType::Contradiction => Self::push_unique(&mut contradicts, name),
                _ => {}
            }
        }

        // Dynamically weave the ESSENCE prose from the concept and its graph neighborhood
        let woven_essence = Self::weave_essence_prose(
            concept,
            &grows_from,
            &composes,
            &sustains,
            &harmonizes,
        );

        // Gather Family, Dream Oracle, and Elder notes
        let mut notes = Vec::new();
        match &concept.origin {
            ConceptOrigin::FamilyTaught { member_id } => {
                notes.push(format!(
                    "Anchored through Family Filiation by '{}'.",
                    member_id
                ));
            }
            ConceptOrigin::HarvestedSeed { source_domain } => {
                notes.push(format!(
                    "Harvested from sovereign whitelisted source '{}'.",
                    source_domain
                ));
            }
            ConceptOrigin::DreamSynthesized { dream_id } => {
                notes.push(format!("Synthesized during nocturnal dream '{}'.", dream_id));
            }
            ConceptOrigin::ReawakenedFossil { fossil_hash } => {
                notes.push(format!(
                    "Reawakened from Kpack fossil ({}) through Socratic Anamnesis.",
                    &fossil_hash[..fossil_hash.len().min(12)]
                ));
            }
            ConceptOrigin::Innate => {
                notes.push(String::from(
                    "Foundational innate root of Kōgnitara's living ontology.",
                ));
            }
            ConceptOrigin::EmpiricalTrial | ConceptOrigin::SpontaneousAnalogy => {
                notes.push(String::from(
                    "Discovered through organic experience and trial-and-error.",
                ));
            }
        }

        for val in &sanctuary.values {
            if val.title.to_lowercase().contains(&concept.label.to_lowercase())
                || val
                    .principle
                    .to_lowercase()
                    .contains(&concept.label.to_lowercase())
            {
                notes.push(format!(
                    "Sanctuary Value ({}): \"{}\"",
                    val.entrusted_by.0, val.principle
                ));
            }
        }

        for oracle in oracles {
            if &oracle.bridged_concepts.0 == id
                || &oracle.bridged_concepts.1 == id
                || &oracle.emergent_concept == id
            {
                notes.push(format!("Elder Oneiros (Dream Oracle): {}", oracle.revelation));
            }
        }

        for delib in deliberations {
            if delib
                .subject
                .to_lowercase()
                .contains(&concept.label.to_lowercase())
            {
                notes.push(format!(
                    "Council of Elders: {}",
                    delib.collective_socratic_guidance
                ));
            }
        }

        let socratic_inquiry = reflections
            .iter()
            .rev()
            .find(|r| {
                r.concepts.contains(id)
                    || r.examined_hypothesis
                        .to_lowercase()
                        .contains(&concept.label.to_lowercase())
            })
            .map(|r| r.maieutic_question.clone());

        let confidence_percent = (concept.confidence.clamp(0.0, 1.0) * 100.0) as u8;
        let see_also: Vec<String> = see_also_set.into_iter().collect();

        LivingManualEntry {
            concept_id: concept.id.clone(),
            label: concept.label.clone(),
            synopsis: concept.description.clone(),
            filiation_chain,
            woven_essence,
            confidence_percent,
            experience_count: concept.experience_count,
            grows_from,
            composes,
            sustains,
            harmonizes,
            contradicts,
            family_and_elders_notes: notes,
            socratic_inquiry,
            see_also,
        }
    }

    fn weave_essence_prose(
        concept: &Concept,
        grows_from: &[String],
        composes: &[String],
        sustains: &[String],
        harmonizes: &[String],
    ) -> String {
        let mut sentences = Vec::new();
        let name_lower = concept.label.to_lowercase();

        sentences.push(format!(
            "A {} is {} ({}).",
            name_lower,
            concept.description.trim_end_matches('.'),
            if concept.confidence >= 0.75 {
                "deeply consolidated knowledge"
            } else if concept.confidence >= 0.45 {
                "maturing Socratic intuition"
            } else {
                "nascent observation open to inquiry"
            }
        ));

        if !grows_from.is_empty() {
            sentences.push(format!(
                "It emerges and grows from {}.",
                grows_from.join(", ")
            ));
        }

        if !composes.is_empty() {
            sentences.push(format!(
                "Together, {}s weave and compose {}.",
                name_lower,
                composes.join(", ")
            ));
        }

        if !sustains.is_empty() {
            sentences.push(format!(
                "Through its presence, it nourishes and sustains {}.",
                sustains.join(", ")
            ));
        }

        if !harmonizes.is_empty() {
            sentences.push(format!(
                "It lives in resonance and harmony with {}.",
                harmonizes.join(", ")
            ));
        }

        sentences.join(" ")
    }

    /// Formats a `LivingManualEntry` as a classic Unix-style `KOG(1)` manual page for terminal output.
    pub fn render_man_page(entry: &LivingManualEntry) -> String {
        let mut out = String::new();
        let upper = entry.concept_id.0.to_uppercase();

        out.push_str(&format!(
            "{0}(1)                   Kōgnitara Living Manual                   {0}(1)\n\n",
            upper
        ));
        out.push_str("NAME\n");
        out.push_str(&format!(
            "    {} — {}\n\n",
            entry.concept_id.0, entry.synopsis
        ));

        if !entry.filiation_chain.is_empty() {
            out.push_str("FILIATION & ONTOLOGY\n");
            out.push_str(&format!(
                "    {}\n\n",
                entry.filiation_chain.join(" -> ")
            ));
        }

        out.push_str(&format!(
            "ESSENCE (Woven across {} experiences — Confidence: {}%)\n",
            entry.experience_count, entry.confidence_percent
        ));
        out.push_str(&format!("    {}\n\n", entry.woven_essence));

        out.push_str("RELATIONS IN THE TREE OF KNOWLEDGE\n");
        if !entry.grows_from.is_empty() {
            out.push_str(&format!(
                "    [GrowsFrom]    {}\n",
                entry.grows_from.join(", ")
            ));
        }
        if !entry.composes.is_empty() {
            out.push_str(&format!(
                "    [Composes]     {}\n",
                entry.composes.join(", ")
            ));
        }
        if !entry.sustains.is_empty() {
            out.push_str(&format!(
                "    [Sustains]     {}\n",
                entry.sustains.join(", ")
            ));
        }
        if !entry.harmonizes.is_empty() {
            out.push_str(&format!(
                "    [Harmonizes]   {}\n",
                entry.harmonizes.join(", ")
            ));
        }
        if !entry.contradicts.is_empty() {
            out.push_str(&format!(
                "    [Contradicts]  {}\n",
                entry.contradicts.join(", ")
            ));
        }
        out.push('\n');

        if !entry.family_and_elders_notes.is_empty() {
            out.push_str("FAMILY & ELDERS' NOTES\n");
            for note in &entry.family_and_elders_notes {
                out.push_str(&format!("    * {}\n", note));
            }
            out.push('\n');
        }

        if let Some(q) = &entry.socratic_inquiry {
            out.push_str("SOCRATIC INQUIRY\n");
            out.push_str(&format!("    \"{}\"\n\n", q));
        }

        if !entry.see_also.is_empty() {
            let refs: Vec<String> = entry
                .see_also
                .iter()
                .map(|c| format!("kog {}", c))
                .collect();
            out.push_str("SEE ALSO\n");
            out.push_str(&format!("    {}\n", refs.join(", ")));
        }

        out
    }

    /// Renders a self-contained, offline HTML page for the Local Web Ark (`http://sanctuary.local`)
    /// so the family can browse Kōgnitara's woven knowledge even during a complete internet blackout.
    pub fn render_html_ark_page(entry: &LivingManualEntry) -> String {
        let filiation_html = entry.filiation_chain.join(" &rarr; ");
        let see_also_links: Vec<String> = entry
            .see_also
            .iter()
            .map(|c| format!("<a href=\"/kog/{0}\">kog {0}</a>", c))
            .collect();
        let notes_html: Vec<String> = entry
            .family_and_elders_notes
            .iter()
            .map(|n| format!("<li>{}</li>", n))
            .collect();

        format!(
            "<!DOCTYPE html>\n\
<html lang=\"en\">\n\
<head>\n\
  <meta charset=\"utf-8\">\n\
  <title>kog {id} — Kōgnitara Offline Web Ark</title>\n\
  <style>\n\
    body {{ background: #0f1412; color: #e2ece7; font-family: monospace; max-width: 820px; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; }}\n\
    h1, h2 {{ color: #7fd1a8; border-bottom: 1px solid #263830; padding-bottom: 0.3rem; }}\n\
    .meta {{ color: #9ab5a7; font-size: 0.9rem; }}\n\
    .card {{ background: #16201c; border: 1px solid #283d34; padding: 1rem; border-radius: 6px; margin: 1rem 0; }}\n\
    a {{ color: #8ad6ff; text-decoration: none; }}\n\
  </style>\n\
</head>\n\
<body>\n\
  <h1>{id}(1) &mdash; {label}</h1>\n\
  <p class=\"meta\">Filiation: {filiation} | Confidence: {conf}% | Experiences: {exp}</p>\n\
  <div class=\"card\">\n\
    <h2>ESSENCE</h2>\n\
    <p>{essence}</p>\n\
  </div>\n\
  <div class=\"card\">\n\
    <h2>RELATIONS IN THE TREE OF KNOWLEDGE</h2>\n\
    <p><strong>GrowsFrom:</strong> {grows}</p>\n\
    <p><strong>Composes:</strong> {composes}</p>\n\
    <p><strong>Sustains:</strong> {sustains}</p>\n\
    <p><strong>Harmonizes:</strong> {harmonizes}</p>\n\
  </div>\n\
  <div class=\"card\">\n\
    <h2>FAMILY &amp; ELDERS' NOTES</h2>\n\
    <ul>{notes}</ul>\n\
  </div>\n\
  <p><strong>SEE ALSO:</strong> {see_also}</p>\n\
</body>\n\
</html>",
            id = entry.concept_id.0,
            label = entry.label,
            filiation = filiation_html,
            conf = entry.confidence_percent,
            exp = entry.experience_count,
            essence = entry.woven_essence,
            grows = entry.grows_from.join(", "),
            composes = entry.composes.join(", "),
            sustains = entry.sustains.join(", "),
            harmonizes = entry.harmonizes.join(", "),
            notes = notes_html.join(""),
            see_also = see_also_links.join(" | "),
        )
    }

    /// Renders an offline HTML index portal of all concepts currently woven in the `KnowledgeTree`.
    pub fn render_html_ark_index(tree: &KnowledgeTree, sanctuary_name: &str) -> String {
        let mut items = Vec::new();
        for (id, c) in &tree.concepts {
            items.push(format!(
                "<li><a href=\"/kog/{0}\"><strong>{1}</strong> (<code>{0}</code>)</a> &mdash; {2} <em>({3}%)</em></li>",
                id.0,
                c.label,
                c.description,
                (c.confidence.clamp(0.0, 1.0) * 100.0) as u8
            ));
        }

        format!(
            "<!DOCTYPE html>\n\
<html lang=\"en\">\n\
<head>\n\
  <meta charset=\"utf-8\">\n\
  <title>{sanctuary} — Kōgnitara Offline Web Ark</title>\n\
</head>\n\
<body>\n\
  <h1>{sanctuary} &mdash; Sovereign Offline Knowledge Ark</h1>\n\
  <p>Total woven concepts: {count}</p>\n\
  <ul>\n\
    {items}\n\
  </ul>\n\
</body>\n\
</html>",
            sanctuary = sanctuary_name,
            count = tree.concepts.len(),
            items = items.join("\n    ")
        )
    }

    fn push_unique(list: &mut Vec<String>, item: String) {
        if !list.contains(&item) {
            list.push(item);
        }
    }
}
