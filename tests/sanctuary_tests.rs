use klm::Kognitara;
use klm::family::{FamilyMember, FamilyRole, MemberId, TrustGate};
use klm::{BrowsingTrace, EvidenceRelation, FilterVerdict, SovereignFilter};

#[test]
fn test_kognitara_family_teaching_mistakes_curiosity_and_dream() {
    let mut kognitara = Kognitara::new("Sanctuary of House Ki");
    let ki = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(ki);

    let member_id = MemberId::from("ki");
    let stranger_id = MemberId::from("stranger");
    assert!(kognitara.sanctuary.is_family(&member_id));
    assert!(!kognitara.sanctuary.is_family(&stranger_id));

    assert!(matches!(
        kognitara.sanctuary.evaluate_trust(&member_id),
        TrustGate::FamilyCircle { .. }
    ));
    assert!(matches!(
        kognitara.sanctuary.evaluate_trust(&stranger_id),
        TrustGate::ExternalVisitor { .. }
    ));

    // Family member teaches a core concept
    let taught_id = kognitara
        .teach_concept(
            &member_id,
            "resilience",
            "Resilience",
            "Growing stronger through gentle perseverance",
            &["strength", "patience"],
        )
        .expect("Family member should be allowed to teach");
    assert_eq!(taught_id.0, "resilience");
    assert_eq!(kognitara.sanctuary.values.len(), 1);

    // Stranger cannot anchor filiation concepts
    assert!(
        kognitara
            .teach_concept(&stranger_id, "dogma", "Dogma", "Rigid rule", &[])
            .is_err()
    );

    // Novel conversation triggers nascent concept and curiosity question
    let (response, activated) =
        kognitara.converse(&member_id, "Exploring mysterious cosmic harmonies");
    assert!(!response.is_empty());
    assert!(!activated.is_empty());

    let questions = kognitara.pending_questions();
    assert!(!questions.is_empty());

    // Gentle correction of a mistake
    let lessons = kognitara
        .correct_mistake(
            &member_id,
            "exp_1",
            "Cosmic Resonance",
            "Harmony emerges from listening before speaking",
        )
        .expect("Correction should succeed");
    assert!(!lessons.is_empty());
    assert!(lessons[0].humility_gained > 0.0);
    let dreams = kognitara.dream_cycle(|_, _| {});
    assert!(!dreams.is_empty());
    assert!(!kognitara.dream_oracles().is_empty());

    let snapshot = kognitara
        .create_snapshot()
        .expect("Snapshot should succeed");
    assert_ne!(snapshot.merkle_root, [0u8; 32]);
}

#[test]
fn harvested_hypotheses_keep_sources_and_corrections() {
    let mut mind = Kognitara::new("Test");
    let trace = BrowsingTrace::new(
        "https://en.wikipedia.org/wiki/Tree",
        None,
        "Tree",
        "Trees grow from seeds.",
        vec![],
    );
    mind.harvest_browsing_trace(&trace).unwrap();
    mind.harvest_browsing_trace(&trace).unwrap();
    assert_eq!(mind.hypotheses.hypotheses.len(), 1);
    let hypothesis = &mind.hypotheses.hypotheses[0];
    assert_eq!(hypothesis.statement, trace.snippet);
    assert_eq!(hypothesis.evidence[0].relation, EvidenceRelation::Reports);
    assert_eq!(hypothesis.evidence[0].source, trace.source_url);
    assert!(!hypothesis.related_concepts.is_empty());

    let hypothesis = mind.hypotheses.get_mut(hypothesis.id).unwrap();
    hypothesis
        .add_evidence(
            EvidenceRelation::Contradicts,
            "book",
            "Some reproduce vegetatively",
            42,
        )
        .unwrap();
    hypothesis
        .correct("Many trees grow from seeds.", "Some reproduce vegetatively")
        .unwrap();
    hypothesis
        .add_evidence(
            EvidenceRelation::Supports,
            "another source",
            "Updated observation",
            43,
        )
        .unwrap();
    assert_eq!(
        hypothesis.corrections[0].previous_statement,
        "Trees grow from seeds."
    );
    assert_eq!(hypothesis.evidence[1].revision, 0);
    assert_eq!(hypothesis.evidence[2].revision, 1);
    assert_eq!(hypothesis.revision(), 1);
    assert!(hypothesis.correct("", "reason").is_err());
    assert!(
        hypothesis
            .add_evidence(EvidenceRelation::Supports, "", "note", 0)
            .is_err()
    );

    let serialized = serde_json::to_string(&mind).unwrap();
    let restored: Kognitara = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored.hypotheses.hypotheses[0].corrections.len(), 1);
    assert_eq!(restored.hypotheses.hypotheses[0].evidence.len(), 3);

    let blocked = BrowsingTrace::new(
        "https://facebook.com/x",
        None,
        "Tree",
        "Trees grow.",
        vec![],
    );
    let report = mind.harvest_browsing_trace(&blocked).unwrap();
    assert!(!report.verdict.is_allowed());
    assert_eq!(mind.hypotheses.hypotheses.len(), 1);
    let missing_source = BrowsingTrace::new("", None, "Tree", "Trees grow.", vec![]);
    assert!(mind.harvest_browsing_trace(&missing_source).is_err());
    assert_eq!(mind.hypotheses.hypotheses.len(), 1);
}

