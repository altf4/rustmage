//! Whole-card ECL rules scenarios using paid actions and owned indexed branches.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::{cast, spawn};
use super::*;
use crate::ChoiceColor;

fn mare(g: &mut Game, p: Player) -> CardId {
    let id = spawn(g, p, Card::from_name("Loch Mare").unwrap(), Zone::Hand);
    g.state.players[p.index()].mana[1] = 20;
    cast(g, id, None);
    settle(g);
    assert_eq!(g.object_view(id).counters.minus_one, 3);
    id
}
fn activate(g: &mut Game, source: CardId, ability_index: u16, target_id: Option<CardId>) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a,
        Action::Activate {card, ability, target: None, ..} | Action::BeginActivate {card, ability, ..}
        if *card == source && *ability == ability_index))
        .expect("paid activation available");
    checked(g, action);
    if let Some(id) = target_id {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
}

#[test]
fn loch_mare_paid_draw_tap_stun_and_no_counter_legality() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = mare(&mut g, player);
        let target = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == player && o.zone == Zone::Hand)
            .count();
        activate(&mut g, source, 0, None);
        assert_eq!(g.object_view(source).counters.minus_one, 2);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == player && o.zone == Zone::Hand)
                .count(),
            hand + 1
        );
        activate(&mut g, source, 1, Some(target));
        assert_eq!(g.object_view(source).counters.minus_one, 0);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert_eq!(g.object_view(target).counters.stun, 1);
        assert!(
            !g.legal_actions().iter().any(|a| matches!(a,
            Action::Activate { card, .. } | Action::BeginActivate { card, .. } if *card == source))
        );
    }
}

#[test]
fn loch_mare_mixed_counter_payment_atomicity_and_departed_source() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    g.set_counters(
        source,
        Counters::of(CounterKind::Stun, 1).with(CounterKind::Soul, 1),
    );
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::BeginActivate {card, ability: 1, counter_payment, ..}
        if *card == source && counter_payment.get(CounterKind::Stun) == 1
            && counter_payment.get(CounterKind::Soul) == 1)
        })
        .unwrap();
    let mut forged = action.clone();
    if let Action::BeginActivate {
        counter_payment, ..
    } = &mut forged
    {
        *counter_payment = Counters::of(CounterKind::Page, 2);
    }
    let before = g.snapshot();
    assert_eq!(g.apply(Player::P0, forged), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    checked(&mut g, action);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    g.move_object(source, Zone::Graveyard);
    settle(&mut g);
    assert!(g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 1);
}

#[test]
fn loch_mare_invalid_target_keeps_paid_cost_and_does_not_stun_reentry() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    activate(&mut g, source, 1, Some(target));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 0);
    assert_eq!(g.object_view(source).counters.minus_one, 1);
}

fn cast_named(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    cast(g, id, None);
    settle(g);
    id
}

#[test]
fn encumbered_reejerey_removes_on_tap_only_while_counter_exists() {
    let mut g = setup(Player::P0);
    let id = cast_named(&mut g, Player::P0, "Encumbered Reejerey");
    assert_eq!(g.object_view(id).counters.minus_one, 3);
    for left in (0..3).rev() {
        g.tap_objects(&[id]);
        settle(&mut g);
        assert_eq!(g.object_view(id).counters.minus_one, left);
        // Becoming tapped requires an actual untapped-to-tapped event.
        g.tap_objects(&[id]);
        assert!(g.state.pending_triggers.is_empty());
        g.untap_objects(&[id], untap_rules::UntapCause::Effect);
    }
    g.tap_objects(&[id]);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(
        g.stats(id),
        Stats {
            power: 5,
            toughness: 4
        }
    );
}

