//! Whole Saga/token scenarios; bounded lore/type fixtures are explicit.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
pub(super) const NAME: &str = "The Coming of Galactus";
pub(super) fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(g.revision(), revision);
}
pub(super) fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
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
                        | Action::ChooseSpellCost { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
pub(super) fn finish(g: &mut Game) {
    for _ in 0..30 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn saga(g: &mut Game, p: Player) -> CardId {
    let s = named(g, p, NAME, Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 2, 0, 1, 2];
    cast(g, s, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    s
}
pub(super) fn tokens(g: &Game, p: Player, card: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == card && o.is_token())
                .then_some(i as CardId)
        })
        .collect()
}
fn chapter(g: &mut Game, s: CardId, ability: u16) {
    until(g, |g| {
        g.state
            .stack
            .iter()
            .any(|i| i.card == s && i.kind == StackKind::TriggeredAbility { ability })
    });
}
fn main_lore(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.begin_step(Step::Main1);
    g.prepare_triggers();
}
pub(super) fn animate(g: &mut Game, id: CardId, preserve: bool) {
    let object = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .animations
        .push(characteristic_layers::AnimationView {
            object,
            timestamp,
            artifact: false,
            preserve_card_types: preserve,
            colors: None,
            creature_types: None,
            all_creature_types: false,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            expires_turn: None,
        });
}
fn iv(g: &mut Game, p: Player) -> CardId {
    let s = saga(g, p);
    // Bounded production counter placement advances to IV; natural turns covered separately.
    g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
    settle(g);
    assert!(!g.on_battlefield(s));
    tokens(g, p, Card::MshGalactusToken)[0]
}
#[test]
fn msh_galactus_paid_natural_four_chapters_exact_token_and_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 1);
        assert!(!g.keywords(s).contains(Keywords::TRAMPLE));
        for n in 2..=4 {
            next_turn(&mut g);
            next_turn(&mut g);
            for _ in 0..100 {
                if g.counter_count(s, CounterKind::Lore) >= n || !g.on_battlefield(s) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - 2 * i128::from(n.min(3) - 1)
            );
            assert_eq!(g.on_battlefield(s), n < 4);
        }
        let t = tokens(&g, p, Card::MshGalactusToken);
        assert_eq!(t.len(), 1);
        let t = t[0];
        let v = g.object_view(t);
        assert_eq!(v.name(), "Galactus");
        assert_eq!((v.power, v.toughness, v.colors, v.kinds), (16, 16, 4, 1));
        assert!(v.card.is_legendary() && v.has_subtype("Elder") && v.has_subtype("Alien"));
        assert_eq!(v.card.fixed_mana_value(), 0);
        assert!(v.summoning_sick && !v.tapped);
        assert!(
            v.keywords
                .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
        );
        assert!(!v.keywords.contains(Keywords::HASTE));
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_galactus_optional_nonland_target_paid_zero_self_or_opponent() {
    for p in [Player::P0, Player::P1] {
        for owner in [None, Some(p), Some(p.other())] {
            let mut g = setup(p);
            let victim = named(
                &mut g,
                owner.unwrap_or(p),
                "Gilded Lotus",
                Zone::Battlefield,
            );
            let s = named(&mut g, p, NAME, Zone::Hand);
            cast(&mut g, s, None, &[]);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == s && a.item.kind != StackKind::Spell)
            });
            if owner.is_some() {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
            }
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.on_battlefield(victim), owner.is_none());
        }
    }
}
#[test]
fn msh_galactus_chapter_one_land_hidden_and_player_choices_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        animate(&mut g, land, true);
        let hidden = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let s = named(&mut g, p, NAME, Zone::Hand);
        cast(&mut g, s, None, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == s && a.item.kind != StackKind::Spell)
        });
        for target in [
            Target::Card(land),
            Target::Card(hidden),
            Target::Player(p.other()),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(!g.on_battlefield(s));
    }
}
#[test]
fn msh_galactus_nonland_indestructible_target_survives_and_blink_stays_exact() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let v = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[v as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            let s = named(&mut g, p, NAME, Zone::Hand);
            cast(&mut g, s, None, &[]);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == s && a.item.kind != StackKind::Spell)
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(v),
                },
            );
            finish(&mut g);
            let old = g.object_ref(v);
            if blink {
                g.move_object(v, Zone::Exile);
                g.move_object(v, Zone::Battlefield);
                assert_ne!(g.object_ref(v), old);
            }
            settle(&mut g);
            assert!(g.on_battlefield(v));
        }
    }
}
#[test]
fn msh_galactus_multiple_thresholds_wait_every_exact_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
        g.prepare_triggers();
        assert!(!g.saga_sacrifice_required(s));
        // All source packets exist together, irrespective of chosen stack ordering.
        assert_eq!(
            g.state.pending_triggers.len()
                + g.state.ordering_triggers.len()
                + g.state.stack.iter().filter(|i| i.card == s).count(),
            3
        );
        settle(&mut g);
        assert!(!g.on_battlefield(s));
        assert_eq!(tokens(&g, p, Card::MshGalactusToken).len(), 1);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn msh_galactus_entry_replacement_crosses_chapters_but_main_lore_is_turn_based() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let s = saga(&mut g, p);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        main_lore(&mut g, p);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 3);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        main_lore(&mut g, p);
        settle(&mut g);
        assert!(!g.on_battlefield(s));
        assert_eq!(tokens(&g, p, Card::MshGalactusToken).len(), 1);
        // Two created legends require a real keep choice; the helper resolves it.
        assert!(
            g.state
                .objects
                .iter()
                .filter(|o| o.card == Card::MshGalactusToken)
                .count()
                >= 2
        );
    }
}
#[test]
fn msh_galactus_iv_survives_paid_source_removal_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
        chapter(&mut g, s, 3);
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        g.state.players[p.other().index()].mana = [30; 6];
        let removal = named(&mut g, p.other(), "Get Lost", Zone::Hand);
        cast(&mut g, removal, None, &[Target::Card(s)]);
        settle(&mut g);
        assert!(!g.on_battlefield(s));
        assert_eq!(tokens(&g, p, Card::MshGalactusToken).len(), 1);
        assert!(tokens(&g, p.other(), Card::MshGalactusToken).is_empty());
    }
}
#[test]
fn msh_galactus_countered_final_packet_allows_sacrifice_without_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
        chapter(&mut g, s, 3);
        let id = g
            .state
            .stack
            .iter()
            .find(|i| i.card == s && i.kind == StackKind::TriggeredAbility { ability: 3 })
            .unwrap()
            .id;
        // Bounded production counter primitive: no unsupported ability-counter spell cast.
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert!(!g.on_battlefield(s));
        assert!(tokens(&g, p, Card::MshGalactusToken).is_empty());
    }
}
fn attacks(g: &mut Game, p: Player, t: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[t as usize].summoning_sick = false;
    checked(g, Action::Attack { card: t });
    checked(g, Action::FinishAttackers);
}
#[test]
fn msh_galactus_attack_must_target_any_land_including_sole_own_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = iv(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attacks(&mut g, p, t);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == t)
        });
        reject(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(!g.on_battlefield(land));
    }
}
#[test]
fn msh_galactus_no_land_attack_continues_and_sickness_rejects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = iv(&mut g, p);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: t });
        g.state.objects[t as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: t });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(g.on_battlefield(t));
        assert_eq!(g.state.players[p.other().index()].life, 0);
    }
}
#[test]
fn msh_galactus_flying_block_legality_and_actual_trample_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = iv(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let flyer = named(&mut g, p.other(), "Aerial Doombot", Zone::Battlefield);
        attacks(&mut g, p, t);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: t,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: flyer,
                attacker: t,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        settle(&mut g);
        assert!(!g.on_battlefield(flyer));
        assert_eq!(g.state.players[p.other().index()].life, 1);
    }
}
#[test]
fn msh_galactus_land_packet_survives_paid_token_bounce_and_exact_reentry() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let t = iv(&mut g, p);
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            attacks(&mut g, p, t);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == t)
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            finish(&mut g);
            if blink {
                g.move_object(land, Zone::Exile);
                g.move_object(land, Zone::Battlefield);
            }
            if g.actor() != Some(p.other()) {
                checked(&mut g, Action::Pass);
            }
            g.state.players[p.other().index()].mana = [30; 6];
            let bounce = named(&mut g, p.other(), "Unsummon", Zone::Hand);
            cast(&mut g, bounce, None, &[Target::Card(t)]);
            settle(&mut g);
            assert!(!g.on_battlefield(t));
            assert_eq!(g.on_battlefield(land), blink);
        }
    }
}
#[test]
fn msh_galactus_saga_live_type_loss_stops_lore_and_sacrifice_not_queued_chapter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        g.effect_counter(s, CounterKind::Lore, 1, replacements::ResumeMode::Pending);
        chapter(&mut g, s, 1);
        // Bounded animation makes Aura targetable; actual Aura removes type/abilities.
        animate(&mut g, s, false);
        let aura = named(&mut g, p, "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        assert!(!g.has_subtype(s, "Saga"));
        assert!(!g.printed_abilities_available(s));
        main_lore(&mut g, p);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 2);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.effect_counter(s, CounterKind::Lore, 2, replacements::ResumeMode::Pending);
        settle(&mut g);
        assert!(g.on_battlefield(s));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.has_subtype(s, "Saga"));
        settle(&mut g);
        assert!(!g.on_battlefield(s));
    }
}
#[test]
fn msh_galactus_saga_chapter_suppression_preserves_subtype_no_lore_no_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        animate(&mut g, s, false);
        let aura = paid(&mut g, p, "Frozen in Ice", &[Target::Card(s)]);
        assert!(g.has_subtype(s, "Saga"));
        assert!(!g.printed_abilities_available(s));
        main_lore(&mut g, p);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 1);
        settle(&mut g);
        g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
        settle(&mut g);
        assert!(g.on_battlefield(s));
        assert!(tokens(&g, p, Card::MshGalactusToken).is_empty());
        g.move_object(aura, Zone::Graveyard);
        settle(&mut g);
        assert!(!g.on_battlefield(s));
        assert!(tokens(&g, p, Card::MshGalactusToken).is_empty());
    }
}
#[test]
fn msh_galactus_suppressed_queued_iv_resolves_then_restoration_sacrifices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        animate(&mut g, s, false);
        g.effect_counter(s, CounterKind::Lore, 3, replacements::ResumeMode::Pending);
        chapter(&mut g, s, 3);
        // Bounded ordinary Aura attachment while IV is queued; no sorcery cast.
        let aura = named(&mut g, p, "Frozen in Ice", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        settle(&mut g);
        assert!(g.on_battlefield(s));
        assert_eq!(tokens(&g, p, Card::MshGalactusToken).len(), 1);
        g.move_object(aura, Zone::Graveyard);
        settle(&mut g);
        assert!(!g.on_battlefield(s));
    }
}
#[test]
fn msh_galactus_wrong_controller_and_second_main_never_place_lore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = saga(&mut g, p);
        main_lore(&mut g, p.other());
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 1);
        g.state.active = p;
        g.begin_step(Step::Main2);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Lore), 1);
    }
}
