//! Complete choice-card acceptance beyond the shared entry/layer core regressions.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;
use crate::{ChoiceColor, CreatureType};

fn color(value: ChoiceColor) -> EntryChoiceValue {
    EntryChoiceValue::Color(value)
}
fn typ(name: &str) -> EntryChoiceValue {
    EntryChoiceValue::CreatureType(CreatureType::from_name(name).unwrap())
}
fn choice_permanent(g: &mut Game, p: Player, name: &str, value: EntryChoiceValue) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    checked(g, Action::ChooseEntryValue { value });
    settle(g);
    id
}
fn tap(g: &mut Game, card: CardId) {
    let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card:id,..}|Action::TapMana {card:id} if *id==card)).expect("real mana activation");
    checked(g, action);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(!g.legal_actions().contains(&action));
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn own_main(g: &mut Game, p: Player) {
    for _ in 0..16 {
        if g.state.active == p && g.state.step == Step::Main1 {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("main phase navigation failed");
}

#[test]
fn fdn_choice_five_haven_natural_untap_and_actual_colored_generic_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let haven = named(&mut g, p, "Uncharted Haven", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: haven });
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: color(ChoiceColor::Green),
            },
        );
        assert!(g.object_view(haven).tapped);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ActivateMana {card,..} if *card==haven))
        );
        // A real untap step, rather than clearing the source's tapped bit.
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        assert!(!g.object_view(haven).tapped);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: bear })
        );
        reject(&mut g, Action::AnnounceCast { card: shock });
        tap(&mut g, haven);
        tap(&mut g, forest);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.object_view(haven).entry_choice,
            Some(color(ChoiceColor::Green))
        );
    }
}

#[test]
fn fdn_choice_five_mare_multicolor_countered_and_foreign_casts_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mare = choice_permanent(&mut g, p, "Diamond Mare", color(ChoiceColor::Red));
        let before = g.state.players[p.index()].life;
        let ruby = named(&mut g, p, "Ruby, Daring Tracker", Zone::Hand);
        cast(&mut g, ruby, None, &[]);
        checked(&mut g, Action::Pass);
        let scatter = named(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        cast(&mut g, scatter, None, &[Target::Spell(ruby)]);
        settle(&mut g);
        assert_eq!(g.object_view(ruby).zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].life,
            before + 1,
            "one cast trigger even for two colors and a countered spell"
        );
        for name in ["Bear Cub", "Scrawling Crawler"] {
            let card = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, card, None, &[]);
            settle(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, before + 1);
        checked(&mut g, Action::Pass);
        let enemy_shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, enemy_shock, None, &[Target::Card(mare)]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            before + 1,
            "opponent's matching spell is not yours"
        );
        let cold = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, cold, None, &[Target::Card(mare)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(mare));
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before + 1);
    }
}

#[test]
fn fdn_choice_five_heraldic_bonus_tracks_multicolor_control_and_pays_real_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let panda = named(&mut g, p, "Fiendish Panda", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Cloudblazer", Zone::Battlefield);
        let colorless = named(&mut g, p, "Scrawling Crawler", Zone::Battlefield);
        let panda_base = g.stats(panda);
        let foreign_base = g.stats(foreign);
        let colorless_base = g.stats(colorless);
        let banner = choice_permanent(&mut g, p, "Heraldic Banner", color(ChoiceColor::White));
        assert_eq!(
            g.stats(panda),
            Stats {
                power: panda_base.power + 1,
                toughness: panda_base.toughness
            }
        );
        assert_eq!(g.stats(foreign), foreign_base);
        assert_eq!(g.stats(colorless), colorless_base);
        let steal = named(&mut g, p, "Involuntary Employment", Zone::Hand);
        cast(&mut g, steal, None, &[Target::Card(foreign)]);
        settle(&mut g);
        assert_eq!(g.object_view(foreign).controller, p);
        assert_eq!(
            g.stats(foreign),
            Stats {
                power: foreign_base.power + 1,
                toughness: foreign_base.toughness
            }
        );
        let hawk = named(&mut g, p, "Healer's Hawk", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: hawk })
        );
        tap(&mut g, banner);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        cast(&mut g, hawk, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(hawk),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert_eq!(
            g.object_view(banner).entry_choice,
            Some(color(ChoiceColor::White))
        );
    }
}

