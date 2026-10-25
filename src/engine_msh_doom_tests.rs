//! Complete Doom cards through paid legal actions, private views and owned replay.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const DOCTOR: &str = "Doctor Doom";
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
}
fn mana(card: CardId, ability: u16, color: usize) -> Action {
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
fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn finish(g: &mut Game) {
    for _ in 0..24 {
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
                )
            })
            .unwrap_or_else(|| panic!("unpaid {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unfinished");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
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
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn castle(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Castle Doom", Zone::Hand);
    checked(g, Action::PlayLand { card: id });
    assert!(!g.object_view(id).tapped);
    id
}
fn select(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    checked(g, Action::ChooseAdditionalCost { object });
}
fn bots(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.object_name(id as CardId) == "Doombot")
                .then_some(id as CardId)
        })
        .collect()
}
fn assert_bot(g: &Game, id: CardId) {
    let o = g.object_view(id);
    assert_eq!(o.name(), "Doombot");
    assert_eq!(o.card, Card::MshDoombotToken);
    assert_eq!((o.power, o.toughness), (3, 3));
    assert_eq!(o.colors, 0);
    assert!(o.has_subtype("Robot") && o.has_subtype("Villain"));
    assert!(g.current_kinds(id) & 17 == 17);
    assert!(!o.tapped);
    assert!(o.summoning_sick);
    assert_eq!(g.keywords(id), Keywords::NONE);
    assert!(!o.card.is_legendary());
}
fn ind(g: &Game, id: CardId) -> bool {
    g.keywords(id).contains(Keywords::INDESTRUCTIBLE)
}
fn make(g: &mut Game, p: Player, c: CardId, fuel: CardId) {
    g.state.priority = p;
    checked(g, announce(c, 2));
    select(g, fuel);
    finish(g);
}
#[test]
fn msh_doom_castle_land_and_both_mana_routes_exact_colors() {
    for p in [Player::P0, Player::P1] {
        for color in 0..6 {
            let mut g = setup(p);
            let c = castle(&mut g, p);
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, mana(c, 1, 5));
            reject(&mut g, mana(c, 0, 0));
            checked(&mut g, mana(c, u16::from(color != 5), color));
            let mut expect = [0; 6];
            expect[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, expect);
            assert!(g.object_view(c).tapped);
            reject(&mut g, mana(c, 0, 5));
            let lots = g.observe(p).mana_lots;
            assert_eq!(lots.len(), usize::from(color != 5));
            if color != 5 {
                assert_eq!(
                    lots[0].restriction,
                    mana_lots::ManaRestrictionView::ArtifactSpell
                );
            }
        }
    }
}
#[test]
fn msh_doom_castle_real_funding_announces_artifact_but_not_other_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = castle(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let artifact = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: shock });
        checked(&mut g, Action::AnnounceCast { card: artifact });
        checked(&mut g, mana(c, 1, 3));
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(shock).zone, Zone::Hand);
    }
}
#[test]
fn msh_doom_castle_restriction_survives_source_departure_and_cannot_pay_activations() {
    for p in [Player::P0, Player::P1] {
        for source_change in 0..3 {
            let mut g = setup(p);
            let c = castle(&mut g, p);
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, mana(c, 1, 3));
            match source_change {
                0 => {
                    g.move_object(c, Zone::Exile);
                }
                1 => {
                    let r = g.object_ref(c);
                    g.change_controller(r, p.other());
                }
                _ => {
                    g.move_object(c, Zone::Exile);
                    g.move_object(c, Zone::Battlefield);
                }
            }
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: shock });
            let dagger = named(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
            reject(&mut g, announce(dagger, 0));
            let artifact = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
            cast(&mut g, artifact, None, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_doom_castle_sacrifice_paid_exact_artifact_and_sorcery_only() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let c = castle(&mut g, p);
            let fuel = if token {
                g.create_tokens(p, Card::MshRobotVillainToken, 1, false);
                g.state.objects.len() as CardId - 1
            } else {
                named(&mut g, p, "Gilded Lotus", Zone::Battlefield)
            };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            let wrong = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            g.state.step = Step::End;
            reject(&mut g, announce(c, 2));
            g.state.step = Step::Main1;
            checked(&mut g, announce(c, 2));
            for card in [wrong, enemy] {
                let object = g.object_ref(card);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let fuel_reference = g.object_ref(fuel);
            select(&mut g, fuel);
            finish(&mut g);
            assert!(g.object_view(c).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(bots(&g, p).is_empty());
            settle(&mut g);
            assert_eq!(bots(&g, p).len(), 1);
            assert_bot(&g, bots(&g, p)[0]);
            assert!(
                g.object_ref(fuel) != fuel_reference
                    || g.object_view(fuel).zone != Zone::Battlefield
            );
        }
    }
}
#[test]
fn msh_doom_castle_artifact_source_can_sacrifice_itself_after_tapping() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = castle(&mut g, p);
        let item = g.cast_preview(p, c, Zone::Battlefield, g.object_ref(c), 0, [0; 6]);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        g.state.objects[c as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        make(&mut g, p, c, c);
        assert_eq!(g.object_view(c).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(bots(&g, p).len(), 1);
        assert_bot(&g, bots(&g, p)[0]);
    }
}
#[test]
fn msh_doom_castle_costs_stay_paid_after_source_theft_departure_or_counter() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let c = castle(&mut g, p);
            let fuel = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            make(&mut g, p, c, fuel);
            if change == 0 {
                g.move_object(c, Zone::Exile);
            } else if change == 1 {
                let r = g.object_ref(c);
                g.change_controller(r, p.other());
            } else {
                let id = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(id, false));
            }
            settle(&mut g);
            assert_eq!(bots(&g, p).len(), usize::from(change != 2));
            assert!(bots(&g, p.other()).is_empty());
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_doom_castle_sacrifice_stolen_artifact_and_replaced_destination() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = castle(&mut g, p);
        let fuel = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let r = g.object_ref(fuel);
        g.change_controller(r, p);
        named(&mut g, p, "Rest in Peace", Zone::Battlefield);
        make(&mut g, p, c, fuel);
        settle(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert_eq!(bots(&g, p).len(), 1);
    }
}
#[test]
fn msh_doom_castle_ordinary_and_restricted_mana_are_not_interchangeable_for_own_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = castle(&mut g, p);
        let other = named(&mut g, p, "Castle Doom", Zone::Battlefield);
        let fuel = named(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(other, 1, 4));
        g.state.players[p.index()].mana[5] = 2;
        reject(&mut g, announce(c, 2));
        g.state.players[p.index()].mana[5] = 3;
        make(&mut g, p, c, fuel);
        settle(&mut g);
        assert_eq!(bots(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
    }
}
#[test]
fn msh_doom_castle_mana_artifact_can_fund_then_be_sacrificed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = castle(&mut g, p);
        let fuel = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, announce(c, 2));
        select(&mut g, fuel);
        let a = Action::ActivateMana {
            card: fuel,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [3, 0, 0, 0, 0, 0],
        };
        checked(&mut g, a);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
        assert_eq!(bots(&g, p).len(), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_doom_doctor_exact_paid_body_and_shared_token_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 4];
        let d = paid(&mut g, p, DOCTOR, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(d),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        for t in ["Human", "Scientist", "Villain"] {
            assert!(g.has_subtype(d, t));
        }
        assert!(g.object_view(d).card.is_legendary());
        assert!(ind(&g, d));
        assert_eq!(bots(&g, p).len(), 2);
        for id in bots(&g, p) {
            assert_bot(&g, id);
        }
        assert_eq!(Card::MshRobotVillainToken.effect(), Effect::Creature(2, 2));
        assert_ne!(Card::MshRobotVillainToken.name(), "Doombot");
    }
}
#[test]
fn msh_doom_doctor_etb_and_doubling_survive_old_source_departure() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let d = named(&mut g, p, DOCTOR, Zone::Hand);
            cast(&mut g, d, None, &[]);
            until(&mut g, |g| {
                g.object_view(d).zone == Zone::Battlefield
                    && g.state
                        .stack
                        .iter()
                        .any(|s| s.card == d && s.kind != StackKind::Spell)
            });
            g.move_object(d, Zone::Exile);
            if blink {
                g.move_object(d, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(bots(&g, p).len(), if blink { 8 } else { 4 });
        }
    }
}
#[test]
fn msh_doom_doctor_per_object_and_type_or_plan_condition_truth_table() {
    for p in [Player::P0, Player::P1] {
        for witness in 0..8 {
            let mut g = setup(p);
            let d = named(&mut g, p, DOCTOR, Zone::Battlefield);
            match witness {
                0 => {}
                1 => {
                    named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
                }
                2 => {
                    named(&mut g, p, "Bear Cub", Zone::Battlefield);
                }
                3 => {
                    named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
                    named(&mut g, p, "Bear Cub", Zone::Battlefield);
                }
                4 => {
                    named(&mut g, p, "Aerial Doombot", Zone::Battlefield);
                }
                5 => {
                    named(&mut g, p, "Claim the Kingdom", Zone::Battlefield);
                }
                6 => {
                    named(&mut g, p.other(), "Aerial Doombot", Zone::Battlefield);
                }
                _ => {
                    let item = g.cast_preview(p, d, Zone::Battlefield, g.object_ref(d), 0, [0; 6]);
                    g.animate_source(
                        &item,
                        crate::cards::AnimationSpec {
                            artifact: true,
                            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                            base: Some((3, 3)),
                            keywords: Keywords::NONE,
                            duration: crate::cards::AnimationDuration::Permanent,
                        },
                    );
                }
            }
            assert_eq!(ind(&g, d), matches!(witness, 4 | 5 | 7));
        }
    }
}
#[test]
fn msh_doom_doctor_live_control_witness_and_source_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p, DOCTOR, Zone::Battlefield);
        let w = named(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        assert!(ind(&g, d));
        let r = g.object_ref(w);
        g.change_controller(r, p.other());
        assert!(!ind(&g, d));
        let r = g.object_ref(d);
        g.change_controller(r, p.other());
        assert!(ind(&g, d));
        let island = named(&mut g, p.other(), "Island", Zone::Battlefield);
        until(&mut g, |g| {
            g.state.active == p.other()
                && g.state.step == Step::Main1
                && g.actor() == Some(p.other())
        });
        checked(&mut g, Action::TapMana { card: island });
        let aura = paid(&mut g, p.other(), "Witness Protection", &[Target::Card(d)]);
        assert!(!ind(&g, d));
        g.move_object(aura, Zone::Graveyard);
        assert!(ind(&g, d));
    }
}
#[test]
fn msh_doom_doctor_marked_lethal_damage_and_consumed_deathtouch_differ() {
    for p in [Player::P0, Player::P1] {
        for lethal in [false, true] {
            let mut g = setup(p);
            let d = named(&mut g, p, DOCTOR, Zone::Battlefield);
            let w = named(&mut g, p, "Aerial Doombot", Zone::Battlefield);
            g.state.objects[d as usize].damage = if lethal { 3 } else { 1 };
            g.state.objects[d as usize].deathtouch_since_sba = !lethal;
            g.state_based_actions();
            assert_eq!(g.object_view(d).zone, Zone::Battlefield);
            assert!(!g.state.objects[d as usize].deathtouch_since_sba);
            g.move_object(w, Zone::Exile);
            g.state_based_actions();
            assert_eq!(
                g.object_view(d).zone,
                if lethal {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn msh_doom_doctor_paid_destroy_is_prevented_but_exile_is_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, DOCTOR, &[]);
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Murder", &[Target::Card(d)]);
        assert_eq!(g.object_view(d).zone, Zone::Battlefield);
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Ride's End", &[Target::Card(d)]);
        assert_eq!(g.object_view(d).zone, Zone::Exile);
    }
}
#[test]
fn msh_doom_doctor_end_step_retains_captured_controller_after_theft_or_departure() {
    for p in [Player::P0, Player::P1] {
        for theft in [false, true] {
            let mut g = setup(p);
            let d = paid(&mut g, p, DOCTOR, &[]);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            until(&mut g, |g| {
                g.state.step == Step::End
                    && g.state.stack.iter().any(|s| {
                        s.card == d && s.kind == StackKind::TriggeredAbility { ability: 1 }
                    })
            });
            if theft {
                let r = g.object_ref(d);
                g.change_controller(r, p.other());
            } else {
                g.move_object(d, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
            assert_eq!(g.state.players[p.index()].life, 19);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn msh_doom_doctor_failed_draw_still_loses_life_before_sba() {
    for p in [Player::P0, Player::P1] {
        for protected in [false, true] {
            let mut g = setup(p);
            let d = paid(&mut g, p, DOCTOR, &[]);
            if protected {
                named(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
            }
            until(&mut g, |g| {
                g.state.step == Step::End
                    && g.state.stack.iter().any(|s| {
                        s.card == d && s.kind == StackKind::TriggeredAbility { ability: 1 }
                    })
            });
            let ids = g.state.players[p.index()].library.to_vec();
            for id in ids {
                g.move_object(id, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 19);
            assert_eq!(g.outcome().is_none(), protected);
        }
    }
}
#[test]
fn msh_doom_doctor_hidden_draw_does_not_disclose_card_identity() {
    for p in [Player::P0, Player::P1] {
        let mut a = setup(p);
        paid(&mut a, p, DOCTOR, &[]);
        let mut b = a.clone();
        let top = *b.state.players[p.index()].library.last().unwrap();
        b.state.objects[top as usize].card = Card::from_name("Shock").unwrap();
        for g in [&mut a, &mut b] {
            until(g, |g| {
                g.state.step == Step::End
                    && g.state
                        .stack
                        .iter()
                        .any(|s| matches!(s.kind, StackKind::TriggeredAbility { ability: 1 }))
            });
            settle(g);
        }
        assert_eq!(
            serde_json::to_value(a.observe(p.other())).unwrap(),
            serde_json::to_value(b.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_value(a.observe(p)).unwrap(),
            serde_json::to_value(b.observe(p)).unwrap()
        );
    }
}
// No admitted effect currently leaves Doctor noncreature while retaining this
// ability. A borrowed earlier-layer fixture changes only his type, not his card.
struct Noncreature<'a> {
    g: &'a Game,
    source: CardId,
}
impl CharacteristicWorld for Noncreature<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn object(&self, id: CardId) -> &Object {
        &self.g.state.objects[id as usize]
    }
    fn life(&self, p: Player) -> i128 {
        self.g.state.players[p.index()].life
    }
    fn gained_life(&self, p: Player) -> bool {
        self.g.state.players[p.index()].life_gained_this_turn > 0
    }
    fn attacking(&self, _: CardId) -> bool {
        false
    }
    fn kinds(&self, id: CardId) -> u16 {
        if id == self.source {
            2
        } else {
            self.g.current_kinds(id)
        }
    }
}
#[test]
fn msh_doom_doctor_conditional_self_keyword_applies_to_noncreature_permanent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p, DOCTOR, Zone::Battlefield);
        named(&mut g, p, "Claim the Kingdom", Zone::Battlefield);
        let w = Noncreature { g: &g, source: d };
        assert_eq!(w.kinds(d), 2);
        assert!(
            characteristic_query::CharacteristicQuery::in_world(&w)
                .object_view(d)
                .keywords
                .contains(Keywords::INDESTRUCTIBLE)
        );
        assert!(
            ability_evaluation::evaluate_abilities(&w).result.objects[d as usize]
                .keywords
                .contains(Keywords::INDESTRUCTIBLE)
        );
    }
}

#[test]
fn msh_doom_doctor_paid_copied_creature_spell_creates_two_independent_etb_batches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p, DOCTOR, Zone::Hand);
        cast(&mut g, d, None, &[]);
        let copy = named(&mut g, p, "Choreographed Sparks", Zone::Hand);
        cast(&mut g, copy, Some(1), &[Target::Spell(d)]);
        settle(&mut g);
        assert_eq!(bots(&g, p).len(), 4);
        let dooms: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield && g.object_name(id as CardId) == DOCTOR)
                    .then_some(id as CardId)
            })
            .collect();
        assert_eq!(dooms.len(), 1);
        assert!(ind(&g, dooms[0]));
    }
}
#[test]
fn msh_doom_doombot_copy_preserves_rules_name_not_factory_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.stage_copy_creation(p, CopyRecipe::new(Card::MshDoombotToken), 1);
        settle(&mut g);
        let ids = bots(&g, p);
        assert_eq!(ids.len(), 1);
        assert_bot(&g, ids[0]);
    }
}
