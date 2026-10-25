//! Whole-card paid proofs; only the owned three may use gated physical setup.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const INC: &str = "Multiversal Incursion";
const JUSTICE: &str = "Justice, Vance Astrovik";
const LOKI: &str = "Loki, God of Mischief";
fn prepared(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, Zone::Hand);
    }
    assert!([INC, JUSTICE, LOKI].contains(&name));
    // Explicit staged physical cards remain outside accepted deck construction.
    // Their immutable definitions and ordinary checked paid actions are real.
    let id = CardId::try_from(g.state.objects.len()).unwrap();
    g.state.objects.push(Object {
        card,
        copy_kind: CopyKind::None,
        owner: p,
        controller: p,
        zone: Zone::Hand,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        attached_to: None,
    });
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished announcement");
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = prepared(g, p, name);
    let c = g.state.objects[id as usize].card;
    let (mana, generic) = c.cost();
    g.state.players[p.index()].mana = mana;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..250 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn next_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    until(g, |g| {
        g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1
    });
}
fn justice_choice(g: &mut Game, id: CardId) {
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == id && a.item.kind == StackKind::TriggeredAbility { ability: 0 }
        }) && matches!(g.state.pending, Pending::Targets { .. })
    });
}
fn choose_justice(g: &mut Game, id: CardId, target: Option<CardId>) {
    justice_choice(g, id);
    if let Some(card) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    finish(g);
}
fn copies(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_copy_token()
                && g.object_view(i as CardId).name() == name)
                .then_some(i as CardId)
        })
        .collect()
}
fn count(g: &Game, id: CardId) -> u64 {
    g.counter_count(id, CounterKind::PlusOne)
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn equip(g: &mut Game, p: Player, pole: CardId, bear: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    announce(g, pole, 0);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(g);
    settle(g);
}
#[test]
fn msh_three_cards_justice_paid_optional_entry_restrictions_self_owner_and_counter() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..4 {
            let mut g = setup(p);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            let foreign = named(&mut g, p.other(), "Fishing Pole", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let token = named(&mut g, p, "@token/msh-moloid", Zone::Battlefield);
            let id = packet(&mut g, p, JUSTICE, &[]);
            justice_choice(&mut g, id);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(g.keywords(id).contains(Keywords::FLYING));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(token),
                },
            );
            let target = match choice {
                0 => None,
                1 => Some(id),
                2 => Some(bear),
                _ => Some(foreign),
            };
            let before = g.snapshot();
            let mut orphan = {
                let parent = g.clone();
                parent.clone()
            };
            if let Some(card) = target {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
                checked(
                    &mut orphan,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            finish(&mut g);
            finish(&mut orphan);
            settle(&mut g);
            settle(&mut orphan);
            assert_eq!(g.snapshot(), orphan.snapshot());
            assert_eq!(
                g.object_view(id).zone,
                if choice == 1 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(count(&g, id), u64::from(choice == 2));
            if let Some(card) = target {
                assert_eq!(g.object_view(card).zone, Zone::Hand);
                assert!(
                    g.observe(g.state.objects[card as usize].owner)
                        .objects
                        .iter()
                        .any(|o| o.id == card)
                );
            }
            orphan.restore(&before).unwrap();
            if let Some(card) = target {
                checked(
                    &mut orphan,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            finish(&mut orphan);
            settle(&mut orphan);
            assert_eq!(g.object_view(id), orphan.object_view(id));
        }
    }
}
#[test]
fn msh_three_cards_justice_paid_flying_changes_actual_block_legality_and_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let justice = packet(&mut g, p, JUSTICE, &[]);
        choose_justice(&mut g, justice, None);
        settle(&mut g);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        next_main(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        let before = g.state.players[p.other().index()].life;
        checked(&mut g, Action::Attack { card: justice });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: justice,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.players[p.other().index()].life < before);
        assert_eq!(g.state.players[p.other().index()].life, before - 2);
    }
}
#[test]
fn msh_three_cards_justice_paid_queued_entry_source_and_target_departure() {
    for p in [Player::P0, Player::P1] {
        for remove_source in [false, true] {
            let mut g = setup(p);
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            let justice = packet(&mut g, p, JUSTICE, &[]);
            choose_justice(&mut g, justice, Some(bear));
            let old = g.object_ref(justice);
            let removed = if remove_source { justice } else { bear };
            paid(&mut g, p, "Unsummon", &[Target::Card(removed)]);
            assert_eq!(g.object_view(bear).zone, Zone::Hand);
            assert_eq!(
                g.object_view(justice).zone,
                if remove_source {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                count(&g, justice),
                u64::from(!remove_source),
                "only actual other return counts; stale entry target does nothing"
            );
            if remove_source {
                assert_ne!(g.object_ref(justice), old);
            }
        }
    }
}
#[test]
fn msh_three_cards_loki_paid_cast_observes_real_justice_trigger_targets_before_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI, &[]);
        assert_eq!(
            g.stats(loki),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let justice = packet(&mut g, p, JUSTICE, &[]);
        justice_choice(&mut g, justice);
        let before = g.hand_count(p);
        let top = *g.state.players[p.index()].library.last().unwrap();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        until(&mut g, |g| g.hand_count(p) == before + 1);
        assert_eq!(
            g.object_view(bear).zone,
            Zone::Battlefield,
            "Loki trigger resolves before causing triggered ability"
        );
        assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == top));
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!(count(&g, justice), 1);
        assert_eq!(g.hand_count(p), before + 2);
    }
}
#[test]
fn msh_three_cards_loki_paid_raw_activation_and_each_players_turn_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, LOKI, &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let before = g.state.players[p.index()].library.len();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let raw=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,ability:0,target:Some(Target::Card(t)),..} if *card==pole&&*t==bear)).unwrap();
        checked(&mut g, raw);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        equip(&mut g, p, pole, bear);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        let source = named(&mut g, p, "Bullseye, Death Dealer", Zone::Battlefield);
        next_turn(&mut g);
        for turn in 0..2 {
            if turn == 1 {
                next_main(&mut g, p);
            }
            priority(&mut g, p);
            let discard = named(&mut g, p, "Bear Cub", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            let library = g.state.players[p.index()].library.len();
            announce(&mut g, source, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            let object = g.object_ref(discard);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - 1,
                "once refreshes on opponent turn as well as controller turn"
            );
        }
    }
}
#[test]
fn msh_three_cards_loki_paid_source_return_keeps_queue_and_recast_is_new_instance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI, &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let original = g.object_ref(loki);
        let library = g.state.players[p.index()].library.len();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        paid(&mut g, p, "Unsummon", &[Target::Card(loki)]);
        assert_eq!(g.object_view(loki).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, loki, None, &[]);
        settle(&mut g);
        assert_ne!(g.object_ref(loki), original);
        equip(&mut g, p, pole, bear);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            library - 2,
            "new incarnation has independent once history"
        );
    }
}
#[test]
fn msh_three_cards_loki_countered_trigger_still_consumes_once_bounded_counter_boundary() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI, &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let library = g.state.players[p.index()].library.len();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == loki && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        });
        let trigger = g
            .state
            .stack
            .iter()
            .find(|s| s.card == loki && matches!(s.kind, StackKind::TriggeredAbility { .. }))
            .unwrap()
            .id;
        // No admitted ability-countering producer exists. Only this direct
        // production counter boundary is bounded; cast, acquisition and costs
        // above are paid actual actions. No unsupported card is executed.
        assert!(g.counter_stack_item(trigger, false));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        equip(&mut g, p, pole, bear);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            library,
            "the ability triggered even though its packet was countered"
        );
    }
}
#[test]
fn msh_three_cards_incursion_exact_seven_mana_and_sorcery_timing_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let inc = prepared(&mut g, p, INC);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        reject(&mut g, Action::AnnounceCast { card: inc });
        next_turn(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
        reject(&mut g, Action::AnnounceCast { card: inc });
        next_main(&mut g, p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
        cast(&mut g, inc, None, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(inc).zone, Zone::Graveyard);
        assert_eq!(copies(&g, p, "Bear Cub").len(), 1);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_three_cards_incursion_uses_resolution_time_creatures_after_paid_flash_and_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let inc = packet(&mut g, p, INC, &[]);
        let unsummon = packet(&mut g, p, "Unsummon", &[Target::Card(bear)]);
        until(&mut g, |g| g.object_view(unsummon).zone == Zone::Graveyard);
        assert_eq!(g.object_view(inc).zone, Zone::Stack);
        let fresh = packet(&mut g, p, "Brineborn Cutthroat", &[]);
        until(&mut g, |g| g.object_view(fresh).zone == Zone::Battlefield);
        assert_eq!(g.object_view(inc).zone, Zone::Stack);
        settle(&mut g);
        assert!(copies(&g, p, "Bear Cub").is_empty());
        assert_eq!(copies(&g, p, "Brineborn Cutthroat").len(), 1);
    }
}
#[test]
fn msh_three_cards_incursion_copies_no_counters_tap_grants_or_external_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = paid(&mut g, p, "Llanowar Elves", &[]);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(elf)]);
        next_main(&mut g, p);
        // Reapply the actual temporary grant this turn, then tap the printed
        // mana ability; both counter values and tap state are noncopiable.
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(elf)]);
        checked(&mut g, Action::TapMana { card: elf });
        assert!(g.object_view(elf).tapped);
        let land = named(&mut g, p, "Soulstone Sanctuary", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        announce(&mut g, land, 1);
        finish(&mut g);
        settle(&mut g);
        assert!(g.is_creature(land));
        packet(&mut g, p, INC, &[]);
        settle(&mut g);
        let copy = copies(&g, p, "Llanowar Elves")[0];
        assert_eq!(
            g.stats(copy),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(count(&g, copy), 0);
        assert!(!g.object_view(copy).tapped);
        assert!(!g.keywords(copy).contains(Keywords::HEXPROOF));
        let land_copy = copies(&g, p, "Soulstone Sanctuary")[0];
        assert!(!g.is_creature(land_copy));
        assert_eq!(g.object_view(land_copy).kinds, 2);
        assert!(!g.keywords(land_copy).contains(Keywords::VIGILANCE));
        assert!(g.state.land_played);
        assert_eq!(
            g.state.continuous_land_plays_used, 0,
            "creation is not another land play"
        );
    }
}
