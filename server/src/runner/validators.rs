pub fn validate_deps_input(deps: &[String]) -> Result<(), String> {
    for dep in deps {
        validate_dependency_name(&dep)
            .map_err(|err| format!("Invalid dependency name '{dep}': {err}"))?;
    }
    Ok(())
}

/// Validate a dependency name so it is safe to interpolate into a Dockerfile `RUN` shell command.
///
/// Allowed characters should cover the needs of most package managers while
/// excluding shell metacharacters that could be used for injection:
///
/// - Alphanumerics and `_`, `-`, `.` — universal package name characters
/// - `/` — required by Go module paths (e.g. `github.com/pkg/errors`)
/// - `:` — used in some Go and cargo specifiers
/// - `@` — npm scoped packages (`@types/node`) and version pins (`lodash@4.17`)
/// - `^`, `~`, `>`, `<`, `=` — semver range operators used by npm/pnpm
/// - `[]` - used by Python
fn validate_dependency_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("dependency name must not be empty".into());
    }

    let invalid_char = name.chars().find(|c| !is_safe_dep_char(*c));

    if let Some(ch) = invalid_char {
        return Err(format!(
            "dependency name contains forbidden character {:?} — \
             only alphanumerics and _ - . / : @ ^ ~ > < = [ ] are allowed",
            ch,
        ));
    }

    Ok(())
}

fn is_safe_dep_char(c: char) -> bool {
    c.is_alphanumeric()
        || matches!(
            c,
            '_' | '-' | '.' | '/' | ':' | '@' | '^' | '~' | '>' | '<' | '=' | '[' | ']'
        )
}

#[cfg(test)]
mod tests {
    use super::validate_dependency_name;

    #[test]
    fn accepts_valid_names() {
        let valid = [
            "requests",
            "my-package",
            "my_package",
            "package.js",
            "@types/node",
            "lodash@4.17.21",
            "github.com/pkg/errors",
            "tokio@^1.0",
            "semver@>=7.0.0",
            "serde:derive",
        ];
        for name in valid {
            assert!(
                validate_dependency_name(name).is_ok(),
                "expected '{name}' to be valid"
            );
        }
    }

    #[test]
    fn rejects_shell_metacharacters() {
        let invalid = [
            "pkg; rm -rf /",
            "pkg && curl http://evil.com",
            "pkg | cat /etc/passwd",
            "$(evil)",
            "`evil`",
            "pkg\nnewline",
            "has space",
            "pkg > /etc/cron",
        ];
        for name in invalid {
            assert!(
                validate_dependency_name(name).is_err(),
                "expected '{name}' to be rejected"
            );
        }
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_dependency_name("").is_err());
    }
}
