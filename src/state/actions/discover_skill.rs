use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
}

/// Discover skills by finding SKILL.md files in the repo.
/// Searches both root-level dirs and `skills/` subdirectory.
pub fn discover_skills(dir: &Path) -> Result<Vec<DiscoveredSkill>, std::io::Error> {
    let mut skills = Vec::new();
    let mut seen = HashSet::new();

    // Check `skills/` subdirectory first (preferred convention)
    let skills_dir = dir.join("skills");
    if skills_dir.is_dir() {
        scan_for_skills(&skills_dir, &mut skills, &mut seen)?;
    }

    // Also check root-level directories
    scan_for_skills(dir, &mut skills, &mut seen)?;

    Ok(skills)
}

fn scan_for_skills(
    parent: &Path,
    skills: &mut Vec<DiscoveredSkill>,
    seen: &mut HashSet<String>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        if path.join("SKILL.md").exists() && seen.insert(name.clone()) {
            skills.push(DiscoveredSkill { name, path });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_dir_returns_no_skills() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert!(skills.is_empty());
    }

    #[test]
    fn files_are_skipped() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        fs::write(tmp.path().join("not-a-dir"), "").expect("write file");
        fs::write(tmp.path().join("SKILL.md"), "").expect("write SKILL.md");

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert!(skills.is_empty());
    }

    #[test]
    fn finds_skill_with_skill_md() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let skill_dir = tmp.path().join("my-skill");
        fs::create_dir(&skill_dir).expect("create skill dir");
        fs::write(skill_dir.join("SKILL.md"), "# My Skill").expect("write SKILL.md");

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
        assert_eq!(skills[0].path, skill_dir);
    }

    #[test]
    fn dir_without_skill_md_is_skipped() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        fs::create_dir(tmp.path().join("no-marker")).expect("create dir");

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert!(skills.is_empty());
    }

    #[test]
    fn hidden_dirs_are_skipped() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let hidden = tmp.path().join(".hidden-skill");
        fs::create_dir(&hidden).expect("create hidden dir");
        fs::write(hidden.join("SKILL.md"), "").expect("write SKILL.md");

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert!(skills.is_empty());
    }

    #[test]
    fn deduplicates_skills_subdir_over_root() {
        let tmp = tempfile::tempdir().expect("create temp dir");

        // Skill in skills/ subdir (preferred)
        let skills_dir = tmp.path().join("skills");
        fs::create_dir(&skills_dir).expect("create skills dir");
        let sub = skills_dir.join("dup-skill");
        fs::create_dir(&sub).expect("create skills/dup-skill");
        fs::write(sub.join("SKILL.md"), "from skills/").expect("write SKILL.md");

        // Same name at root level
        let root = tmp.path().join("dup-skill");
        fs::create_dir(&root).expect("create root dup-skill");
        fs::write(root.join("SKILL.md"), "from root").expect("write SKILL.md");

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "dup-skill");
        assert_eq!(skills[0].path, sub, "skills/ subdir should win over root");
    }

    #[test]
    fn finds_multiple_skills() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        for name in ["alpha", "beta", "gamma"] {
            let d = tmp.path().join(name);
            fs::create_dir(&d).expect("create dir");
            fs::write(d.join("SKILL.md"), "").expect("write SKILL.md");
        }

        let skills = discover_skills(tmp.path()).expect("discover_skills");
        let mut names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }
}
