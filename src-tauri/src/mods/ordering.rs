use crate::models::{ModEntry, SettingsData};
use std::collections::HashMap;

fn data_property_mut(root: &mut SettingsData) -> Option<&mut Vec<ModEntry>> {
    root.properties
        .iter_mut()
        .find(|p| p.name == "Data")
        .map(|p| &mut p.mods)
}

pub fn delete_mod_and_reindex(root: &mut SettingsData, mod_name: &str) {
    let Some(mods) = data_property_mut(root) else {
        return;
    };

    mods.retain(|entry| {
        let entry_name = entry
            .properties
            .iter()
            .find(|p| p.name == "Name")
            .and_then(|p| p.value.as_deref())
            .unwrap_or("");

        !entry_name.eq_ignore_ascii_case(mod_name)
    });

    mods.sort_by_key(|entry| {
        entry
            .properties
            .iter()
            .find(|p| p.name == "ModPriority")
            .and_then(|p| p.value.as_ref())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(u32::MAX)
    });

    for (i, mod_entry) in mods.iter_mut().enumerate() {
        mod_entry.index = i.to_string();
    }
}

pub fn reorder_mods(root: &mut SettingsData, ordered_mod_names: &[String]) {
    let Some(mods) = data_property_mut(root) else {
        return;
    };

    let mut mods_map: HashMap<String, ModEntry> = mods
        .drain(..)
        .map(|entry| {
            let name = entry
                .properties
                .iter()
                .find(|p| p.name == "Name")
                .and_then(|p| p.value.as_ref())
                .cloned()
                .unwrap_or_default();
            (name, entry)
        })
        .collect();

    let mut sorted_mods: Vec<ModEntry> = Vec::new();
    for (new_priority, mod_name_upper) in ordered_mod_names
        .iter()
        .map(|n| n.to_uppercase())
        .enumerate()
    {
        if let Some(mut mod_entry) = mods_map.remove(&mod_name_upper) {
            let new_order_str = new_priority.to_string();
            mod_entry.index = new_order_str.clone();
            if let Some(priority_prop) = mod_entry
                .properties
                .iter_mut()
                .find(|p| p.name == "ModPriority")
            {
                priority_prop.value = Some(new_order_str);
            }
            sorted_mods.push(mod_entry);
        }
    }

    sorted_mods.extend(mods_map.into_values());
    *mods = sorted_mods;
}

pub fn rename_mod_in_xml(
    root: &mut SettingsData,
    old_name: &str,
    new_name: &str,
) -> Result<(), String> {
    let Some(mods) = data_property_mut(root) else {
        return Err(format!(
            "Could not find a mod entry with the name '{}' in the XML file.",
            old_name
        ));
    };

    let Some(name_prop) = mods
        .iter_mut()
        .flat_map(|entry| entry.properties.iter_mut())
        .find(|prop| {
            prop.name == "Name"
                && prop
                    .value
                    .as_deref()
                    .is_some_and(|val| val.eq_ignore_ascii_case(old_name))
        })
    else {
        return Err(format!(
            "Could not find a mod entry with the name '{}' in the XML file.",
            old_name
        ));
    };

    name_prop.value = Some(new_name.to_uppercase());
    Ok(())
}

#[cfg(test)]
#[path = "ordering_tests.rs"]
mod tests;