#[test]
fn responses_and_manuals_show_related_current_sources_without_verifying_them() {
    let mut mind = Kognitara::new("Test");
    let trace = BrowsingTrace::new(
        "https://en.wikipedia.org/wiki/Tree",
        None,
        "Tree",
        "Trees grow from seeds.",
        vec![],
    );
    mind.harvest_browsing_trace(&trace).unwrap();
    let other = BrowsingTrace::new(
        "https://en.wikipedia.org/wiki/Ocean",
        None,
        "Ocean",
        "Oceans contain water.",
        vec![],
    );
    mind.harvest_browsing_trace(&other).unwrap();

    let (answer, _) = mind.converse(&MemberId::from("ki"), "Tell me about trees");
    assert!(answer.contains("Unverified hypothesis #1"));
    assert!(answer.contains(&trace.source_url));
    assert!(answer.contains("not fact verification"));
    assert!(!answer.contains(&other.source_url));
    for page in [
        mind.render_markdown_page("tree").unwrap(),
        mind.render_man_page("tree").unwrap(),
        mind.render_html_ark_page("tree").unwrap(),
    ] {
        assert!(page.contains("Unverified hypothesis #1"));
        assert!(page.contains(&trace.source_url));
        assert!(page.contains("not fact verification"));
        assert!(!page.contains(&other.source_url));
    }

    mind.hypotheses
        .get_mut(1)
        .unwrap()
        .correct("Many trees grow from seeds.", "Not all trees do")
        .unwrap();
    let page = mind.render_markdown_page("tree").unwrap();
    assert!(page.contains("Many trees grow from seeds."));
    assert!(page.contains("No source recorded for this revision."));
    assert!(!page.contains(&trace.source_url));
    mind.hypotheses
        .get_mut(1)
        .unwrap()
        .add_evidence(EvidenceRelation::Supports, "new source", "Updated", 42)
        .unwrap();
    let page = mind.render_markdown_page("tree").unwrap();
    assert!(page.contains("Supports: new source"));
    assert!(!page.contains(&trace.source_url));
}

#[test]
fn confidence_growth_does_not_resolve_questions() {
    let mut mind = Kognitara::new("Test");
    let member = MemberId::from("ki");
    mind.converse(&member, "A new and unfamiliar thought");
    let questions_before = mind.pending_questions();
    assert!(!questions_before.is_empty());
    let question = questions_before[0].question_text.clone();
    for brain in mind.swarm.brains.values_mut() {
        for concept in brain.tree.concepts.values_mut() {
            concept.confidence = 1.0;
        }
        brain
            .learner
            .curiosity_engine
            .contemplate_and_question(&brain.tree);
    }
    mind.rest_cycle();
    mind.contemplate_cycle(1);
    assert!(
        mind.pending_questions()
            .iter()
            .any(|q| q.question_text == question)
    );
}

