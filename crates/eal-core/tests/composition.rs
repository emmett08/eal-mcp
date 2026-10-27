//! Exercise lexical identity, closed templates and trusted source-bundle boundaries.
use eal_core::{ExecutionLimits, composition};
use std::collections::BTreeMap;

const BASE: &str = r#"language "EAL/3"
environment lab {
require site == "bench"
}
tool collector {
version "1"
}
context environment lab, tool collector, max_age 60, kind test {
evidence first {
require ok
}
evidence second {
require ok
}
claim passed {
statement "The bounded checks pass."
}
}
reasoning measured {
method "structured/1"
rationale "An authored relation."
}
"#;

fn compile(extra: &str) -> eal_core::semantic::Program {
    composition::compile(
        &format!("{BASE}{extra}"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap()
}
fn codes(program: &eal_core::semantic::Program) -> Vec<&str> {
    program
        .lowering_diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect()
}

#[test]
fn module_and_argument_blocks_bind_forward_local_references() {
    let p = compile(
        r#"module release {
claim ready {
statement "The bounded release condition follows."
environment lab
}
argument assess {
argument child = [evidence first] via measured => intermediate
claim intermediate {
statement "A local result."
environment lab
}
[premises intermediate] via measured => ready
}
}
"#,
    );
    assert!(
        p.lowering_diagnostics.is_empty(),
        "{:?}",
        p.lowering_diagnostics
    );
    assert_eq!(
        p.arguments["release.assess.child"].conclusion,
        "release.assess.intermediate"
    );
    assert_eq!(
        p.arguments["release.assess"].premises,
        vec!["release.assess.intermediate"]
    );
    assert_eq!(p.arguments["release.assess"].conclusion, "release.ready");
}

#[test]
fn compact_list_patterns_preserve_order_and_shared_evidence_identity() {
    let p = compile(
        "pattern together(c: claim, r: reasoning, es: evidence[]) = [evidence es] via r => c\napply checks = together(c=passed, r=measured, es=[second, first, first])\n",
    );
    assert!(
        p.lowering_diagnostics.is_empty(),
        "{:?}",
        p.lowering_diagnostics
    );
    assert_eq!(
        p.arguments["checks"].evidence,
        vec!["second", "first", "first"]
    );
    assert_eq!(
        p.arguments["checks"].origin.as_ref().unwrap().pattern,
        "together"
    );
    assert_eq!(p.evidence.len(), 2);
}

#[test]
fn recursive_patterns_consume_guarded_finite_lists() {
    let p = compile(
        "pattern each(c: claim, r: reasoning, es: evidence[]) decreases es {\nwhen es {\nargument item = [evidence head(es)] via r => c\napply rest = each(c=c, r=r, es=tail(es))\n}\n}\napply checks = each(c=passed, r=measured, es=[first, second])\n",
    );
    assert!(
        p.lowering_diagnostics.is_empty(),
        "{:?}",
        p.lowering_diagnostics
    );
    assert_eq!(
        p.arguments.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["checks.item", "checks.rest.item"]
    );
    assert_eq!(p.arguments["checks.item"].evidence, vec!["first"]);
    assert_eq!(p.arguments["checks.rest.item"].evidence, vec!["second"]);
    assert!(p.applications.contains_key("checks.rest.rest"));
    let replay = composition::relower(&p).unwrap();
    assert_eq!(replay.arguments, p.arguments);
    assert_eq!(replay.declaration_count, p.declaration_count);
}

#[test]
fn nested_pattern_locals_are_hygienic_between_applications() {
    let p = compile(
        "pattern build(c: claim, r: reasoning, e: evidence, env: environment) {\npattern leaf(c: claim, r: reasoning, e: evidence) = [evidence e] via r => c\nclaim intermediate {\nstatement \"A local bounded claim.\"\nenvironment env\n}\napply child = leaf(c=intermediate, r=r, e=e)\nargument parent = [premises intermediate] via r => c\n}\napply one = build(c=passed, r=measured, e=first, env=lab)\napply two = build(c=passed, r=measured, e=second, env=lab)\n",
    );
    assert!(
        p.lowering_diagnostics.is_empty(),
        "{:?}",
        p.lowering_diagnostics
    );
    assert_eq!(p.arguments["one.child"].conclusion, "one.intermediate");
    assert_eq!(p.arguments["two.child"].conclusion, "two.intermediate");
    assert_eq!(p.arguments["one.child"].evidence, vec!["first"]);
    assert_eq!(p.arguments["two.child"].evidence, vec!["second"]);
}

#[test]
fn unused_templates_are_closed_and_recursion_is_statically_guarded() {
    let p = compile("pattern capture(c: claim, r: reasoning) = [evidence first] via r => c\n");
    assert!(codes(&p).contains(&"unbound_pattern_reference"));
    let p = compile(
        "pattern loop(c: claim, r: reasoning, es: evidence[]) decreases es {\napply again = loop(c=c,r=r,es=tail(es))\n}\n",
    );
    assert!(codes(&p).contains(&"nonterminating_pattern"));
    let p = compile(
        "pattern left(c: claim, r: reasoning, es: evidence[]) decreases es {\nwhen es {\napply next = right(c=c,r=r,es=es)\n}\n}\npattern right(c: claim, r: reasoning, es: evidence[]) decreases es {\nwhen es {\napply next = left(c=c,r=r,es=tail(es))\n}\n}\n",
    );
    assert!(codes(&p).contains(&"nonterminating_pattern"));
}

#[test]
fn trusted_imports_keep_source_identity_and_replay_without_io() {
    let sources = BTreeMap::from([("λ.eal".to_owned(), BASE.to_owned())]);
    let resolver = |path: &str| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| "unknown source".into())
    };
    let source = "language \"EAL/3\"\nimport \"λ.eal\" as library\n";
    let p = composition::compile(
        source,
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        p.evidence["library.first"].tool.as_deref(),
        Some("library.collector")
    );
    assert_eq!(p.source_files["library.first"], "λ.eal");
    assert_eq!(p.imports.len(), 1);
    let replay = composition::relower(&p).unwrap();
    assert_eq!(replay.evidence, p.evidence);
    assert_eq!(replay.imports, p.imports);
    use sha2::{Digest, Sha256};
    let root_digest = format!("{:x}", Sha256::digest(source.as_bytes()));
    let import_digest = format!("{:x}", Sha256::digest(BASE.as_bytes()));
    let expected = format!(
        "{{\"imports\":{{\"\\u03bb.eal\":\"{import_digest}\"}},\"source\":\"{root_digest}\"}}"
    );
    assert_eq!(
        p.source_digest,
        format!("{:x}", Sha256::digest(expected.as_bytes()))
    );
}

