use super::*;
use super::*;
pub use rustmage_core::engine::ability_order::*;
#[cfg(test)]
mod basic_land_type_tests {
    use super::*;

    #[test]
    fn basic_type_removes_printed_abilities_but_retains_external_grants() {
        let source = ObjectRef {
            card: 0,
            generation: 1,
        };
        let initial = LayerSixObject {
            printed: true,
            keywords: Keywords::FLYING,
            granted_keywords: Keywords::NONE,
            grants: Vec::new(),
            self_grants: Vec::new(),
            mana: Vec::new(),
        };
        let external = LayerSixEffect {
            provider: None,
            started_earlier: false,
            timestamp: 1,
            recipients: vec![0],
            action: LayerSixAction::Grant {
                keywords: Keywords::FLYING.union(Keywords::HEXPROOF),
                abilities: Vec::new(),
            },
        };
        let replacement = LayerSixEffect {
            provider: Some(0),
            started_earlier: true,
            timestamp: 2,
            recipients: vec![0],
            action: LayerSixAction::RemovePrintedAndGrant(vec![LayerSixGrant {
                recipient: 0,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![GrantedManaAbility {
                    source,
                    production: ManaProduction::Fixed([0, 0, 0, 0, 1, 0]),
                    sacrifice: false,
                    subtype_derived: true,
                }],
            }]),
        };
        let result = apply_layer_six(vec![initial.clone()], &[external, replacement.clone()]);
        assert!(!result.objects[0].printed);
        assert!(
            result.objects[0]
                .keywords
                .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
        );
        assert_eq!(result.objects[0].mana.len(), 1);
        let remove = LayerSixEffect {
            provider: None,
            started_earlier: false,
            timestamp: 3,
            recipients: vec![0],
            action: LayerSixAction::Remove,
        };
        let result = apply_layer_six(vec![initial], &[replacement, remove]);
        assert_eq!(result.objects[0].keywords, Keywords::NONE);
        assert!(result.objects[0].mana.is_empty());
    }
    #[test]
    fn replacing_land_type_removes_only_old_type_mana() {
        let source = ObjectRef {
            card: 0,
            generation: 1,
        };
        let mana = |color, subtype_derived| {
            let mut production = [0; 6];
            production[color] = 1;
            GrantedManaAbility {
                source,
                production: ManaProduction::Fixed(production),
                sacrifice: false,
                subtype_derived,
            }
        };
        let initial = LayerSixObject {
            printed: true,
            keywords: Keywords::NONE,
            granted_keywords: Keywords::NONE,
            grants: Vec::new(),
            self_grants: Vec::new(),
            mana: vec![mana(1, true), mana(5, false)],
        };
        let change = LayerSixEffect {
            provider: None,
            started_earlier: true,
            timestamp: 2,
            recipients: vec![0],
            action: LayerSixAction::RemovePrintedAndGrant(vec![LayerSixGrant {
                recipient: 0,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![mana(4, true)],
            }]),
        };
        let result = apply_layer_six(vec![initial.clone()], std::slice::from_ref(&change));
        assert_eq!(result.objects[0].mana, vec![mana(5, false), mana(4, true)]);
        let blank = LayerSixEffect {
            provider: None,
            started_earlier: false,
            timestamp: 3,
            recipients: vec![0],
            action: LayerSixAction::Remove,
        };
        let result = apply_layer_six(vec![initial.clone()], &[change.clone(), blank.clone()]);
        assert!(result.objects[0].mana.is_empty());
        let early_blank = LayerSixEffect {
            timestamp: 1,
            ..blank
        };
        let result = apply_layer_six(vec![initial], &[early_blank, change]);
        assert_eq!(result.objects[0].mana, vec![mana(4, true)]);
    }
    #[test]
    fn later_subtype_addition_survives_earlier_blank_and_not_later_blank() {
        let source = ObjectRef {
            card: 0,
            generation: 1,
        };
        let initial = LayerSixObject {
            printed: true,
            keywords: Keywords::NONE,
            granted_keywords: Keywords::NONE,
            grants: Vec::new(),
            self_grants: Vec::new(),
            mana: Vec::new(),
        };
        let grant = LayerSixEffect {
            provider: None,
            started_earlier: true,
            timestamp: 2,
            recipients: vec![0],
            action: LayerSixAction::GrantEach(vec![LayerSixGrant {
                recipient: 0,
                keywords: Keywords::NONE,
                abilities: Vec::new(),
                mana: vec![GrantedManaAbility {
                    source,
                    production: ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
                    sacrifice: false,
                    subtype_derived: true,
                }],
            }]),
        };
        let blank = LayerSixEffect {
            provider: None,
            started_earlier: false,
            timestamp: 1,
            recipients: vec![0],
            action: LayerSixAction::Remove,
        };
        assert_eq!(
            apply_layer_six(vec![initial.clone()], &[blank.clone(), grant.clone()]).objects[0]
                .mana
                .len(),
            1
        );
        let later_blank = LayerSixEffect {
            timestamp: 3,
            ..blank
        };
        assert!(
            apply_layer_six(vec![initial], &[grant, later_blank]).objects[0]
                .mana
                .is_empty()
        );
    }
}
#[cfg(test)]
mod basic_land_loss_tests {
    use super::*;

    #[test]
    fn losing_land_type_keeps_independent_granted_mana() {
        let source = ObjectRef {
            card: 0,
            generation: 1,
        };
        let mana = |color, subtype_derived| {
            let mut production = [0; 6];
            production[color] = 1;
            GrantedManaAbility {
                source,
                production: ManaProduction::Fixed(production),
                sacrifice: false,
                subtype_derived,
            }
        };
        let object = LayerSixObject {
            printed: true,
            keywords: Keywords::NONE,
            granted_keywords: Keywords::NONE,
            grants: Vec::new(),
            self_grants: Vec::new(),
            mana: vec![mana(1, true), mana(5, false)],
        };
        let effect = LayerSixEffect {
            provider: None,
            started_earlier: true,
            timestamp: 2,
            recipients: vec![0],
            action: LayerSixAction::RemoveSubtypeMana,
        };
        let result = apply_layer_six(vec![object], &[effect]);
        assert!(result.objects[0].printed);
        assert_eq!(result.objects[0].mana, vec![mana(5, false)]);
    }
}
