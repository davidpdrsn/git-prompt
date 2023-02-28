//! Print something like `[master@484c4c9670bb2] ` for shell prompt

use colorful::*;
use git2::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

macro_rules! print_and_exit {
    ($s:expr) => {{
        print!("{}", $s);
        std::process::exit(0)
    }};
}

fn main() -> Result<()> {
    let repo = Repository::open_from_env().unwrap_or_else(|_| print_and_exit!(""));

    let out = OutputData::new(repo)?;

    let rest = vec![
        out.rev().map(|s| s.yellow()),
        out.status().map(|s| s.red()),
        out.state().map(|s| s.green()),
    ]
    .into_iter()
    .flatten()
    .map(|s| s.to_string())
    .collect::<Vec<_>>()
    .join(" ");

    print!("[{} @ {}] ", out.branch().blue(), rest,);

    Ok(())
}

struct OutputData {
    repo: Repository,
}

impl OutputData {
    fn new(repo: Repository) -> Result<Self> {
        Ok(Self { repo })
    }

    fn branch(&self) -> String {
        self.head()
            .shorthand()
            .unwrap_or_else(|| print_and_exit!("???"))
            .to_string()
    }

    fn rev(&self) -> Option<String> {
        let rev = self.head().target().expect("rev").to_string()[0..13].to_string();
        Some(rev)
    }

    fn state(&self) -> Option<String> {
        use git2::RepositoryState::*;

        let s = match self.repo.state() {
            Clean => None,
            Merge => Some("merge"),
            Revert => Some("revert"),
            RevertSequence => Some("revert"),
            CherryPick => Some("cherry-pick"),
            CherryPickSequence => Some("cherry-pick"),
            Bisect => Some("bisect"),
            Rebase => Some("rebase"),
            RebaseInteractive => Some("rebase"),
            RebaseMerge => Some("rebase-merge"),
            ApplyMailbox => Some("apply-mailbox"),
            ApplyMailboxOrRebase => Some("apply-mailbox"),
        };

        s.map(|s| s.to_string())
    }

    fn status(&self) -> Option<String> {
        use git2::Delta::*;

        let mut out = String::new();
        let mut seen = Vec::new();

        if let Ok(status) = self.repo.statuses(None) {
            for status in status.iter() {
                if let Some(status) = status.index_to_workdir() {
                    let delta = status.status();

                    if seen.contains(&delta) {
                        continue;
                    }
                    seen.push(delta);

                    match delta {
                        Unmodified => {}
                        Added => out.push('+'),
                        Deleted => out.push('-'),
                        Modified | Renamed | Typechange => out.push('*'),
                        Copied => {}
                        Ignored => {}
                        Untracked => out.push('?'),
                        Unreadable => {}
                        Conflicted => out.push('#'),
                    }
                }
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn head(&self) -> Reference {
        self.repo
            .head()
            .unwrap_or_else(|_| print_and_exit!("[no head] ".red()))
    }
}
