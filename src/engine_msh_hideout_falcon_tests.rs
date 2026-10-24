//! Paid Hideout/Falcon rules and exact Redwing choices. All actions replay raw/index/restore.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
pub(super) fn mana(card: CardId, ability: u16, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
pub(super) fn finish(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() && !matches!(g.state.pending, Pending::ManaLotSpend { .. })
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseManaLotSpend { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment {:?}", g.state.pending);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..90 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
pub(super) fn hideout(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Villainous Hideout", Zone::Hand);
    checked(g, Action::PlayLand { card: id });
    id
}
pub(super) fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    // Ordinary owned layer-four effect represents a synthetic Villain producer.
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    assert!(timestamp > 0);
    let rows = &mut g.state.roles.get_or_insert_default().creature_types;
    rows.retain(|row| row.recipient != recipient);
    rows.push(subtypes::CreatureTypesEffectView {
        recipient,
        source: recipient,
        types: names,
        additional: false,
        timestamp,
    });
}
fn tokens(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.name() == name)
                .then_some(id as CardId)
        })
        .collect()
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}
fn connive(g: &mut Game, source: CardId, target: CardId) {
    checked(g, activation(source, 2));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
}

#[test]
fn msh_hideout_all_colors_pay_noncreature_kindred_villain_activation_and_colorless_is_unrestricted()
{
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let h = hideout(&mut g, p);
            let source = paid(&mut g, p, "Clachan Festival");
            types(&mut g, source, &["Villain"]);
            assert_eq!(g.current_kinds(source), 160);
            assert!(!g.is_creature(source));
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
            reject(&mut g, mana(h, 1, 5));
            checked(&mut g, mana(h, 1, color));
            assert_eq!(
                g.mana_lots(p)[0].quality.restriction,
                ManaRestrictionView::SubtypeSpellOrActivation("Villain")
            );
            checked(&mut g, activation(source, 0));
            finish(&mut g);
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Kithkin Token").len(), 3);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(h).tapped);
            reject(&mut g, mana(h, 0, 5));
        }
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, mana(h, 0, 5));
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        cast(&mut g, boots, None, &[]);
        settle(&mut g);
        assert!(g.mana_lots(p).is_empty());
        assert_eq!(g.object_view(boots).zone, Zone::Battlefield);
    }
}

#[test]
fn msh_hideout_actual_spell_funding_rejects_nonvillain_and_survives_producer_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let recruit = named(&mut g, p, "Red Room Recruit", Zone::Hand);
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Hand);
        // Untapped producer participates in speculative spell funding before actual production.
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: recruit })
        );
        checked(&mut g, mana(h, 1, 2));
        g.move_object(h, Zone::Graveyard);
        reject(&mut g, Action::AnnounceCast { card: boots });
        cast(&mut g, recruit, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(recruit).zone, Zone::Battlefield);
        assert!(g.mana_lots(p).is_empty());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn msh_hideout_kindred_activation_departure_captures_villain_and_triggered_ward_is_ineligible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        let source = paid(&mut g, p, "Morcant's Eyes");
        types(&mut g, source, &["Villain"]);
        named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
        checked(&mut g, mana(h, 1, 4));
        checked(&mut g, activation(source, 0));
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        let item = g.state.stack.last().unwrap();
        let quality = g.mana_lots(p).first().map(|v| v.quality);
        assert!(quality.is_none());
        assert!(matches!(
            g.purpose_for_item(item),
            mana_lots::Purpose::Activation(_)
        ));
        settle(&mut g);
        // Sacrificed Kindred Elf regains its printed Elf subtype in the graveyard.
        assert_eq!(tokens(&g, p, "Elf Token").len(), 2);
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        let target = named(&mut g, p.other(), "Colorstorm Stallion", Zone::Battlefield);
        types(&mut g, target, &["Villain"]);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, mana(h, 1, 1));
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(target)]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 0);
        assert_eq!(g.mana_lots(p).len(), 1);
        next_turn(&mut g);
        assert!(g.mana_lots(p).is_empty());
    }
}

#[test]
fn msh_hideout_connive_targets_kindred_permanent_private_discard_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for nonland in [false, true] {
            let mut g = setup(p);
            let h = hideout(&mut g, p);
            let target = paid(&mut g, p, "Clachan Festival");
            types(&mut g, target, &["Villain"]);
            paid(&mut g, p, "Doubling Season");
            let enemy = named(&mut g, p.other(), "Red Room Recruit", Zone::Battlefield);
            let ordinary = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let hidden = named(&mut g, p, "Red Room Recruit", Zone::Hand);
            let grave = named(&mut g, p, "Red Room Recruit", Zone::Graveyard);
            let drawn = named(
                &mut g,
                p,
                if nonland { "Shock" } else { "Forest" },
                Zone::Library,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            checked(&mut g, activation(h, 2));
            for id in [enemy, ordinary, hidden, grave] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(h).tapped);
            g.move_object(h, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert!(visible(&g, p, drawn));
            assert!(!visible(&g, p.other(), drawn));
            reject(&mut g, Action::Discard { card: enemy });
            checked(&mut g, Action::Discard { card: drawn });
            settle(&mut g);
            assert_eq!(
                g.counter_count(target, CounterKind::PlusOne),
                if nonland { 2 } else { 0 }
            );
            assert_eq!(g.object_view(drawn).zone, Zone::Graveyard);
            assert!(visible(&g, p.other(), drawn));
            assert!(!visible(&g, p.other(), hidden));
        }
    }
}

