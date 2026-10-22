//! Public frozen characteristics of actual declarations; attacking entry is excluded.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DeclaredAttackView {
    pub object: ObjectRef,
    pub controller: Player,
    pub card: Card,
    pub subtypes: GrantedSubtypes,
}
impl Game {
    pub(super) fn record_declared_attack_types(
        &mut self,
        controller: Player,
        attacks: &[AttackDeclaration],
    ) {
        let rows: Vec<_> = attacks
            .iter()
            .map(|attack| DeclaredAttackView {
                object: self.object_ref(attack.card),
                controller,
                card: self.state.objects[attack.card as usize].card,
                subtypes: self.granted_subtypes(attack.card),
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        let history = &mut self
            .state
            .roles
            .get_or_insert_with(Default::default)
            .attack_type_history;
        for row in rows {
            if !history.contains(&row) {
                history.push(row);
            }
        }
    }
    pub(super) fn declared_subtype_attackers(&self, controller: Player, subtype: &str) -> usize {
        let Some(roles) = &self.state.roles else {
            return 0;
        };
        let matches = |row: &DeclaredAttackView| {
            row.controller == controller && row.subtypes.has(row.card, subtype)
        };
        roles
            .attack_type_history
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                matches(row)
                    && !roles.attack_type_history[..*index]
                        .iter()
                        .any(|previous| previous.object == row.object && matches(previous))
            })
            .count()
    }
}
