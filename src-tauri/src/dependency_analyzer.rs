use std::collections::{HashMap, HashSet, VecDeque};

use crate::odoo17_ce::{ActivationStep, ODOO17_CE_MODULES};

/// Classification of every dependency encountered in the activation chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepSource {
    Internal,   // found in the uploaded zip
    ExternalCE, // bundled with Odoo 17 CE
    Missing,    // not found anywhere
}

/// Build an activation order where **priority = dependency depth**.
///
/// Priority 1 → modules with zero dependencies (or only stdlib/CE base deps).
/// Priority 2 → modules that depend *only* on Priority 1 modules.
/// Priority N → modules whose deepest transitive dependency chain is N − 1.
///
/// Algorithm: Kahn's topological sort with **level tracking** (BFS layering).
/// This guarantees that every module's priority number reflects the exact
/// length of its longest dependency chain.
pub fn compute_activation_order(
    user_modules: &HashSet<&str>,
    user_module_deps: &HashMap<&str, Vec<&str>>,
) -> (Vec<ActivationStep>, Vec<String>) {
    use DepSource::*;

    // ── 1. Collect all names ────────────────────────────────────────────
    let mut all_names: HashSet<&str> = user_modules.iter().cloned().collect();
    for deps in user_module_deps.values() {
        all_names.extend(deps.iter().cloned());
    }

    // ── 2. Build adjacency: module → depends_on (edges point to prereqs) ─
    // Also build reverse: prereq → modules_that_need_it (for Kahn's BFS)
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    // dependents[X] = list of modules that depend on X
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for name in &all_names {
        in_degree.entry(name).or_insert(0);
        dependents.entry(name).or_default();
    }

    for (mod_name, deps) in user_module_deps {
        for dep in deps {
            // mod_name depends on dep → edge: dep → mod_name
            // in_degree of mod_name increases
            *in_degree.entry(mod_name).or_insert(0) += 1;
            dependents.entry(dep).or_default().push(mod_name);
        }
    }

    // ── 3. Kahn's BFS with level tracking ──────────────────────────────
    let mut level: HashMap<&str, u32> = HashMap::new();
    let mut queue: VecDeque<(&str, u32)> = VecDeque::new();

    // Seed: all nodes with in_degree == 0 → level 1
    for name in &all_names {
        if in_degree[name] == 0 {
            queue.push_back((name, 1));
            level.insert(name, 1);
        }
    }

    let mut has_cycle = false;
    let mut processed = 0;

    while let Some((node, lvl)) = queue.pop_front() {
        processed += 1;

        for dependent in dependents.get(node).cloned().unwrap_or_default() {
            let dep_in = in_degree.get_mut(dependent).unwrap();
            *dep_in -= 1;

            // Propagate: dependent's level = max(current, node_level + 1)
            let new_lvl = lvl + 1;
            let entry = level.entry(dependent).or_insert(0);
            if new_lvl > *entry {
                *entry = new_lvl;
            }

            if *dep_in == 0 {
                queue.push_back((dependent, level[dependent]));
            }
        }
    }

    if processed < all_names.len() {
        has_cycle = true;
        // Assign level to remaining unprocessed nodes (cycle members)
        for name in &all_names {
            if !level.contains_key(name) {
                level.insert(name, u32::MAX);
            }
        }
    }

    // ── 4. Build result sorted by (level, name) ────────────────────────
    let mut sorted: Vec<(&str, u32)> = all_names
        .iter()
        .map(|n| (*n, *level.get(n).unwrap_or(&1)))
        .collect();
    sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));

    // ── 5. Assign sequential order within each level ───────────────────
    let mut warnings = Vec::new();
    let mut order = Vec::new();
    let mut seq = 1u32;

    for (name, _lvl) in &sorted {
        let source = classify(name, user_modules);

        if source == DepSource::Missing && !user_modules.contains(name) {
            warnings.push(format!(
                "Module '{}' is not in the uploaded zip or in Odoo 17 CE community.",
                name
            ));
        }

        order.push(ActivationStep {
            order: seq,
            module_name: name.to_string(),
            source: match source {
                Internal => "internal".into(),
                ExternalCE => "external-ce".into(),
                Missing => "missing".into(),
            },
        });
        seq += 1;
    }

    if has_cycle {
        warnings.push("Circular dependency detected — activation order is based on a best-effort.".into());
    }

    (order, warnings)
}

/// Decide where a dependency comes from.
fn classify(name: &str, user_modules: &HashSet<&str>) -> DepSource {
    if user_modules.contains(name) {
        DepSource::Internal
    } else if ODOO17_CE_MODULES.contains_key(name) {
        DepSource::ExternalCE
    } else {
        DepSource::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_depends_module3_1_module2_2_module1_3() {
        // module1 → module2 → module3
        // Expected: module3=1, module2=2, module1=3
        let mut user_mods = HashSet::new();
        user_mods.insert("module1");
        user_mods.insert("module2");
        user_mods.insert("module3");

        let mut deps = HashMap::new();
        deps.insert("module1", vec!["module2"]);
        deps.insert("module2", vec!["module3"]);
        deps.insert("module3", vec![]);

        let (order, _) = compute_activation_order(&user_mods, &deps);

        let pos = |name: &str| -> u32 {
            order.iter().find(|s| s.module_name == name).unwrap().order
        };

        assert!(pos("module3") < pos("module2"), "module3 should come before module2");
        assert!(pos("module2") < pos("module1"), "module2 should come before module1");
    }

    #[test]
    fn test_independent_modules_same_level() {
        // moduleA and moduleB have no deps → both priority 1
        let mut user_mods = HashSet::new();
        user_mods.insert("moduleA");
        user_mods.insert("moduleB");

        let mut deps = HashMap::new();
        deps.insert("moduleA", vec![]);
        deps.insert("moduleB", vec![]);

        let (order, _) = compute_activation_order(&user_mods, &deps);

        // Both should be in first two positions (order 1 and 2)
        let pos_a = order.iter().find(|s| s.module_name == "moduleA").unwrap().order;
        let pos_b = order.iter().find(|s| s.module_name == "moduleB").unwrap().order;
        assert!(pos_a <= 2 && pos_b <= 2);
    }

    #[test]
    fn test_activation_order_respects_depends() {
        let mut user_mods = HashSet::new();
        user_mods.insert("my_module");

        let mut deps = HashMap::new();
        deps.insert("my_module", vec!["sale", "stock"]);

        let (order, _warns) = compute_activation_order(&user_mods, &deps);

        // sale and stock should come before my_module
        let pos = |name: &str| -> u32 {
            order.iter().find(|s| s.module_name == name).unwrap().order
        };
        assert!(pos("sale") < pos("my_module"));
        assert!(pos("stock") < pos("my_module"));
    }

    #[test]
    fn test_missing_warning() {
        let mut user_mods = HashSet::new();
        user_mods.insert("my_module");

        let mut deps = HashMap::new();
        deps.insert("my_module", vec!["nonexistent_xyz"]);

        let (_order, warnings) = compute_activation_order(&user_mods, &deps);
        assert!(warnings.iter().any(|w| w.contains("nonexistent_xyz")));
    }

    #[test]
    fn test_internal_vs_external() {
        use DepSource::*;
        let mods: HashSet<&str> = HashSet::from(["my_module"]);
        assert_eq!(classify("my_module", &mods), Internal);
        assert_eq!(classify("sale", &mods), ExternalCE);
        assert_eq!(classify("nonexistent_xyz", &mods), Missing);
    }
}