#[test]
fn fdn_choice_five_automaton_chosen_type_at_entry_and_independent_lords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let observer = named(&mut g, p, "Valley Mightcaller", Zone::Battlefield);
        let construct = named(&mut g, p, "Scrawling Crawler", Zone::Battlefield);
        let changeling = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        let construct_base = g.stats(construct);
        let changeling_base = g.stats(changeling);
        let foreign_base = g.stats(foreign);
        let rabbit = choice_permanent(&mut g, p, "Adaptive Automaton", typ("Rabbit"));
        assert_eq!(
            g.counter_count(observer, CounterKind::PlusOne),
            1,
            "chosen Rabbit exists when actual ETB triggers inspect the entrant"
        );
        assert_eq!(
            g.stats(construct),
            construct_base,
            "printed Construct does not determine the chosen-type lord"
        );
        assert_eq!(
            g.stats(rabbit),
            Stats {
                power: 2,
                toughness: 2
            },
            "other excludes the actual source"
        );
        let second = choice_permanent(&mut g, p, "Adaptive Automaton", typ("Construct"));
        assert_eq!(g.counter_count(observer, CounterKind::PlusOne), 1);
        assert_eq!(
            g.stats(rabbit),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(
            g.stats(second),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(
            g.stats(construct),
            Stats {
                power: construct_base.power + 1,
                toughness: construct_base.toughness + 1
            }
        );
        assert_eq!(
            g.stats(changeling),
            Stats {
                power: changeling_base.power + 2,
                toughness: changeling_base.toughness + 2
            }
        );
        assert_eq!(g.stats(foreign), foreign_base);
        assert_eq!(g.object_view(rabbit).entry_choice, Some(typ("Rabbit")));
        assert_eq!(g.object_view(second).entry_choice, Some(typ("Construct")));
    }
}

#[test]
fn fdn_choice_five_kinship_zero_then_live_recipients_and_reentry_recounts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let banner = choice_permanent(&mut g, p, "Banner of Kinship", typ("Time Lord"));
        assert_eq!(g.counter_count(banner, CounterKind::Fellowship), 0);
        let automaton = choice_permanent(&mut g, p, "Adaptive Automaton", typ("Time Lord"));
        assert_eq!(
            g.counter_count(banner, CounterKind::Fellowship),
            0,
            "later entries do not add entry counters"
        );
        assert_eq!(
            g.stats(automaton),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let bounce = named(&mut g, p, "Into the Roil", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: bounce });
        checked(&mut g, Action::ChooseKicker { pay: false });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(banner),
            },
        );
        for _ in 0..3 {
            if g.state.announcing.is_none() {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .expect("unkicked bounce payment");
            checked(&mut g, action);
        }
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.object_view(banner).entry_choice, None);
        cast(&mut g, banner, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: typ("Time Lord"),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(banner, CounterKind::Fellowship), 1);
        assert_eq!(
            g.stats(automaton),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(automaton)]);
        settle(&mut g);
        assert_eq!(
            g.counter_count(banner, CounterKind::Fellowship),
            1,
            "creature departure does not remove stored Fellowship counters"
        );
        cast(&mut g, automaton, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: typ("Time Lord"),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(banner, CounterKind::Fellowship), 1);
        assert_eq!(
            g.stats(automaton),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        for viewer in [p, p.other()] {
            assert!(g.observe(viewer).objects.iter().any(|o| o.id == banner
                && o.counters.get(CounterKind::Fellowship) == 1
                && o.entry_choice == Some(typ("Time Lord"))));
        }
    }
}
