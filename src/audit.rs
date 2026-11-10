//! Cross-table audit: everything parsing does not check (identifier and registry uniqueness,
//! references between tables, runtime collisions, catalog completeness). Produces one report
//! per configuration and per required symbol, usable in CI.

use crate::{ClassId, Dataset, Type};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Problem {
    /// `"<minecraft_version> <launcher> <loader>"` when the problem concerns a configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Semantic identifier concerned, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfigurationCoverage {
    pub minecraft_version: String,
    pub launcher: &'static str,
    pub loader: &'static str,
    pub profile_id: String,
    /// Required symbols resolved through [`Dataset::select`], per kind.
    pub classes: usize,
    pub fields: usize,
    pub methods: usize,
    /// Every catalog symbol resolves.
    pub complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AuditReport {
    pub data_version: u64,
    pub required: [usize; 3],
    pub configurations: Vec<ConfigurationCoverage>,
    pub problems: Vec<Problem>,
}

impl AuditReport {
    pub fn is_ok(&self) -> bool {
        self.problems.is_empty() && self.configurations.iter().all(|c| c.complete)
    }
}

fn problem(
    configuration: Option<String>,
    profile: Option<&str>,
    symbol: Option<&str>,
    message: impl Into<String>,
) -> Problem {
    Problem {
        configuration,
        profile: profile.map(str::to_owned),
        symbol: symbol.map(str::to_owned),
        message: message.into(),
    }
}

/// Mapped classes referenced by a type, and external names it uses.
fn type_refs<'a>(ty: &'a Type, classes: &mut Vec<&'a ClassId>, externals: &mut Vec<&'a str>) {
    match ty {
        Type::Class(id) => classes.push(id),
        Type::External(name) => externals.push(name),
        Type::Array(element) => type_refs(element, classes, externals),
        Type::Primitive(_) | Type::Void => {}
    }
}

impl Dataset {
    /// Checks every cross-table rule of the format and resolves every required symbol of every
    /// configuration. Local rules are already enforced by parsing and [`Dataset::validate`].
    pub fn audit(&self) -> AuditReport {
        let mut problems = Vec::new();
        let catalog = &self.catalog;
        // Identifiers are unique across the whole catalog, kinds included.
        let mut ids = HashSet::new();
        let all_ids = catalog
            .classes
            .iter()
            .map(|i| i.as_str())
            .chain(catalog.fields.iter().map(|i| i.as_str()))
            .chain(catalog.methods.iter().map(|i| i.as_str()));
        for id in all_ids {
            if !ids.insert(id) {
                problems.push(problem(
                    None,
                    None,
                    Some(id),
                    "duplicate catalog identifier",
                ));
            }
        }
        let mut profile_ids = HashSet::new();
        for p in &self.profiles {
            if !profile_ids.insert(p.profile_id.as_str()) {
                problems.push(problem(
                    None,
                    Some(p.profile_id.as_str()),
                    None,
                    "duplicate profile id",
                ));
            }
        }
        let mut tuples = HashSet::new();
        for c in &self.configurations {
            let name = format!(
                "{} {} {}",
                c.minecraft_version,
                c.launcher.as_str(),
                c.loader.as_str()
            );
            if !tuples.insert((c.minecraft_version.as_str(), c.launcher, c.loader)) {
                problems.push(problem(
                    Some(name.clone()),
                    None,
                    None,
                    "duplicate configuration",
                ));
            }
            if !profile_ids.contains(c.profile_id.as_str()) {
                problems.push(problem(
                    Some(name),
                    Some(c.profile_id.as_str()),
                    None,
                    "missing profile",
                ));
            }
        }
        for p in &self.profiles {
            problems.extend(self.audit_profile(p));
        }
        let mut configurations = Vec::new();
        for c in &self.configurations {
            let name = format!(
                "{} {} {}",
                c.minecraft_version,
                c.launcher.as_str(),
                c.loader.as_str()
            );
            let (mut classes, mut fields, mut methods) = (0, 0, 0);
            let mut complete = true;
            match self.select(&c.minecraft_version, c.launcher, c.loader) {
                Ok(selection) => {
                    let mut record =
                        |result: Result<(), crate::ResolveError>, id: &str, count: &mut usize| {
                            match result {
                                Ok(()) => *count += 1,
                                Err(e) => {
                                    complete = false;
                                    problems.push(problem(
                                        Some(name.clone()),
                                        Some(c.profile_id.as_str()),
                                        Some(id),
                                        e.to_string(),
                                    ));
                                }
                            }
                        };
                    for id in &catalog.classes {
                        record(selection.class(id).map(drop), id.as_str(), &mut classes);
                    }
                    for id in &catalog.fields {
                        record(selection.field(id).map(drop), id.as_str(), &mut fields);
                    }
                    for id in &catalog.methods {
                        record(selection.method(id).map(drop), id.as_str(), &mut methods);
                    }
                }
                Err(e) => {
                    complete = false;
                    problems.push(problem(
                        Some(name),
                        Some(c.profile_id.as_str()),
                        None,
                        e.to_string(),
                    ));
                }
            }
            configurations.push(ConfigurationCoverage {
                minecraft_version: c.minecraft_version.clone(),
                launcher: c.launcher.as_str(),
                loader: c.loader.as_str(),
                profile_id: c.profile_id.to_string(),
                classes,
                fields,
                methods,
                complete,
            });
        }
        // Several problems can describe one defect; keep each once, in a stable order.
        let mut seen = HashSet::new();
        problems.retain(|p| seen.insert(p.clone()));
        AuditReport {
            data_version: self.data_version.get(),
            required: [
                catalog.classes.len(),
                catalog.fields.len(),
                catalog.methods.len(),
            ],
            configurations,
            problems,
        }
    }

