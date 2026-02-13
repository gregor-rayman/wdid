use git2::Sort;
use rusqlite::{params, Row};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{fs, thread};

use super::Database;
use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitCommit {
    pub id: String,
    pub date: String, // YYYY-MM-DD format
    pub time: String, // HH:MM format
    pub folder: String,
    pub description: String,
}

pub fn spawn_git_worker() -> (Sender<GitCommand>, Receiver<GitResult>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<GitCommand>();
    let (result_tx, result_rx) = mpsc::channel::<GitResult>();

    thread::spawn(move || {
        // Create a new tokio runtime for this thread
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async {
            worker_loop(cmd_rx, result_tx).await;
        });
    });

    (cmd_tx, result_rx)
}

fn find_git_repos_recursive(dir: &Path, repos: &mut Vec<String>) {
    // Check if current directory contains .git
    let git_dir = dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        if let Some(path_str) = dir.to_str() {
            repos.push(path_str.to_string());
        }
        // Don't recurse into subdirectories of a git repo
        return;
    }

    // Recursively search subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    // Skip hidden directories (except .git which we check above)
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('.') {
                            find_git_repos_recursive(&entry.path(), repos);
                        }
                    }
                }
            }
        }
    }
}

fn find_git_repositories(base_path: &str) -> Vec<String> {
    let mut git_repos = Vec::new();

    if let Ok(path) = fs::canonicalize(base_path) {
        find_git_repos_recursive(&path, &mut git_repos);
    }

    git_repos
}

fn read_git_commits(
    repo_path: &str,
    emails: &Vec<String>,
    from_date: &chrono::NaiveDate,
) -> Vec<GitCommit> {
    let from_ts = from_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let repo_name = Path::new(repo_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap_or("unknown");

    let repo = match git2::Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(_) => return Vec::new(),
    };
    let mut revwalk = match repo.revwalk() {
        Ok(walk) => walk,
        Err(_) => return Vec::new(),
    };
    // Sort by time (date order)
    if revwalk.set_sorting(Sort::TIME).is_err() {
        return Vec::new();
    }

    // Start from HEAD
    if revwalk.push_head().is_err() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for oid in revwalk {
        let oid = match oid {
            Ok(oid) => oid,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(commit) => commit,
            Err(_) => continue,
        };
        let author = commit.author();
        for email in emails {
            if author.email().unwrap_or("").contains(email) {
                let commit_time = author.when().seconds();
                if commit_time < from_ts {
                    return result;
                }
                let hash = oid.to_string();
                let commit_time = chrono::DateTime::from_timestamp(commit_time, 0).unwrap();
                let author_date = commit_time.format("%Y-%m-%d").to_string();
                let author_time = commit_time.format("%H:%M").to_string();
                let subject = commit.summary().unwrap_or("");
                if !subject.is_empty() {
                    result.push(GitCommit {
                        id: hash,
                        date: author_date,
                        time: author_time,
                        folder: repo_name.to_string(),
                        description: subject.to_string(),
                    });
                }
                break;
            }
        }
    }

    result
}

pub fn find_git_commits(
    main_folder: String,
    emails: Vec<String>,
    from_date: chrono::NaiveDate,
    result_tx: &Sender<GitResult>
) {
    let git_repos = find_git_repositories(&main_folder);
    for repo in git_repos {
        let commits = read_git_commits(&repo, &emails, &from_date);
        //println!("Found {} commits in {}", commits.len(), repo);
        if !commits.is_empty() {
            result_tx.send(GitResult::CommitsFound(commits)).unwrap();
        }
    }
}

impl Database {

    pub fn save_git_commit(&self, git_commit: &GitCommit) -> Result<usize> {
        let inserted = self.conn().execute(
            r#"INSERT INTO git_commits (id, date, time, folder, description)
               VALUES (?1, ?2, ?3, ?4, ?5) on conflict(id) do nothing"#,
            params![
                git_commit.id,
                git_commit.date,
                git_commit.time,
                git_commit.folder,
                git_commit.description,
            ],
        )?;
        Ok(inserted)
    }

    pub fn get_git_commits_for_date(&self, date: &str) -> Result<Vec<GitCommit>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, time, folder, description
               FROM git_commits WHERE date = ?1
               ORDER BY time, folder, id"#,
        )?;

        let git_commits = stmt
            .query_map([date], |row| Ok(GitCommit::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(git_commits)
    }

    pub fn _delete_git_commit(&self, id: String) -> Result<()> {
        self.conn()
            .execute("DELETE FROM diary_entries WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn _search_git_entry(&self, text: &str) -> Result<Vec<GitCommit>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, time, folder, description
               FROM git_commits
               WHERE description LIKE '%' || ?1 || '%'
               ORDER BY date DESC, time DESC"#,
        )?;

        let git_commit = stmt
            .query_map([text], |row| Ok(GitCommit::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(git_commit)
    }
}

pub enum GitCommand {
    RefreshAll(Vec<String>, Vec<String>),
    Shutdown,
}

pub enum GitResult {
    CommitsFound(Vec<GitCommit>),
    RefreshComplete,
    ShutdownComplete,
}

impl GitCommit {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            date: row.get(1).unwrap(),
            time: row.get(2).unwrap(),
            folder: row.get(3).unwrap(),
            description: row.get(4).unwrap(),
        }
    }
}

async fn worker_loop(cmd_rx: Receiver<GitCommand>, result_tx: Sender<GitResult>) {
    loop {
        match cmd_rx.recv() {
            Ok(GitCommand::RefreshAll(work_folders, emails)) => {
                for folder in work_folders {
                    let today = chrono::Local::now().date_naive();
                    let from_date = today.checked_sub_days(chrono::Days::new(60)).unwrap();
                    find_git_commits(folder, emails.clone(), from_date, &result_tx);
                }
                let _ = result_tx.send(GitResult::RefreshComplete);
            }
            Ok(GitCommand::Shutdown) | Err(_) => {
                // Channel closed or shutdown requested, exit loop
                let _ = result_tx.send(GitResult::ShutdownComplete);
                break;
            }
        }
    }
}
