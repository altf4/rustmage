//! Four mana cards in natural FDN-only games; native rules tests cover edge cases.
use super::{deck, run_matrix_actions_observed};
use rustmage::{
    Card, CounterKind, Keywords, ManaRestrictionView, ManaRetention, ManaSpellBonusView,
    Observation, StackKind, Zone,
};

fn retained(observation: &Observation, player: rustmage::Player) -> u32 {
    observation
        .mana_lots
        .iter()
        .filter(|lot| lot.player == player && lot.retention == ManaRetention::Turn)
        .flat_map(|lot| lot.amounts)
        .sum()
}
fn orb_mana(observation: &Observation) -> u32 {
    observation
        .mana_lots
        .iter()
        .filter(|lot| lot.spell_bonus == ManaSpellBonusView::DragonCreatureHaste)
        .flat_map(|lot| lot.amounts)
        .sum()
}
#[test]
fn foundations_four_mana_cards_natural_games_and_owned_lot_state() {
    let mut angels = deck(
        "Plains",
        &[
            "Giada, Font of Hope",
            "Serra Angel",
            "Inspiring Overseer",
            "Angel of Vitality",
            "Resolute Reinforcements",
            "Hedron Archive",
            "Mazemind Tome",
            "Pacifism",
            "Banishing Light",
        ],
    );
    let mut dragons = deck(
        "Mountain",
        &[
            "Carnelian Orb of Dragonkind",
            "Savage Ventmaw",
            "Shivan Dragon",
            "Dragonlord's Servant",
            "Rapacious Dragon",
            "Llanowar Elves",
            "Hedron Archive",
            "Dragon Trainer",
            "Burst Lightning",
        ],
    );
    let courtyard = Card::from_name("Secluded Courtyard").unwrap();
    angels[..4].fill(courtyard);
    dragons[..4].fill(courtyard);
    dragons[4..12].fill(Card::Forest);
    let mut orb_spent = false;
    let mut orb_haste = false;
    let mut giada_entry = false;
    let mut courtyard_restricted_mana = false;
    let mut ventmaw_attack_mana = false;
    let mut retained_across_step = false;
    let mut owned_lot_replays = 0;
    let mut observe = |before: &Observation, after: &Observation, owned_replay: bool| {
        if owned_replay && !before.mana_lots.is_empty() {
            owned_lot_replays += 1;
        }
        orb_spent |= orb_mana(after) < orb_mana(before)
            && after.objects.iter().any(|object| {
                object.zone == Zone::Stack
                    && matches!(object.name(), "Shivan Dragon" | "Savage Ventmaw")
                    && object.keywords.contains(Keywords::HASTE)
                    && before.objects.iter().any(|prior| {
                        prior.id == object.id && !prior.keywords.contains(Keywords::HASTE)
                    })
            });
        orb_haste |= orb_spent
            && after.objects.iter().any(|object| {
                object.zone == Zone::Battlefield
                    && matches!(object.name(), "Shivan Dragon" | "Savage Ventmaw")
                    && object.keywords.contains(Keywords::HASTE)
            });
        courtyard_restricted_mana |= after.mana_lots.iter().any(|lot| {
            matches!(
                lot.restriction,
                ManaRestrictionView::CreatureSpellOrActivation(_)
            ) && lot.amounts.iter().any(|&amount| amount > 0)
        });
        giada_entry |= after.objects.iter().any(|object| {
            object.zone == Zone::Battlefield
                && object.name() != "Giada, Font of Hope"
                && object.has_subtype("Angel")
                && object.counters.get(CounterKind::PlusOne) > 0
                && before
                    .objects
                    .iter()
                    .any(|prior| prior.id == object.id && prior.zone == Zone::Stack)
                && before.objects.iter().any(|source| {
                    source.zone == Zone::Battlefield
                        && source.name() == "Giada, Font of Hope"
                        && source.controller == object.controller
                })
        });
        let resolving = before.resolving.as_ref().or_else(|| before.stack.last());
        ventmaw_attack_mana |= resolving.is_some_and(|item| {
            item.kind != StackKind::Spell
                && item.source_card.name() == "Savage Ventmaw"
                && retained(after, item.controller) == retained(before, item.controller) + 6
        });
        if before.turn == after.turn && before.step != after.step {
            for player in [rustmage::Player::P0, rustmage::Player::P1] {
                retained_across_step |= retained(before, player) > 0
                    && retained(before, player) == retained(after, player);
            }
        }
    };
    run_matrix_actions_observed(
        &[angels, dragons],
        &[
            "Giada, Font of Hope",
            "Carnelian Orb of Dragonkind",
            "Savage Ventmaw",
        ],
        &["Secluded Courtyard"],
        &[
            "Giada, Font of Hope",
            "Carnelian Orb of Dragonkind",
            "Secluded Courtyard",
        ],
        &[],
        &[7, 41],
        Some(&mut observe),
    );
    assert!(orb_haste, "no Orb-paid Dragon entered with haste");
    assert!(giada_entry, "no Angel entered with Giada counters");
    assert!(
        courtyard_restricted_mana,
        "no Courtyard chosen-type mana produced"
    );
    assert!(
        ventmaw_attack_mana,
        "no Ventmaw attack produced its six retained mana"
    );
    assert!(
        retained_across_step,
        "no Ventmaw mana survived a step boundary"
    );
    assert!(
        owned_lot_replays > 0,
        "no owned replay with actual mana lots"
    );
    println!(
        "FDN four-mana acceptance: Orb haste, Giada counters, Courtyard restriction and Ventmaw retention; {owned_lot_replays} owned lot-state replays"
    );
}
