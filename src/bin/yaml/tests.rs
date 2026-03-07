use super::*;
use std::io::Write as IoWrite;

fn temp_yaml(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn temp_dir_with(defaults: &str, config: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(dir.path().join("defaults.yaml"), defaults).unwrap();
    if let Some(cfg) = config {
        fs::write(dir.path().join("config.yaml"), cfg).unwrap();
    }
    dir
}

// --- parse_path ---

#[test]
fn parse_simple_key() {
    let segs = parse_path("agents");
    assert_eq!(segs.len(), 1);
    assert!(matches!(&segs[0], PathSegment::Key(k) if k == "agents"));
}

#[test]
fn parse_dotted_path() {
    let segs = parse_path(".skills.claude");
    assert_eq!(segs.len(), 2);
    assert!(matches!(&segs[0], PathSegment::Key(k) if k == "skills"));
    assert!(matches!(&segs[1], PathSegment::Key(k) if k == "claude"));
}

#[test]
fn parse_leading_dot_optional() {
    let a = parse_path(".agents");
    let b = parse_path("agents");
    assert_eq!(a.len(), b.len());
}

#[test]
fn parse_array_index() {
    let segs = parse_path(".modules[0]");
    assert_eq!(segs.len(), 2);
    assert!(matches!(&segs[0], PathSegment::Key(k) if k == "modules"));
    assert!(matches!(&segs[1], PathSegment::Index(0)));
}

#[test]
fn parse_array_index_deep() {
    let segs = parse_path(".items[2].name");
    assert_eq!(segs.len(), 3);
    assert!(matches!(&segs[0], PathSegment::Key(k) if k == "items"));
    assert!(matches!(&segs[1], PathSegment::Index(2)));
    assert!(matches!(&segs[2], PathSegment::Key(k) if k == "name"));
}

#[test]
fn parse_multi_index() {
    let segs = parse_path(".matrix[0][1]");
    assert_eq!(segs.len(), 3);
    assert!(matches!(&segs[0], PathSegment::Key(k) if k == "matrix"));
    assert!(matches!(&segs[1], PathSegment::Index(0)));
    assert!(matches!(&segs[2], PathSegment::Index(1)));
}

#[test]
fn parse_empty_path() {
    let segs = parse_path("");
    assert!(segs.is_empty());
}

#[test]
fn parse_dot_only() {
    let segs = parse_path(".");
    assert!(segs.is_empty());
}

// --- walk ---

#[test]
fn walk_single_key() {
    let f = temp_yaml("name: forge-test\n");
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".name")).unwrap();
    assert_eq!(as_str(&v), "forge-test");
}

#[test]
fn walk_nested_key() {
    let f = temp_yaml("user:\n  root: Vaults/Personal\n");
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".user.root")).unwrap();
    assert_eq!(as_str(&v), "Vaults/Personal");
}

#[test]
fn walk_deep_nesting() {
    let f = temp_yaml("a:\n  b:\n    c:\n      d: value\n");
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".a.b.c.d")).unwrap();
    assert_eq!(as_str(&v), "value");
}

#[test]
fn walk_array_index() {
    let f = temp_yaml("modules:\n  - alpha\n  - beta\n  - gamma\n");
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".modules[1]")).unwrap();
    assert_eq!(as_str(&v), "beta");
}

#[test]
fn walk_array_nested() {
    let f = temp_yaml("items:\n  - name: first\n    val: 1\n  - name: second\n    val: 2\n");
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".items[1].name")).unwrap();
    assert_eq!(as_str(&v), "second");
}

#[test]
fn walk_missing_returns_none() {
    let f = temp_yaml("name: test\n");
    let doc = load(f.path().to_str().unwrap());
    assert!(walk(&doc, &parse_path(".nonexistent")).is_none());
}

#[test]
fn walk_missing_nested_returns_none() {
    let f = temp_yaml("a:\n  b: value\n");
    let doc = load(f.path().to_str().unwrap());
    assert!(walk(&doc, &parse_path(".a.c")).is_none());
}

#[test]
fn walk_out_of_bounds_returns_none() {
    let f = temp_yaml("items:\n  - one\n  - two\n");
    let doc = load(f.path().to_str().unwrap());
    assert!(walk(&doc, &parse_path(".items[5]")).is_none());
}

// --- keys ---

