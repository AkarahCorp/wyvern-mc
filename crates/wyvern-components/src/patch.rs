use wyvern_values::Id;

use super::DataComponentMap;

#[derive(Clone, Debug)]
pub struct DataComponentPatch {
    added_fields: DataComponentMap,
    removed_fields: Vec<Id>,
}

impl DataComponentPatch {
    pub fn from_maps(
        prototype: &DataComponentMap,
        new_form: &DataComponentMap,
    ) -> DataComponentPatch {
        let mut added_fields = DataComponentMap::new();

        for key in new_form.keys() {
            let Some(new_form_value) = new_form.inner.get(key) else {
                continue;
            };
            let Some(prototype_value) = prototype.inner.get(key) else {
                added_fields
                    .inner
                    .insert(key.clone(), new_form.inner.get(key).unwrap().clone());
                continue;
            };

            if !new_form_value.compare(prototype_value.as_ref()) {
                added_fields
                    .inner
                    .insert(key.clone(), new_form.inner.get(key).unwrap().clone());
            }
        }
        let mut removed_fields = Vec::new();
        for key in prototype.keys() {
            if !new_form.contains(key) {
                removed_fields.push(key.clone());
            }
        }

        DataComponentPatch {
            added_fields,
            removed_fields,
        }
    }

    pub fn added_fields(&self) -> &DataComponentMap {
        &self.added_fields
    }

    pub fn removed_fields(&self) -> &[Id] {
        &self.removed_fields
    }
}