#[test]
fn reluctant_dounguard_qualifies_only_other_owned_entries_and_exact_source() {
    let mut g = setup(Player::P0);
    let id = cast_named(&mut g, Player::P0, "Reluctant Dounguard");
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    let other = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Hand);
    g.move_object(other, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    assert_eq!(g.object_view(id).counters.minus_one, 1);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    g.prepare_triggers();
    // The queued trigger cannot remove a counter from a new incarnation.
    g.move_object(id, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(id).counters.minus_one, 2);
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    cast_named(&mut g, Player::P0, "Grizzly Bears");
    assert_eq!(g.object_view(id).counters.minus_one, 0);
    let last = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(last, Zone::Battlefield);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn gallant_fowlknight_boosts_resolution_group_and_expires_at_cleanup() {
    let mut g = setup(Player::P0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let fowl = cast_named(&mut g, Player::P0, "Gallant Fowlknight");
    assert_eq!(g.stats(bear).power, 3);
    assert_eq!(g.stats(enemy).power, 2);
    assert_eq!(g.stats(fowl).power, 4);
    assert!(g.keywords(fowl).contains(Keywords::FIRST_STRIKE));
    assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    let late = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Hand);
    g.move_object(late, Zone::Battlefield);
    assert_eq!(g.stats(late).power, 2);
    g.move_object(fowl, Zone::Graveyard);
    assert_eq!(g.stats(bear).power, 3);
    g.finish_cleanup();
    assert_eq!(g.stats(bear).power, 2);
}

fn spell(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    super::ability_timeline_tests::cast(g, id, None, targets);
    id
}
fn changelings(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .count()
}

#[test]
fn rime_chill_vivid_cost_zero_targets_and_partial_target_failure() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        spawn(&mut g, player, Card::GrizzlyBears, Zone::Battlefield);
        spawn(
            &mut g,
            player,
            Card::from_name("Gallant Fowlknight").unwrap(),
            Zone::Battlefield,
        );
        // Two colors, not two permanents or colored mana symbols: {6}{U} becomes {4}{U}.
        g.state.players[player.index()].mana = [0, 5, 0, 0, 0, 0];
        let hand = g.hand_count(player);
        spell(&mut g, player, "Rime Chill", &[]);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(player), hand + 1);
        g.state.players[player.index()].mana = [0, 5, 0, 0, 0, 0];
        let a = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        let b = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        spell(
            &mut g,
            player,
            "Rime Chill",
            &[Target::Card(a), Target::Card(b)],
        );
        g.move_object(a, Zone::Hand);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
        assert_eq!(g.object_view(b).counters.stun, 1);
        assert_eq!(g.hand_count(player), hand + 2);
    }
}

#[test]
fn rime_chill_all_illegal_targets_fizzles_draw() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let before = g.hand_count(Player::P0);
    spell(&mut g, Player::P0, "Rime Chill", &[Target::Card(target)]);
    g.move_object(target, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), before);
}

#[test]
fn tributary_vaulter_restricts_targets_and_boost_survives_source_departure() {
    let mut g = setup(Player::P0);
    let source = cast_named(&mut g, Player::P0, "Tributary Vaulter");
    let recipient = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Silvergill Peddler").unwrap(),
        Zone::Battlefield,
    );
    let enemy = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Silvergill Peddler").unwrap(),
        Zone::Battlefield,
    );
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    g.tap_objects(&[source]);
    g.prepare_triggers();
    if let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(&mut g, action);
    }
    for bad in [source, enemy, bear] {
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bad)
        }));
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    g.move_object(source, Zone::Graveyard);
    let base = g.stats(recipient).power;
    settle(&mut g);
    assert_eq!(g.stats(recipient).power, base + 2);
    g.finish_cleanup();
    assert_eq!(g.stats(recipient).power, base);
}

#[test]
fn personify_returns_foreign_owned_creature_and_creates_real_changeling() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.record_control(g.object_ref(target), Player::P0, None);
    g.add_counter(target, CounterKind::PlusOne, 2);
    let old = g.object_ref(target);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(target)]);
    settle(&mut g);
    let view = g.object_view(target);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!(view.controller, Player::P1);
    assert_ne!(view.reference, old);
    assert!(view.counters.is_empty());
    assert_eq!(changelings(&g, Player::P0), 1);
    let token = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .unwrap() as CardId;
    for subtype in ["Merfolk", "Kithkin", "Coward", "Elf"] {
        assert!(g.object_view(token).has_subtype(subtype));
    }
    assert_eq!(
        g.stats(token),
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert_eq!(g.object_view(token).colors, 0);
}

#[test]
fn personify_tokens_do_not_return_and_invalid_targets_make_no_token() {
    let mut g = setup(Player::P0);
    g.create_tokens(
        Player::P0,
        Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap(),
        1,
        false,
    );
    let token = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    == Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap()
        })
        .unwrap() as CardId;
    let old = g.object_ref(token);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(token)]);
    settle(&mut g);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_ne!(g.object_ref(token), old);
    let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Personify", &[Target::Card(target)]);
    g.move_object(target, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(changelings(&g, Player::P0), 1);
}