#[test]
fn import_escape_cycles_and_aggregate_limits_fail_before_exposing_a_program() {
    assert!(
        composition::compile(
            "language \"EAL/3\"\nimport \"../outside.eal\" as outside\n",
            None,
            "root.eal",
            ExecutionLimits::default()
        )
        .is_err()
    );
    let resolver = |_: &str| Ok("language \"EAL/3\"\nimport \"root.eal\" as again\n".to_owned());
    assert!(
        composition::compile(
            "language \"EAL/3\"\nimport \"other.eal\" as other\n",
            Some(&resolver),
            "root.eal",
            ExecutionLimits::default()
        )
        .is_err()
    );
    let resolver = |_: &str| Ok(BASE.to_owned());
    let source = "language \"EAL/3\"\nimport \"library.eal\" as lib\n";
    let limits = ExecutionLimits {
        source_bytes: BASE.len(),
        ..Default::default()
    };
    assert!(composition::compile(source, Some(&resolver), "root.eal", limits).is_err());
}

#[test]
fn exhausted_expansion_discards_all_generated_entities() {
    let source = format!(
        "{BASE}pattern each(c: claim, r: reasoning, es: evidence[]) decreases es {{\nwhen es {{\nargument item = [evidence head(es)] via r => c\napply rest = each(c=c,r=r,es=tail(es))\n}}\n}}\napply checks = each(c=passed,r=measured,es=[first,second])\n"
    );
    let p = composition::compile(
        &source,
        None,
        "",
        ExecutionLimits {
            applications: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(codes(&p).contains(&"resource_limit"));
    assert!(p.generated.is_empty());
    assert!(p.arguments.is_empty());
    assert_eq!(p.evidence.len(), 2);
}

#[test]
fn unused_pattern_imports_are_captured_and_checked_before_application() {
    let source = "language \"EAL/3\"\npattern build() {\nimport \"library.eal\" as lib\n}\n";
    assert!(composition::compile(source, None, "root.eal", ExecutionLimits::default()).is_err());
    let resolver = |_: &str| Ok(BASE.to_owned());
    let unused = composition::compile(
        source,
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(unused.imports.len(), 1);
    assert!(
        unused.lowering_diagnostics.is_empty(),
        "{:?}",
        unused.lowering_diagnostics
    );
    assert!(unused.evidence.is_empty());
    let applied = composition::compile(
        &format!("{source}apply one = build()\n"),
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        applied.lowering_diagnostics.is_empty(),
        "{:?}",
        applied.lowering_diagnostics
    );
    assert_eq!(
        applied.evidence["one.lib.first"].tool.as_deref(),
        Some("one.lib.collector")
    );
    assert_eq!(applied.source_files["one.lib.first"], "library.eal");
}

#[test]
fn imported_pattern_bodies_keep_definition_relative_imports_and_origins() {
    let sources = BTreeMap::from([
        (
            "lib/outer.eal".to_owned(),
            "language \"EAL/3\"\npattern make() {\nimport \"inner.eal\" as inside\n}\n".to_owned(),
        ),
        ("lib/inner.eal".to_owned(), BASE.to_owned()),
    ]);
    let resolver = |path: &str| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| "unknown source".into())
    };
    let source =
        "language \"EAL/3\"\nimport \"lib/outer.eal\" as outer\napply one = outer.make()\n";
    let p = composition::compile(
        source,
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        p.lowering_diagnostics.is_empty(),
        "{:?}",
        p.lowering_diagnostics
    );
    assert_eq!(p.source_files["one.inside.first"], "lib/inner.eal");
    assert_eq!(p.evidence["one.inside.first"].environment, "one.inside.lab");
    assert_eq!(p.imports.len(), 2);
    assert_eq!(composition::relower(&p).unwrap().evidence, p.evidence);
}

#[test]
fn raised_host_budgets_cannot_remove_native_import_recursion_bounds() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for nested in [false, true] {
                let resolver = move |path: &str| {
                    let index = path
                        .strip_prefix('s')
                        .and_then(|name| name.strip_suffix(".eal"))
                        .and_then(|index| index.parse::<usize>().ok())
                        .ok_or_else(|| format!("Unexpected source {path}"))?;
                    let import = format!("import \"s{}.eal\" as next\n", index + 1);
                    let body = if nested {
                        format!("module inner {{\n{import}}}\n")
                    } else {
                        import
                    };
                    Ok(format!("language \"EAL/3\"\n{body}"))
                };
                let limits = ExecutionLimits {
                    expansion_depth: usize::MAX,
                    ..ExecutionLimits::default()
                };
                let error = composition::compile(
                    "language \"EAL/3\"\nimport \"s0.eal\" as initial\n",
                    Some(&resolver),
                    "root.eal",
                    limits,
                )
                .unwrap_err();
                assert!(error.to_string().contains("resource_limit"), "{error}");
                assert!(
                    error.to_string().contains("depth budget exceeded"),
                    "{error}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn raised_host_depth_still_allows_iterative_pattern_expansion() {
    let source = format!(
        "{BASE}pattern each(c: claim, r: reasoning, es: evidence[]) decreases es {{\nwhen es {{\nargument item = [evidence head(es)] via r => c\napply rest = each(c=c, r=r, es=tail(es))\n}}\n}}\napply checks = each(c=passed, r=measured, es=[{}])\n",
        vec!["first"; 129].join(", ")
    );
    let limits = ExecutionLimits {
        expansion_depth: usize::MAX,
        ..ExecutionLimits::default()
    };
    let program = composition::compile(&source, None, "", limits).unwrap();
    assert!(
        program.lowering_diagnostics.is_empty(),
        "{:?}",
        program.lowering_diagnostics
    );
    assert_eq!(program.arguments.len(), 129);
    assert_eq!(program.applications.len(), 130);
}