#[test]
fn keys_top_level() {
    let f = temp_yaml("agents:\n  Foo:\n    model: fast\n  Bar:\n    model: strong\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".agents")) {
        let keys: Vec<String> = map.keys().map(as_str).collect();
        assert_eq!(keys, vec!["Foo", "Bar"]);
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn keys_nested() {
    let f = temp_yaml("skills:\n  claude:\n    SkillA:\n      scope: ws\n    SkillB:\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".skills.claude")) {
        let keys: Vec<String> = map.keys().map(as_str).collect();
        assert_eq!(keys, vec!["SkillA", "SkillB"]);
    } else {
        panic!("expected mapping");
    }
}

// --- value (scalar) ---

#[test]
fn value_scalar() {
    let f = temp_yaml("name: forge-test\nversion: 0.1.0\n");
    let doc = load(f.path().to_str().unwrap());
    assert_eq!(
        as_str(&walk(&doc, &parse_path(".name")).unwrap()),
        "forge-test"
    );
    assert_eq!(
        as_str(&walk(&doc, &parse_path(".version")).unwrap()),
        "0.1.0"
    );
}

#[test]
fn value_nested_scalar() {
    let f = temp_yaml("user:\n  root: Vaults/Personal\n  name: test\n");
    let doc = load(f.path().to_str().unwrap());
    assert_eq!(
        as_str(&walk(&doc, &parse_path(".user.root")).unwrap()),
        "Vaults/Personal"
    );
}

// --- list ---

#[test]
fn list_block_syntax() {
    let f = temp_yaml("modules:\n  - alpha\n  - beta\n  - gamma\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Sequence(items)) = walk(&doc, &parse_path(".modules")) {
        let strs: Vec<String> = items.iter().map(as_str).collect();
        assert_eq!(strs, vec!["alpha", "beta", "gamma"]);
    } else {
        panic!("expected sequence");
    }
}

#[test]
fn list_flow_syntax() {
    let f = temp_yaml("events: [SessionStart, PreToolUse]\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Sequence(items)) = walk(&doc, &parse_path(".events")) {
        let strs: Vec<String> = items.iter().map(as_str).collect();
        assert_eq!(strs, vec!["SessionStart", "PreToolUse"]);
    } else {
        panic!("expected sequence");
    }
}

// --- map ---

#[test]
fn map_scalar_values() {
    let f = temp_yaml("user:\n  root: Vaults/Personal\n  name: test\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".user")) {
        assert_eq!(as_str(map.get("root").unwrap()), "Vaults/Personal");
        assert_eq!(as_str(map.get("name").unwrap()), "test");
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn map_list_values() {
    let f = temp_yaml("commands:\n  hooks: [pre, post]\n  run: test\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".commands")) {
        match map.get("hooks").unwrap() {
            Value::Sequence(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(as_str(&items[0]), "pre");
            }
            _ => panic!("expected sequence"),
        }
    } else {
        panic!("expected mapping");
    }
}

// --- edge cases ---

#[test]
fn missing_file_returns_empty_mapping() {
    let doc = load("/nonexistent/path.yaml");
    assert!(doc.is_mapping());
    assert!(doc.as_mapping().unwrap().is_empty());
}

#[test]
fn quoted_values_stripped() {
    assert_eq!(strip_quotes("\"hello\""), "hello");
    assert_eq!(strip_quotes("'world'"), "world");
    assert_eq!(strip_quotes("plain"), "plain");
}

// --- realistic forge patterns ---

#[test]
fn forge_agents_keys() {
    let yaml = "\
agents:
    SoftwareDeveloper:
        model: fast
        tools: Read, Grep, Glob
    DatabaseEngineer:
        model: fast
        tools: Read, Grep, Glob, Bash
    TheOpponent:
        model: strong
        tools: Read, Grep, Glob, WebSearch
";
    let f = temp_yaml(yaml);
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".agents")) {
        let keys: Vec<String> = map.keys().map(as_str).collect();
        assert_eq!(
            keys,
            vec!["SoftwareDeveloper", "DatabaseEngineer", "TheOpponent"]
        );
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn forge_skills_claude_keys() {
    let yaml = "\
skills:
    claude:
        DebateCouncil:
            scope: workspace
        DeveloperCouncil:
            scope: workspace
        HiringCouncil:
            scope: workspace
    gemini:
        - DebateCouncil
        - DeveloperCouncil
";
    let f = temp_yaml(yaml);
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".skills.claude")) {
        let keys: Vec<String> = map.keys().map(as_str).collect();
        assert_eq!(
            keys,
            vec!["DebateCouncil", "DeveloperCouncil", "HiringCouncil"]
        );
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn forge_agent_nested_value() {
    let yaml = "\
agents:
    SoftwareDeveloper:
        model: fast
        tools: Read, Grep, Glob
";
    let f = temp_yaml(yaml);
    let doc = load(f.path().to_str().unwrap());
    let v = walk(&doc, &parse_path(".agents.SoftwareDeveloper.model")).unwrap();
    assert_eq!(as_str(&v), "fast");
}

// --- env ---

#[test]
fn env_flat_mapping() {
    let f = temp_yaml("commands:\n  calendar: bin/calendar.sh\n  safe_read: bin/safe-read\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".commands")) {
        let lines = flatten_env(&map, "FORGE");
        assert_eq!(
            lines,
            vec![
                "export FORGE_CALENDAR='bin/calendar.sh'",
                "export FORGE_SAFE_READ='bin/safe-read'",
            ]
        );
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn env_nested_mapping() {
    let f = temp_yaml("a:\n  b:\n    c: deep\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".a")) {
        let lines = flatten_env(&map, "FORGE");
        assert_eq!(lines, vec!["export FORGE_B_C='deep'"]);
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn env_custom_prefix() {
    let f = temp_yaml("tools:\n  git: /usr/bin/git\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".tools")) {
        let lines = flatten_env(&map, "MY_PREFIX");
        assert_eq!(lines, vec!["export MY_PREFIX_GIT='/usr/bin/git'"]);
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn env_default_prefix() {
    let f = temp_yaml("name: test\n");
    let doc = load(f.path().to_str().unwrap());
    if let Value::Mapping(ref map) = doc {
        let lines = flatten_env(map, "FORGE");
        assert_eq!(lines, vec!["export FORGE_NAME='test'"]);
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn env_skips_sequences() {
    let f = temp_yaml("items:\n  list:\n    - a\n    - b\n  name: test\n");
    let doc = load(f.path().to_str().unwrap());
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".items")) {
        let lines = flatten_env(&map, "FORGE");
        assert_eq!(lines, vec!["export FORGE_NAME='test'"]);
    } else {
        panic!("expected mapping");
    }
}

// --- deep_merge ---

#[test]
fn merge_scalar_override() {
    let mut base: Value = serde_yaml::from_str("adr:\n  directory: docs/decisions\n  prefix: date\n").unwrap();
    let overlay: Value = serde_yaml::from_str("adr:\n  directory: Inbox/ADR\n").unwrap();
    deep_merge(&mut base, &overlay);
    let v = walk(&base, &parse_path(".adr.directory")).unwrap();
    assert_eq!(as_str(&v), "Inbox/ADR");
    let v = walk(&base, &parse_path(".adr.prefix")).unwrap();
    assert_eq!(as_str(&v), "date");
}

#[test]
fn merge_adds_new_keys() {
    let mut base: Value = serde_yaml::from_str("a:\n  x: 1\n").unwrap();
    let overlay: Value = serde_yaml::from_str("a:\n  y: 2\n").unwrap();
    deep_merge(&mut base, &overlay);
    assert_eq!(as_str(&walk(&base, &parse_path(".a.x")).unwrap()), "1");
    assert_eq!(as_str(&walk(&base, &parse_path(".a.y")).unwrap()), "2");
}

#[test]
fn merge_empty_overlay_is_noop() {
    let mut base: Value = serde_yaml::from_str("key: value\n").unwrap();
    let overlay = Value::Mapping(Mapping::default());
    deep_merge(&mut base, &overlay);
    assert_eq!(as_str(&walk(&base, &parse_path(".key")).unwrap()), "value");
}

// --- env directory mode ---

#[test]
fn env_dir_defaults_only() {
    let dir = temp_dir_with("adr:\n  directory: docs/decisions\n  prefix: date\n", None);
    let dir_path = dir.path().to_str().unwrap().to_string();
    let doc = {
        let mut base = load(&format!("{dir_path}/defaults.yaml"));
        let overlay = load(&format!("{dir_path}/config.yaml"));
        deep_merge(&mut base, &overlay);
        base
    };
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".adr")) {
        let lines = flatten_env(&map, "FORGE_ADR");
        assert!(lines.contains(&"export FORGE_ADR_DIRECTORY='docs/decisions'".to_string()));
        assert!(lines.contains(&"export FORGE_ADR_PREFIX='date'".to_string()));
    } else {
        panic!("expected mapping");
    }
}

#[test]
fn env_dir_config_overrides() {
    let dir = temp_dir_with(
        "adr:\n  directory: docs/decisions\n  prefix: date\n",
        Some("adr:\n  directory: Inbox/ADR\n"),
    );
    let dir_path = dir.path().to_str().unwrap().to_string();
    let doc = {
        let mut base = load(&format!("{dir_path}/defaults.yaml"));
        let overlay = load(&format!("{dir_path}/config.yaml"));
        deep_merge(&mut base, &overlay);
        base
    };
    if let Some(Value::Mapping(map)) = walk(&doc, &parse_path(".adr")) {
        let lines = flatten_env(&map, "FORGE_ADR");
        assert!(lines.contains(&"export FORGE_ADR_DIRECTORY='Inbox/ADR'".to_string()));
        assert!(lines.contains(&"export FORGE_ADR_PREFIX='date'".to_string()));
    } else {
        panic!("expected mapping");
    }
}
