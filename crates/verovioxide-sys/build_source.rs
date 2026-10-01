use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(source: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .args(arguments)
        .output()
        .map_err(|error| format!("inspect Verovio source {}: {error}", source.display()))?;
    if !output.status.success() {
        return Err(format!(
            "inspect Verovio source {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("read Verovio Git output: {error}"))
}

pub fn require_pinned_source(
    requested: Option<&OsStr>,
    expected_revision: &str,
) -> Result<PathBuf, String> {
    let requested = requested.ok_or_else(|| {
        format!(
            "VEROVIO_SOURCE_DIR is required for this local-only build; point it at the clean Verovio fork at commit {expected_revision}"
        )
    })?;
    let source = canonicalize(Path::new(requested))?;
    let root = canonicalize(Path::new(
        git(&source, &["rev-parse", "--show-toplevel"])?.trim(),
    ))?;
    if source != root || !source.join("src").is_dir() {
        return Err("VEROVIO_SOURCE_DIR must point at the Verovio Git checkout root".into());
    }
    let revision = git(&source, &["rev-parse", "HEAD"])?;
    if revision.trim() != expected_revision {
        return Err(format!(
            "local fingering build requires Verovio commit {expected_revision}, found {}",
            revision.trim()
        ));
    }
    let status = git(
        &source,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignored",
        ],
    )?;
    let changes: Vec<_> = status
        .lines()
        .filter(|line| *line != "!! include/vrv/git_commit.h")
        .collect();
    if !changes.is_empty() {
        return Err(format!(
            "local fingering build requires a clean Verovio source tree:\n{}",
            changes.join("\n")
        ));
    }
    Ok(source)
}

fn canonicalize(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path)
        .map_err(|error| format!("resolve Verovio source {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Checkout(PathBuf);

    impl Checkout {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "verovio-source-guard-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(path.join("src")).unwrap();
            std::fs::create_dir_all(path.join("include/vrv")).unwrap();
            std::fs::write(path.join("src/probe.cpp"), "int probe = 1;\n").unwrap();
            std::fs::write(
                path.join(".gitignore"),
                "include/vrv/git_commit.h\n*.ignored.cpp\n",
            )
            .unwrap();
            git(&path, &["init", "-q"]).unwrap();
            git(&path, &["add", "."]).unwrap();
            git(
                &path,
                &[
                    "-c",
                    "user.name=Source guard",
                    "-c",
                    "user.email=source-guard@example.invalid",
                    "commit",
                    "-qm",
                    "fixture",
                ],
            )
            .unwrap();
            Self(path)
        }

        fn revision(&self) -> String {
            git(&self.0, &["rev-parse", "HEAD"]).unwrap().trim().into()
        }

        fn validate(&self) -> Result<PathBuf, String> {
            require_pinned_source(Some(self.0.as_os_str()), &self.revision())
        }
    }

    impl Drop for Checkout {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn missing_source_fails_closed() {
        let error = require_pinned_source(None, "expected").unwrap_err();
        assert!(error.contains("VEROVIO_SOURCE_DIR is required"));
    }

    #[test]
    fn exact_clean_checkout_is_accepted() {
        let checkout = Checkout::new();
        assert_eq!(
            checkout.validate().unwrap(),
            checkout.0.canonicalize().unwrap()
        );
    }

    #[test]
    fn wrong_revision_is_rejected() {
        let checkout = Checkout::new();
        let error = require_pinned_source(Some(checkout.0.as_os_str()), "wrong").unwrap_err();
        assert!(error.contains("requires Verovio commit wrong"));
    }

    #[test]
    fn tracked_and_staged_changes_are_rejected() {
        let checkout = Checkout::new();
        std::fs::write(checkout.0.join("src/probe.cpp"), "int probe = 2;\n").unwrap();
        assert!(
            checkout
                .validate()
                .unwrap_err()
                .contains(" M src/probe.cpp")
        );
        git(&checkout.0, &["add", "src/probe.cpp"]).unwrap();
        assert!(
            checkout
                .validate()
                .unwrap_err()
                .contains("M  src/probe.cpp")
        );
    }

    #[test]
    fn untracked_and_ignored_sources_are_rejected() {
        let checkout = Checkout::new();
        std::fs::write(checkout.0.join("src/extra.cpp"), "int extra = 1;\n").unwrap();
        assert!(
            checkout
                .validate()
                .unwrap_err()
                .contains("?? src/extra.cpp")
        );
        std::fs::remove_file(checkout.0.join("src/extra.cpp")).unwrap();
        std::fs::write(checkout.0.join("src/extra.ignored.cpp"), "int extra = 1;\n").unwrap();
        assert!(
            checkout
                .validate()
                .unwrap_err()
                .contains("!! src/extra.ignored.cpp")
        );
    }

    #[test]
    fn ignored_old_build_header_is_accepted() {
        let checkout = Checkout::new();
        std::fs::write(
            checkout.0.join("include/vrv/git_commit.h"),
            "old build header\n",
        )
        .unwrap();
        assert_eq!(
            checkout.validate().unwrap(),
            checkout.0.canonicalize().unwrap()
        );
    }
}
