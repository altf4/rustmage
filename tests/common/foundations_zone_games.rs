//! Whole zone-replacement/protection consumers in natural Foundations games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Card, CastFace, CastMethod, Keywords, Observation, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_four_zone_cards_natural_games_and_owned_rule_state() {
    let blue = deck(
        "Island",
        &[
            "Fog Bank",
            "Think Twice",
            "Hedron Archive",
            "Solemn Simulacrum",
            "Gilded Lotus",
            "Icewind Elemental",
            "Omniscience",
            "Darksteel Colossus",
            "Progenitus",
        ],
    );
    let mut green = deck(
        "Forest",
        &[
            "Dryad Militant",
            "Llanowar Elves",
            "Resolute Reinforcements",
            "Wilt-Leaf Liege",
            "Hedron Archive",
            "Gilded Lotus",
            "Solemn Simulacrum",
            "Angel of Vitality",
            "Inspiring Overseer",
        ],
    );
    green[..12].fill(Card::Plains);
    let mut black = deck(
        "Swamp",
        &[
            "Burglar Rat",
            "Pilfer",
            "Bloodtithe Collector",
            "Hungry Ghoul",
            "Arbiter of Woe",
            "Maalfeld Twins",
            "Seize the Spoils",
            "Gilded Lotus",
            "Dragon Mage",
        ],
    );
    black[..4].fill(Card::Mountain);
    let mut shuffled = BTreeSet::new();
    let mut dryad_exile = false;
    let mut dryad_routes = BTreeSet::new();
    let mut liege_discard = false;
    let mut liege_bonus = false;
    let mut protected_attack = false;
    let mut owned_zone_rules = 0;
    let mut observe = |before: &Observation, after: &Observation, replay: bool| {
        for reveal in &after.zone_reveals {
            if ["Darksteel Colossus", "Progenitus"].contains(&reveal.card.name()) {
                shuffled.insert(reveal.card.name());
            }
        }
        // These normal-face spells go to the graveyard after resolution.
        // In particular, Think Twice's flashback self-exile is not evidence.
        if let Some(item) = before.resolving.as_ref().or_else(|| before.stack.last())
            && item.kind == StackKind::Spell
            && !item.is_copy
            && item.cast.is_some_and(|cast| {
                cast.method == CastMethod::Normal
                    && cast.origin == Zone::Hand
                    && cast.face == CastFace::Normal
            })
            && (item.source_card.is_instant()
                || item.source_card.info().type_line.contains("Sorcery"))
            && ["Think Twice", "Pilfer", "Seize the Spoils"].contains(&item.source_card.name())
            && (before
                .resolving
                .as_ref()
                .is_some_and(|current| current.id == item.id)
                || !after.stack.iter().any(|queued| queued.id == item.id))
            && before
                .objects
                .iter()
                .any(|o| o.reference == item.object && o.zone == Zone::Stack)
            && before.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && o.name() == "Dryad Militant"
                    && o.abilities
                        .as_ref()
                        .is_none_or(|abilities| abilities.printed)
            })
            && after
                .objects
                .iter()
                .any(|o| o.id == item.card && o.zone == Zone::Exile && o.reference != item.object)
        {
            dryad_exile = true;
            dryad_routes.insert(item.source_card.name());
        }
        liege_discard |= before.objects.iter().any(|old| {
            old.zone == Zone::Hand
                && old.name() == "Wilt-Leaf Liege"
                && after.objects.iter().any(|new| {
                    new.id == old.id
                        && new.zone == Zone::Battlefield
                        && new.reference != old.reference
                })
        });
        liege_bonus |= after.objects.iter().any(|o| {
            o.zone == Zone::Battlefield
                && o.name() == "Dryad Militant"
                && o.power >= 4
                && o.toughness >= 3
        });
        protected_attack |= after.objects.iter().any(|o| {
            o.zone == Zone::Battlefield
                && o.name() == "Progenitus"
                && o.attacking
                && o.keywords.contains(Keywords::PROTECTION_EVERYTHING)
        });
        if replay
            && before.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && [
                        "Dryad Militant",
                        "Wilt-Leaf Liege",
                        "Darksteel Colossus",
                        "Progenitus",
                    ]
                    .contains(&o.name())
            })
        {
            owned_zone_rules += 1;
        }
    };
    run_matrix_actions_observed(
        &[blue, green, black],
        &[
            "Darksteel Colossus",
            "Dryad Militant",
            "Progenitus",
            "Wilt-Leaf Liege",
        ],
        &[],
        &["Hedron Archive", "Gilded Lotus"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    for name in ["Darksteel Colossus", "Progenitus"] {
        assert!(
            shuffled.contains(name),
            "no actual reveal/shuffle replacement for {name}; {shuffled:?}"
        );
    }
    assert!(dryad_exile, "no Dryad spell exile replacement");
    assert!(
        liege_discard,
        "no opponent-caused Liege discard-to-battlefield"
    );
    assert!(liege_bonus, "no combined green/white Liege bonus");
    assert!(protected_attack, "no protected Progenitus attack");
    assert!(
        owned_zone_rules > 0,
        "no owned replay with actual zone/protection providers"
    );
    println!(
        "FDN zone witnesses: shuffles={shuffled:?}, ordinary Dryad exile={dryad_routes:?}, Liege discard/bonuses, Progenitus attack; owned rule replays={owned_zone_rules}"
    );
}