#[test]
fn msh_hideout_connive_timing_and_stale_or_changed_targets_fail_atomically() {
    for p in [Player::P0, Player::P1] {
        for change in 0..4 {
            let mut g = setup(p);
            let h = hideout(&mut g, p);
            let target = named(&mut g, p, "Red Room Recruit", Zone::Battlefield);
            g.state.active = p.other();
            reject(&mut g, activation(h, 2));
            g.state.active = p;
            let opt = named(&mut g, p, "Opt", Zone::Hand);
            cast(&mut g, opt, None, &[]);
            reject(&mut g, activation(h, 2));
            settle(&mut g);
            let hand = g.hand_count(p);
            connive(&mut g, h, target);
            match change {
                0 => types(&mut g, target, &["Hero"]),
                1 => {
                    let r = g.object_ref(target);
                    g.change_controller(r, p.other());
                }
                2 => {
                    g.move_object(target, Zone::Hand);
                    g.move_object(target, Zone::Battlefield);
                }
                _ => {
                    g.move_object(target, Zone::Exile);
                }
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn msh_hideout_connive_captures_ability_controller_and_does_not_counter_reentered_recipient() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        let target = paid(&mut g, p, "Clachan Festival");
        types(&mut g, target, &["Villain"]);
        let drawn = named(&mut g, p, "Shock", Zone::Library);
        connive(&mut g, h, target);
        let r = g.object_ref(h);
        g.change_controller(r, p.other());
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        let old = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        checked(&mut g, Action::Discard { card: drawn });
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        assert_eq!(g.object_view(drawn).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_falcon_paid_entry_exact_redwing_recipe_flying_and_summoning_sickness() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let falcon = paid(&mut g, p, "Falcon, Winged Wonder");
        assert_eq!(
            (g.object_view(falcon).power, g.object_view(falcon).toughness),
            (3, 4)
        );
        let red = tokens(&g, p, "Redwing");
        assert_eq!(red.len(), 1);
        let red = red[0];
        let v = g.object_view(red);
        assert_eq!((v.power, v.toughness, v.kinds), (1, 1, 1));
        assert_eq!(v.colors, 2);
        assert!(g.state.objects[red as usize].card.is_legendary());
        assert!(g.has_subtype(red, "Bird") && g.has_subtype(red, "Scout"));
        assert!(
            g.keywords(falcon).contains(Keywords::FLYING)
                && g.keywords(red).contains(Keywords::FLYING)
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: red });
        reject(&mut g, Action::Attack { card: falcon });
    }
}

#[test]
fn msh_falcon_departure_and_control_change_keep_etb_controller_with_doubled_legend_choices() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, "Doubling Season");
            let falcon = named(&mut g, p, "Falcon, Winged Wonder", Zone::Hand);
            cast(&mut g, falcon, None, &[]);
            until(&mut g, |g| g.object_view(falcon).zone == Zone::Battlefield);
            if depart {
                g.move_object(falcon, Zone::Exile);
            } else {
                let r = g.object_ref(falcon);
                g.change_controller(r, p.other());
            }
            until(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::KeepLegend { .. }))
            });
            let red = tokens(&g, p, "Redwing");
            assert_eq!(red.len(), 2);
            assert!(tokens(&g, p.other(), "Redwing").is_empty());
            let keep = g.object_ref(red[1]);
            let invalid = g.object_ref(falcon);
            reject(&mut g, Action::KeepLegend { object: invalid });
            checked(&mut g, Action::KeepLegend { object: keep });
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Redwing"), vec![red[1]]);
        }
    }
}

#[test]
fn msh_falcon_redwing_attack_surveil_is_private_optional_and_frozen_to_trigger_controller() {
    for p in [Player::P0, Player::P1] {
        for grave in [false, true] {
            let mut g = setup(p);
            let falcon = paid(&mut g, p, "Falcon, Winged Wonder");
            let red = tokens(&g, p, "Redwing")[0];
            let top = named(&mut g, p, "Shock", Zone::Library);
            let wrong = named(&mut g, p, "Forest", Zone::Hand);
            g.state.objects[red as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: red });
            checked(&mut g, Action::FinishAttackers);
            let old = g.object_ref(red);
            g.change_controller(old, p.other());
            g.move_object(falcon, Zone::Exile);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });
            assert!(g.observe(p).library_choice.as_ref().unwrap().surveil);
            assert!(g.observe(p.other()).library_choice.is_none());
            assert_eq!(
                g.observe(p).library_choice.as_ref().unwrap().remaining[0]
                    .reference
                    .card,
                top
            );
            assert!(!visible(&g, p.other(), top));
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: wrong,
                    destination: LibraryDestination::Graveyard,
                },
            );
            g.move_object(red, Zone::Graveyard);
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: if grave {
                        LibraryDestination::Graveyard
                    } else {
                        LibraryDestination::Top
                    },
                },
            );
            assert_eq!(
                g.object_view(top).zone,
                if grave {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(visible(&g, p.other(), top), grave);
        }
    }
}

#[test]
fn msh_falcon_redwing_can_be_blocked_only_by_flying_or_reach_and_empty_surveil_finishes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Falcon, Winged Wonder");
        let red = tokens(&g, p, "Redwing")[0];
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let reach = named(&mut g, p.other(), "Magnigoth Sentry", Zone::Battlefield);
        let library = g.state.players[p.index()].library.clone();
        for id in library {
            g.move_object(id, Zone::Graveyard);
        }
        g.state.objects[red as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: red });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: red,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: reach,
                attacker: red,
            },
        );
        assert!(g.observe(p).library_choice.is_none());
    }
}