#[test]
fn crib_swap_tracks_resolution_controller_and_invalid_incarnation() {
    let mut g = setup(Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let crib = spell(&mut g, Player::P0, "Crib Swap", &[Target::Card(target)]);
    assert!(Card::from_name("Crib Swap").unwrap().has_subtype("Merfolk"));
    g.record_control(g.object_ref(target), Player::P0, None);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_eq!(changelings(&g, Player::P1), 0);
    assert_eq!(g.object_view(crib).zone, Zone::Graveyard);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    spell(&mut g, Player::P0, "Crib Swap", &[Target::Card(target)]);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(changelings(&g, Player::P0), 1);
    assert_eq!(changelings(&g, Player::P1), 0);
}

#[test]
fn personify_finishes_returned_entry_choice_before_creating_token() {
    let mut g = setup(Player::P0);
    let mare = spawn(
        &mut g,
        Player::P0,
        Card::from_name("Diamond Mare").unwrap(),
        Zone::Battlefield,
    );
    spell(&mut g, Player::P0, "Personify", &[Target::Card(mare)]);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    assert_eq!(changelings(&g, Player::P0), 0);
    // Dispose the source branch while the returned card's own entry is suspended.
    let mut g = {
        let child = g.clone();
        drop(g);
        child
    };
    checked(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    settle(&mut g);
    assert_eq!(
        g.object_view(mare).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Blue))
    );
    assert_eq!(changelings(&g, Player::P0), 1);
}

#[test]
fn noggle_preserves_artifact_type_removes_abilities_color_and_creature_types() {
    let mut g = setup(Player::P0);
    let target = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Diamond Mare").unwrap(),
        Zone::Battlefield,
    );
    g.add_counter(target, CounterKind::PlusOne, 2);
    // Flash works during an opponent's main phase.
    g.state.active = Player::P1;
    spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(target)],
    );
    settle(&mut g);
    let aura = g
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield && o.card == Card::from_name("Noggle the Mind").unwrap()
        })
        .unwrap() as CardId;
    let view = g.object_view(target);
    assert_eq!(view.kinds & 17, 17);
    assert_eq!(view.colors, 0);
    assert!(view.has_subtype("Noggle"));
    assert!(!view.has_subtype("Horse"));
    assert_eq!(
        g.stats(target),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    assert!(!g.printed_abilities_available(target));
    g.move_object(aura, Zone::Graveyard);
    assert!(g.object_view(target).has_subtype("Horse"));
    assert!(g.printed_abilities_available(target));
    assert_eq!(
        g.stats(target),
        Stats {
            power: 3,
            toughness: 5
        }
    );
}

