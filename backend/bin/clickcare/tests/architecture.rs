//! Guardián de la arquitectura cebolla (ver `docs/arquitectura_cebolla.md`, regla 2 y sección 12).
//!
//! Revisa los `use` de nivel superior y los `derive` de `domain/` y `application/` en cada crate
//! del workspace. Cada excepción vive en `EXCEPTIONS` con su motivo. Si una excepción deja de
//! aplicar, el test falla para obligar a borrarla.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Librerías externas permitidas en `domain/` y `application/` (regla 2 del documento).
const ALLOWED_EXTERNAL: &[&str] = &[
    "log",
    "tracing",
    "uuid",
    "uuid_v7",
    "derive_getters",
    "bon",
    "strum_macros",
    "thiserror",
    "async_trait",
    "pulldown_cmark",
    "sha2",
];

/// Crate compartido del workspace que el dominio puede usar (su nombre de paquete es `core`).
const ALLOWED_WORKSPACE: &[&str] = &["app_core"];

/// Prefijos de `std` prohibidos (acceso a disco, procesos y red).
const FORBIDDEN_STD: &[&str] = &["std::fs", "std::process", "std::net"];

/// Excepciones temporales: (archivo relativo a `crates/`, clave de la violación, motivo).
/// La clave es `use:<crate>` o `derive:serde`. Borrar la entrada al corregir el código.
/// Sin excepciones: las Fases 0 a 3 cerraron todas las violaciones de la regla 2.
const EXCEPTIONS: &[(&str, &str, &str)] = &[];

#[test]
fn inner_layers_follow_onion_rules() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates");

    let mut violations: BTreeSet<(String, String)> = BTreeSet::new();
    for krate in fs::read_dir(&crates_dir).expect("crates/ debe existir") {
        let krate = krate.expect("entrada de crates/").path();
        let own_name = krate
            .file_name()
            .expect("nombre de crate")
            .to_string_lossy()
            .into_owned();
        for layer in ["domain", "application"] {
            let mut files = Vec::new();
            collect_rs_files(&krate.join("src").join(layer), &mut files);
            for file in files {
                let rel = file
                    .strip_prefix(&crates_dir)
                    .expect("archivo dentro de crates/")
                    .to_string_lossy()
                    .replace('\\', "/");
                let content = fs::read_to_string(&file).expect("leer fuente");
                let ctx = Context {
                    layer,
                    own_name: &own_name,
                    dir: file.parent().expect("directorio del archivo"),
                };
                for key in violations_in(&content, &ctx) {
                    violations.insert((rel.clone(), key));
                }
            }
        }
    }

    let unexcused: Vec<String> = violations
        .iter()
        .filter(|(rel, key)| !EXCEPTIONS.iter().any(|(f, k, _)| f == rel && k == key))
        .map(|(rel, key)| format!("{rel} -> {key}"))
        .collect();

    let stale: Vec<String> = EXCEPTIONS
        .iter()
        .filter(|(f, k, _)| !violations.contains(&(f.to_string(), k.to_string())))
        .map(|(f, k, motivo)| format!("{f} ({k}) ya no aplica, borrar la excepción: {motivo}"))
        .collect();

    assert!(
        unexcused.is_empty() && stale.is_empty(),
        "violaciones sin excepción:\n{}\nexcepciones obsoletas:\n{}",
        unexcused.join("\n"),
        stale.join("\n")
    );
}

struct Context<'a> {
    layer: &'a str,
    own_name: &'a str,
    dir: &'a Path,
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Devuelve las claves de violación del archivo. Solo mira `use` y `derive` de nivel superior:
/// los `use` indentados pertenecen a módulos de prueba (`#[cfg(test)]`) y se ignoran.
fn violations_in(content: &str, ctx: &Context) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for line in content.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        if let Some(path) = use_path(line)
            && let Some(key) = classify_use(path, ctx)
        {
            found.insert(key);
        }
        let derives_serde = line.trim_start().starts_with("#[derive(")
            && (line.contains("Serialize") || line.contains("Deserialize"));
        if derives_serde {
            found.insert("derive:serde".to_string());
        }
    }
    found
}

/// Extrae la ruta de un `use` de nivel superior, por ejemplo `serde::Deserialize`.
fn use_path(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub ").unwrap_or(line);
    let rest = rest.strip_prefix("use ")?;
    Some(rest.trim_end_matches(';').trim())
}

/// Clasifica una ruta importada. Devuelve la clave de violación o `None` si está permitida.
/// Las capas internas no pueden importar `crate::infrastructure`, y `domain` tampoco puede
/// importar `crate::application`.
fn classify_use(path: &str, ctx: &Context) -> Option<String> {
    let first = path
        .split(|c: char| c == ':' || c == '{' || c == ';' || c.is_whitespace())
        .next()
        .unwrap_or_default();

    if first == "crate" {
        let forbidden: &[&str] = if ctx.layer == "domain" {
            &["crate::application", "crate::infrastructure"]
        } else {
            &["crate::infrastructure"]
        };
        return forbidden
            .iter()
            .find(|p| path.starts_with(*p))
            .map(|p| format!("use:{p}"));
    }

    match first {
        "" | "self" | "super" | "core" | "alloc" => None,
        "std" => FORBIDDEN_STD
            .iter()
            .find(|p| path.starts_with(*p))
            .map(|p| format!("use:{p}")),
        krate if krate == ctx.own_name || is_sibling_module(ctx.dir, krate) => None,
        krate if ALLOWED_EXTERNAL.contains(&krate) || ALLOWED_WORKSPACE.contains(&krate) => None,
        krate => Some(format!("use:{krate}")),
    }
}

/// Un `use` de un módulo hermano (`pub use human_name::HumanName;` en `fhir/mod.rs`) es interno.
fn is_sibling_module(dir: &Path, name: &str) -> bool {
    dir.join(format!("{name}.rs")).exists() || dir.join(name).is_dir()
}
