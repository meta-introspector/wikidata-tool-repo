use anyhow::{Result, Context};
use git2::{Repository, Oid, Commit, DiffOptions, DiffFormat, DiffLineType};
use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;
use lazy_static::lazy_static;
use clap::Parser;
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref CRQ_REGEX: Regex = Regex::new(r"CRQ-\d+").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"https?://[\w./?=#&%~-]+[^\.,\s\n\r)]").unwrap();
    static ref WORD_REGEX: Regex = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the repository to scan (defaults to current directory)
    #[arg(long, default_value = ".")]
    repo_to_scan_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ScanCache {
    last_scanned_commit: Option<String>,
    found_crq_links: Vec<String>,
    found_urls: Vec<String>,
    found_terms: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo_to_scan_path = args.repo_to_scan_path;

    let repo = Repository::open(&repo_to_scan_path)
        .context(format!("Failed to open repository at: {}", repo_to_scan_path.display()))?;

    let cache_dir = repo_to_scan_path.join(".wikidata_cache"); // Use a hidden directory for cache
    fs::create_dir_all(&cache_dir)
        .context(format!("Failed to create cache directory: {}", cache_dir.display()))?;

    let cache_file_path = cache_dir.join("scan_cache.json");
    let mut scan_cache: ScanCache = if cache_file_path.exists() {
        let cache_content = fs::read_to_string(&cache_file_path)
            .context("Failed to read scan_cache.json")?;
        serde_json::from_str(&cache_content)
            .context("Failed to deserialize scan_cache.json")?
    } else {
        ScanCache::default()
    };

    let last_scanned_commit_oid: Option<Oid> = scan_cache.last_scanned_commit.as_ref()
        .and_then(|s| Oid::from_str(s).ok());

    let head_commit = repo.head()?.peel_to_commit()?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit.id())?;

    if let Some(last_oid) = last_scanned_commit_oid {
        revwalk.hide(last_oid)?;
    }

    println!("Scanning for changes since last scan...");

    let mut new_crq_links = Vec::new();
    let mut new_urls = Vec::new();
    let mut new_terms = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Compare with parent to get changes in this commit
        let diff = if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?;
        } else {
            // Initial commit, diff against empty tree
            repo.diff_tree_to_tree(None, Some(&commit.tree()?), None)?;
        };

        diff.print(DiffFormat::Patch, |delta, hunk, line| {
            if line.new_file_line_number() > 0 && (line.origin() == DiffLineType::Addition || line.origin() == DiffLineType::Context) {
                if let Some(content) = std::str::from_utf8(line.content()).ok() {
                    // Extract CRQ links
                    for mat in CRQ_REGEX.find_iter(content) {
                        new_crq_links.push(mat.as_str().to_string());
                    }
                    // Extract URLs
                    for mat in URL_REGEX.find_iter(content) {
                        new_urls.push(mat.as_str().to_string());
                    }
                    // Extract terms (simple word extraction for now)
                    for mat in WORD_REGEX.find_iter(content) {
                        new_terms.push(mat.as_str().to_string());
                    }
                }
            }
            Ok(true)
        })?;
    }

    // Aggregate new findings into the cache
    scan_cache.found_crq_links.extend(new_crq_links.into_iter().filter(|link| !scan_cache.found_crq_links.contains(link)));
    scan_cache.found_urls.extend(new_urls.into_iter().filter(|url| !scan_cache.found_urls.contains(url)));
    scan_cache.found_terms.extend(new_terms.into_iter().filter(|term| !scan_cache.found_terms.contains(term)));

    // Update last scanned commit
    scan_cache.last_scanned_commit = Some(head_commit.id().to_string());

    // Write updated cache to file
    let updated_cache_content = serde_json::to_string_pretty(&scan_cache)
        .context("Failed to serialize scan_cache.json")?;
    fs::write(&cache_file_path, updated_cache_content)
        .context("Failed to write scan_cache.json")?;

    println!("Scan complete for repository: {}", repo_to_scan_path.display());
    println!("Total CRQ links found: {}", scan_cache.found_crq_links.len());
    println!("Total URLs found: {}", scan_cache.found_urls.len());
    println!("Total terms found: {}", scan_cache.found_terms.len());

    Ok(())
}