fn hold_target(g: &mut Game, target: CardId) -> CardId {
    let source = spell(g, Player::P0, "Liminal Hold", &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        checked(g, a);
    }
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    source
}
#[test]
fn liminal_hold_optional_target_life_and_owner_return() {
    let mut g = setup(Player::P0);
    cast_named(&mut g, Player::P0, "Liminal Hold");
    assert_eq!(g.state.players[0].life, 22);
    let target = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.record_control(g.object_ref(target), Player::P1, None);
    let source = hold_target(&mut g, target);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    assert_eq!(g.state.players[0].life, 24);
    g.move_object(source, Zone::Graveyard);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.object_view(target).controller, Player::P0);
}
#[test]
fn liminal_hold_source_departure_still_gains_life_but_invalid_target_fizzles() {
    for source_departure in [false, true] {
        let mut g = setup(Player::P0);
        let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        let source = hold_target(&mut g, target);
        if source_departure {
            g.move_object(source, Zone::Graveyard);
        } else {
            g.move_object(target, Zone::Hand);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if source_departure {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        assert_eq!(
            g.state.players[0].life,
            if source_departure { 22 } else { 20 }
        );
    }
}

#[test]
fn noggle_removes_colored_flying_vigilance_and_reverses_on_aura_departure() {
    let mut g = setup(Player::P0);
    let angel = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Serra Angel").unwrap(),
        Zone::Battlefield,
    );
    assert_ne!(g.object_view(angel).colors, 0);
    assert!(g.keywords(angel).contains(Keywords::FLYING));
    let aura = spell(
        &mut g,
        Player::P0,
        "Noggle the Mind",
        &[Target::Card(angel)],
    );
    settle(&mut g);
    assert_eq!(g.object_view(angel).colors, 0);
    assert_eq!(
        g.stats(angel),
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert!(
        !g.keywords(angel)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    g.move_object(aura, Zone::Graveyard);
    assert_ne!(g.object_view(angel).colors, 0);
    assert_eq!(
        g.stats(angel),
        Stats {
            power: 4,
            toughness: 4
        }
    );
    assert!(
        g.keywords(angel)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
}

fn guardian_setup() -> (Game, CardId) {
    let mut g = setup(Player::P1);
    g.state.priority = Player::P0;
    let source = cast_named(&mut g, Player::P0, "Glen Elendra Guardian");
    assert_eq!(g.object_view(source).counters.minus_one, 1);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    assert_eq!(
        g.stats(source),
        Stats {
            power: 2,
            toughness: 3
        }
    );
    (g, source)
}
fn guardian_counter(g: &mut Game, source: CardId, victim: CardId) {
    checked(g, Action::Pass);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::BeginActivate {card, ability: 0, ..} if *card == source)
        })
        .unwrap();
    checked(g, action);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Spell(victim),
        },
    );
}
#[test]
fn glen_elendra_guardian_flash_paid_counter_draw_and_source_departure() {
    for departure in [false, true] {
        let (mut g, source) = guardian_setup();
        let bolt = spell(
            &mut g,
            Player::P1,
            "Lightning Bolt",
            &[Target::Player(Player::P0)],
        );
        let hand = g.hand_count(Player::P1);
        guardian_counter(&mut g, source, bolt);
        assert_eq!(g.object_view(source).counters.minus_one, 0);
        if departure {
            g.move_object(source, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(g.object_view(bolt).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].life, 20);
        assert_eq!(g.hand_count(Player::P1), hand + 1);
    }
}
#[test]
fn glen_elendra_guardian_any_named_counter_pays_and_uncounterable_still_draws() {
    let (mut g, source) = guardian_setup();
    g.set_counters(source, Counters::of(CounterKind::Soul, 1));
    spawn(
        &mut g,
        Player::P1,
        Card::from_name("Sphinx of the Final Word").unwrap(),
        Zone::Battlefield,
    );
    let bolt = spell(
        &mut g,
        Player::P1,
        "Lightning Bolt",
        &[Target::Player(Player::P0)],
    );
    let hand = g.hand_count(Player::P1);
    guardian_counter(&mut g, source, bolt);
    assert!(g.object_view(source).counters.is_empty());
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P1), hand + 1);
    assert_eq!(g.state.players[0].life, 17);
}
#[test]
fn glen_elendra_guardian_cannot_target_creature_spells() {
    let (mut g, source) = guardian_setup();
    spell(&mut g, Player::P1, "Grizzly Bears", &[]);
    checked(&mut g, Action::Pass);
    assert!(!g.legal_actions().iter().any(|a| matches!(a,
        Action::BeginActivate {card, ..} | Action::Activate {card, ..} if *card == source)));
}

#[test]
fn immediate_exile_programs_wait_for_departure_linked_entry_choices() {
    for name in ["Personify", "Crib Swap"] {
        let mut g = setup(Player::P0);
        let returning = spawn(
            &mut g,
            Player::P1,
            Card::from_name("Diamond Mare").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[returning as usize].tapped = true;
        let exiler = cast_named(&mut g, Player::P0, "Henchbots");
        assert_eq!(g.object_view(returning).zone, Zone::Exile);
        spell(&mut g, Player::P0, name, &[Target::Card(exiler)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(
            g.state.pending,
            Pending::EntryValue { player: Player::P1 }
        ));
        assert_eq!(g.object_view(exiler).zone, Zone::Exile);
        assert_eq!(changelings(&g, Player::P0), 0);
        assert!(g.state.resolving.as_ref().unwrap().continuation.is_some());
        let snapshot = g.snapshot();
        let bytes = g.state_bytes();
        let mut child = g.clone();
        drop(g);
        child.restore(&snapshot).unwrap();
        assert_eq!(child.state_bytes(), bytes);
        checked(
            &mut child,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Blue),
            },
        );
        settle(&mut child);
        assert_eq!(child.object_view(returning).zone, Zone::Battlefield);
        assert_eq!(
            child.object_view(exiler).zone,
            if name == "Personify" {
                Zone::Battlefield
            } else {
                Zone::Exile
            }
        );
        assert_eq!(changelings(&child, Player::P0), 1);
    }
}