#[test]
fn old_saves_reopen_only_questions_resolved_by_maturation() {
    let mut mind = Kognitara::new("Test");
    mind.converse(&MemberId::from("ki"), "A new and unfamiliar thought");
    let brain = mind
        .swarm
        .brains
        .values_mut()
        .find(|b| !b.learner.curiosity_engine.questions.is_empty())
        .unwrap();
    let question = &mut brain.learner.curiosity_engine.questions[0];
    question.is_resolved = true;
    question.resolution_insight =
        Some("Assimilated autonomously through gravitational maturation and rest.".into());
    let mut answered = question.clone();
    answered.id = "human-answer".into();
    answered.resolution_insight = Some("An explicit answer".into());
    brain.learner.curiosity_engine.questions.push(answered);
    let mut saved = serde_json::to_value(&mind).unwrap();
    saved.as_object_mut().unwrap().remove("hypotheses");
    let restored: Kognitara = serde_json::from_value(saved).unwrap();
    assert!(restored.hypotheses.hypotheses.is_empty());
    assert!(!restored.pending_questions().is_empty());
    assert!(restored.swarm.brains.values().any(|brain| {
        brain
            .learner
            .curiosity_engine
            .questions
            .iter()
            .any(|question| question.id == "human-answer" && question.is_resolved)
    }));
}

#[test]
fn wikimedia_projects_are_whitelisted_without_accepting_lookalike_domains() {
    let filter = SovereignFilter::new();
    for domain in [
        "wikipedia.org",
        "wiktionary.org",
        "wikibooks.org",
        "wikinews.org",
        "wikiversity.org",
        "wikisource.org",
        "wikiquote.org",
        "wikivoyage.org",
        "wikidata.org",
        "wikimedia.org",
        "mediawiki.org",
        "wikifunctions.org",
    ] {
        let url = format!("https://fr.{domain}/wiki/Test");
        assert_eq!(
            filter.evaluate(&url, "unrelated text"),
            FilterVerdict::AllowedByWhitelist {
                matched_rule: domain.into()
            },
            "{url}"
        );
    }
    assert!(matches!(
        filter.evaluate("https://wikibooks.org.example.com/wiki/Test", "unrelated"),
        FilterVerdict::UnverifiedDomain { .. }
    ));
    assert!(matches!(
        filter.evaluate("https://fr.wikibooks.org/wiki/Test", "password"),
        FilterVerdict::BlockedSensitivePrivacy { .. }
    ));
}

#[test]
fn older_filters_gain_new_wikimedia_domains_without_undoing_blocks() {
    let mut old = SovereignFilter::new();
    old.block_domain("wikibooks.org");
    old.block_domain("wikipedia.org");
    old.whitelisted_domains.remove("wikisource.org");
    let mut saved = serde_json::to_value(old).unwrap();
    saved
        .as_object_mut()
        .unwrap()
        .remove("wikimedia_sources_version");
    let restored: SovereignFilter = serde_json::from_value(saved).unwrap();
    assert!(matches!(
        restored.evaluate("https://fr.wikisource.org/wiki/Test", "unrelated"),
        FilterVerdict::AllowedByWhitelist { .. }
    ));
    for domain in ["wikibooks.org", "wikipedia.org"] {
        assert!(matches!(
            restored.evaluate(domain, "unrelated"),
            FilterVerdict::BlockedByBlacklist { .. }
        ));
    }
    let mut restored = restored;
    restored.block_domain("wikisource.org");
    let roundtrip: SovereignFilter =
        serde_json::from_str(&serde_json::to_string(&restored).unwrap()).unwrap();
    assert!(matches!(
        roundtrip.evaluate("fr.wikisource.org", "unrelated"),
        FilterVerdict::BlockedByBlacklist { .. }
    ));
}
