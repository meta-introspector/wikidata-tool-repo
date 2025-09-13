use anyhow::{Result, Context};
use git2::{Repository, Oid, Commit, DiffOptions, DiffFormat, DiffLineType};
use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref CRQ_REGEX: Regex = Regex::new(r"CRQ-\d+").unwrap();
}

fn main() -> Result<()> {
    let repo_path = PathBuf::from("."); // Current directory is the wikidata-tool submodule
    let repo = Repository::open(&repo_path)
        .context(format!("Failed to open repository at: {}", repo_path.display()))?;

    let cache_dir = repo_path.join("cache");
    fs::create_dir_all(&cache_dir)
        .context(format!("Failed to create cache directory: {}", cache_dir.display()))?;

    let last_scanned_commit_path = cache_dir.join("last_scanned_commit.txt");
    let mut last_scanned_commit_oid: Option<Oid> = None;

    if last_scanned_commit_path.exists() {
        let oid_str = fs::read_to_string(&last_scanned_commit_path)
            .context("Failed to read last_scanned_commit.txt")?;
        last_scanned_commit_oid = Some(Oid::from_str(oid_str.trim())
            .context("Failed to parse OID from last_scanned_commit.txt")?);
    }

    let head_commit = repo.head()?.peel_to_commit()?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit.id())?;

    if let Some(last_oid) = last_scanned_commit_oid {
        revwalk.hide(last_oid)?;
    }

    println!("Scanning for CRQ links since last scan...");

    let mut found_crq_links = Vec::new();
    let mut current_head_oid = head_commit.id();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Compare with parent to get changes in this commit
        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let diff = repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?;

            diff.print(DiffFormat::Patch, |delta, hunk, line| {
                if line.new_file_line_number() > 0 && (line.origin() == DiffLineType::Addition || line.origin() == DiffLineType::Context) {
                    if let Some(content) = std::str::from_utf8(line.content()).ok() {
                        for mat in CRQ_REGEX.find_iter(content) {
                            found_crq_links.push(mat.as_str().to_string());
                        }
                    }
                }
                Ok(true)
            })?;
        } else {
            // Initial commit, diff against empty tree
            let diff = repo.diff_tree_to_tree(None, Some(&commit.tree()?), None)?;
            diff.print(DiffFormat::Patch, |delta, hunk, line| {
                if line.new_file_line_number() > 0 && (line.origin() == DiffLineType::Addition || line.origin() == DiffLineType::Context) {
                    if let Some(content) = std::str::from_utf8(line.content()).ok() {
                        for mat in CRQ_REGEX.find_iter(content) {
                            found_crq_links.push(mat.as_str().to_string());
                        }
                    }
                }
                Ok(true)
            })?;
        }
    }

    if found_crq_links.is_empty() {
        println!("No new CRQ links found.");
    } else {
        println!("Found CRQ links:");
        for link in found_crq_links {
            println!("- {}", link);
        }
    }

    // Update last scanned commit
    fs::write(&last_scanned_commit_path, head_commit.id().to_string())
        .context("Failed to write last_scanned_commit.txt")?;

    println!("Updated last scanned commit to: {}", head_commit.id());

    Ok(())
}