    fn audit_profile(&self, p: &crate::Profile) -> Vec<Problem> {
        let mut problems = Vec::new();
        let pid = Some(p.profile_id.as_str());
        let mut push = |symbol: Option<&str>, message: String| {
            problems.push(problem(None, pid, symbol, message))
        };
        // Exactly one record per catalog identifier of each kind, and no other.
        let mut check_ids = |kind: &str, catalog: Vec<&str>, records: Vec<&str>| {
            let wanted: HashSet<&str> = catalog.iter().copied().collect();
            let mut seen = HashSet::new();
            for id in &records {
                if !wanted.contains(id) {
                    push(Some(id), format!("{kind} record not in the catalog"));
                } else if !seen.insert(*id) {
                    push(Some(id), format!("several {kind} records"));
                }
            }
            for id in catalog {
                if !seen.contains(id) {
                    push(Some(id), format!("missing {kind} record"));
                }
            }
        };
        check_ids(
            "class",
            self.catalog.classes.iter().map(|i| i.as_str()).collect(),
            p.classes.iter().map(|r| r.id.as_str()).collect(),
        );
        check_ids(
            "field",
            self.catalog.fields.iter().map(|i| i.as_str()).collect(),
            p.fields.iter().map(|r| r.id.as_str()).collect(),
        );
        check_ids(
            "method",
            self.catalog.methods.iter().map(|i| i.as_str()).collect(),
            p.methods.iter().map(|r| r.id.as_str()).collect(),
        );
        let runtime: HashMap<&ClassId, &str> = p
            .classes
            .iter()
            .map(|c| (&c.id, c.runtime_name.as_str()))
            .collect();
        let mut names = HashSet::new();
        for c in &p.classes {
            if !names.insert(c.runtime_name.as_str()) {
                push(
                    Some(c.id.as_str()),
                    format!("runtime class {} is mapped twice", c.runtime_name),
                );
            }
        }
        let mut check_types = |symbol: &str, types: Vec<&Type>| {
            let (mut classes, mut externals) = (Vec::new(), Vec::new());
            for t in types {
                type_refs(t, &mut classes, &mut externals);
            }
            for id in classes {
                if !runtime.contains_key(id) {
                    push(
                        Some(symbol),
                        format!("type references {id}, which has no class record"),
                    );
                }
            }
            for name in externals {
                if names.contains(name) {
                    push(
                        Some(symbol),
                        format!("external type {name} is a mapped class: use its class id"),
                    );
                }
            }
        };
        for f in &p.fields {
            check_types(f.id.as_str(), vec![&f.field_type]);
        }
        for m in &p.methods {
            check_types(
                m.id.as_str(),
                m.parameters.iter().chain([&m.returns]).collect(),
            );
        }
        // Runtime collisions: (owner, name, resolved descriptor), static flag not distinguishing.
        let Ok(selection) = self.select_profile(p) else {
            problems.push(problem(
                None,
                pid,
                None,
                "profile is not used by any configuration",
            ));
            return problems;
        };
        let mut members = HashSet::new();
        for f in &p.fields {
            if !runtime.contains_key(&f.owner) {
                problems.push(problem(
                    None,
                    pid,
                    Some(f.id.as_str()),
                    format!("owner {} has no class record", f.owner),
                ));
            } else if let Ok(d) = selection.descriptor(&f.field_type)
                && !members.insert(('f', runtime[&f.owner], f.runtime_name.as_str(), d.clone()))
            {
                problems.push(problem(
                    None,
                    pid,
                    Some(f.id.as_str()),
                    format!(
                        "runtime field {}.{} {d} is mapped twice",
                        runtime[&f.owner], f.runtime_name
                    ),
                ));
            }
        }
        for m in &p.methods {
            if !runtime.contains_key(&m.owner) {
                problems.push(problem(
                    None,
                    pid,
                    Some(m.id.as_str()),
                    format!("owner {} has no class record", m.owner),
                ));
                continue;
            }
            let descriptor: Result<String, _> = m
                .parameters
                .iter()
                .map(|t| selection.descriptor(t))
                .collect::<Result<String, _>>()
                .and_then(|params| Ok(format!("({params}){}", selection.descriptor(&m.returns)?)));
            if let Ok(d) = descriptor
                && !members.insert(('m', runtime[&m.owner], m.runtime_name.as_str(), d.clone()))
            {
                problems.push(problem(
                    None,
                    pid,
                    Some(m.id.as_str()),
                    format!(
                        "runtime method {}.{}{d} is mapped twice",
                        runtime[&m.owner], m.runtime_name
                    ),
                ));
            }
        }
        problems
    }

    /// A selection over `profile` regardless of the registry (for auditing unused profiles too).
    fn select_profile<'a>(
        &'a self,
        profile: &'a crate::Profile,
    ) -> Result<crate::Selection<'a>, ()> {
        let configuration = self
            .configurations
            .iter()
            .find(|c| c.profile_id == profile.profile_id)
            .ok_or(())?;
        Ok(crate::Selection {
            dataset: self,
            configuration,
            profile,
        })
    }
}
