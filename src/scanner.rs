use std::fs;
use std::path::Path;

use rayon::prelude::*;

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub kind: EntryKind,
}

#[derive(Debug)]
pub enum EntryKind {
    File,
    Dir { children: Vec<Entry> },
    Symlink,
}

#[derive(Debug)]
pub struct Stats {
    pub total_size: u64,
    pub file_count: u64,
    pub dir_count:  u64,
}

impl Stats {
    pub fn from_entry(entry: &Entry) -> Stats {
        let mut stats = Stats { total_size: 0, file_count: 0, dir_count: 0 };
        accumulate(&mut stats, entry);
        stats
    }
}

fn accumulate(stats: &mut Stats, entry: &Entry) {
    match &entry.kind {
        EntryKind::File => {
            stats.file_count += 1;
            stats.total_size += entry.size;
        }
        EntryKind::Dir { children } => {
            stats.dir_count += 1;
            for child in children {
                accumulate(stats, child);
            }
        }
        // TODO: Possibly tell user how many symlinks there are.
        EntryKind::Symlink => {}
    }
}

#[derive(Debug)]
pub enum ScanError {
    Io(std::io::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ScanError {}

impl From<std::io::Error> for ScanError {
    fn from(e: std::io::Error) -> Self {
        ScanError::Io(e)
    }
}

pub fn scan(path: &Path, max_depth: Option<usize>) -> Result<Entry, ScanError> {
    // Root has no DirEntry, so we need symlink_metadata here.
    let meta = fs::symlink_metadata(path)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    if meta.file_type().is_symlink() {
        return Ok(Entry { name, size: 0, kind: EntryKind::Symlink });
    }

    if meta.is_file() {
        return Ok(Entry { name, size: meta.len(), kind: EntryKind::File });
    }

    scan_dir(path, name, 0, max_depth)
}

// scan a directory we already know is a directory
fn scan_dir(path: &Path, name: String, depth: usize, max_depth: Option<usize>) -> Result<Entry, ScanError> {
    let at_limit = max_depth.is_some_and(|max| depth >= max);

    if at_limit {
        let size = compute_total_size(path);
        return Ok(Entry { name, size, kind: EntryKind::Dir { children: vec![] } });
    }

    let dir_entries: Vec<fs::DirEntry> = fs::read_dir(path)?
        .filter_map(|r| r.ok())
        .collect();

    let mut children: Vec<Entry> = dir_entries
        .into_par_iter()
        .filter_map(|de| scan_dirent(de, depth + 1, max_depth).ok())
        .collect();

    children.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    let size: u64 = children.iter().map(|e| e.size).sum();

    Ok(Entry { name, size, kind: EntryKind::Dir { children } })
}

// Process a DirEntry. Uses DirEntry::file_type() which reads d_type from getdents
// on macOS/Linux — no extra lstat syscall per entry.
fn scan_dirent(de: fs::DirEntry, depth: usize, max_depth: Option<usize>) -> Result<Entry, ScanError> {
    let file_type = de.file_type()?;
    let name = de.file_name().to_string_lossy().into_owned();

    if file_type.is_symlink() {
        return Ok(Entry { name, size: 0, kind: EntryKind::Symlink });
    }

    if file_type.is_file() {
        // DirEntry::metadata() uses fstatat relative to the dir fd — faster than
        // a full-path lstat on every entry.
        let size = de.metadata().map(|m| m.len()).unwrap_or(0);
        return Ok(Entry { name, size, kind: EntryKind::File });
    }

    scan_dir(&de.path(), name, depth, max_depth)
}

// Parallel recursive size accumulation used when at the depth limit.
// Rayon work-steals across the recursive fan-out automatically.
fn compute_total_size(path: &Path) -> u64 {
    let entries: Vec<fs::DirEntry> = match fs::read_dir(path) {
        Ok(rd) => rd.filter_map(|r| r.ok()).collect(),
        Err(_) => return 0,
    };

    entries
        .into_par_iter()
        .map(|de| {
            let ft = match de.file_type() {
                Ok(ft) => ft,
                Err(_) => return 0,
            };
            if ft.is_file() {
                de.metadata().map(|m| m.len()).unwrap_or(0)
            } else if ft.is_dir() {
                compute_total_size(&de.path())
            } else {
                0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // create temp dir such that it is used to test
    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("rdu_test_{}", name));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }
        fn path(&self) -> &Path {
            &self.path
        }
    }

    // drop the temp dir after the test finishes
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn stats_from_file_entry() {
        let entry = Entry { name: "file.txt".into(), size: 100, kind: EntryKind::File };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.dir_count, 0);
        assert_eq!(stats.total_size, 100);
    }

    #[test]
    fn stats_from_symlink_entry() {
        let entry = Entry { name: "link".into(), size: 0, kind: EntryKind::Symlink };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.dir_count, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn stats_from_empty_dir() {
        let entry = Entry {
            name: "root".into(),
            size: 0,
            kind: EntryKind::Dir { children: vec![] },
        };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.dir_count, 1);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn stats_from_dir_with_files() {
        let entry = Entry {
            name: "root".into(),
            size: 300,
            kind: EntryKind::Dir {
                children: vec![
                    Entry { name: "a.txt".into(), size: 100, kind: EntryKind::File },
                    Entry { name: "b.txt".into(), size: 200, kind: EntryKind::File },
                ],
            },
        };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.dir_count, 1);
        assert_eq!(stats.total_size, 300);
    }

    #[test]
    fn stats_nested_dirs_counted() {
        let entry = Entry {
            name: "root".into(),
            size: 150,
            kind: EntryKind::Dir {
                children: vec![Entry {
                    name: "sub".into(),
                    size: 150,
                    kind: EntryKind::Dir {
                        children: vec![
                            Entry { name: "x.txt".into(), size: 50,  kind: EntryKind::File },
                            Entry { name: "y.txt".into(), size: 100, kind: EntryKind::File },
                        ],
                    },
                }],
            },
        };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.dir_count, 2); // root + sub
        assert_eq!(stats.total_size, 150);
    }

    #[test]
    fn stats_symlinks_not_counted_in_files() {
        let entry = Entry {
            name: "root".into(),
            size: 10,
            kind: EntryKind::Dir {
                children: vec![
                    Entry { name: "file.txt".into(), size: 10, kind: EntryKind::File },
                    Entry { name: "link".into(),     size: 0,  kind: EntryKind::Symlink },
                ],
            },
        };
        let stats = Stats::from_entry(&entry);
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.dir_count, 1);
    }

    #[test]
    fn scan_nonexistent_path_returns_error() {
        let result = scan(Path::new("/no/such/path/rdu_test_nonexistent"), None);
        assert!(result.is_err());
    }

    #[test]
    fn scan_single_file() {
        let tmp = TempDir::new("single_file");
        let file_path = tmp.path().join("hello.txt");
        fs::write(&file_path, b"hello world").unwrap(); // 11 bytes

        let entry = scan(&file_path, None).unwrap();
        assert_eq!(entry.name, "hello.txt");
        assert_eq!(entry.size, 11);
        assert!(matches!(entry.kind, EntryKind::File));
    }

    #[test]
    fn scan_empty_directory() {
        let tmp = TempDir::new("empty_dir");
        let entry = scan(tmp.path(), None).unwrap();
        assert_eq!(entry.size, 0);
        assert!(matches!(&entry.kind, EntryKind::Dir { children } if children.is_empty()));
    }

    #[test]
    fn scan_directory_totals_file_sizes() {
        let tmp = TempDir::new("dir_totals");
        fs::write(tmp.path().join("a.txt"), b"aaaa").unwrap();   // 4 bytes
        fs::write(tmp.path().join("b.txt"), b"bb").unwrap();     // 2 bytes
        fs::write(tmp.path().join("c.txt"), b"cccccc").unwrap(); // 6 bytes

        let entry = scan(tmp.path(), None).unwrap();
        assert_eq!(entry.size, 12);
    }

    #[test]
    fn scan_children_sorted_by_size_descending() {
        let tmp = TempDir::new("sorted");
        fs::write(tmp.path().join("small.txt"),  b"x").unwrap();          // 1 byte
        fs::write(tmp.path().join("large.txt"),  b"xxxxxxxxxx").unwrap(); // 10 bytes
        fs::write(tmp.path().join("medium.txt"), b"xxxxx").unwrap();      // 5 bytes

        let entry = scan(tmp.path(), None).unwrap();
        if let EntryKind::Dir { children } = &entry.kind {
            let sizes: Vec<u64> = children.iter().map(|e| e.size).collect();
            assert_eq!(sizes, vec![10, 5, 1]);
        } else {
            panic!("expected Dir");
        }
    }

    #[test]
    fn scan_depth_limit_collapses_grandchildren() {
        let tmp = TempDir::new("depth_limit");
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("deep.txt"), b"deep content").unwrap(); // 12 bytes

        // depth=1 — subdir shows up but its children are hidden
        let entry = scan(tmp.path(), Some(1)).unwrap();
        if let EntryKind::Dir { children } = &entry.kind {
            assert_eq!(children.len(), 1);
            let child = &children[0];
            assert_eq!(child.size, 12);
            if let EntryKind::Dir { children: gc } = &child.kind {
                assert!(gc.is_empty(), "grandchildren must be hidden at depth limit");
            } else {
                panic!("expected child to be a Dir");
            }
        } else {
            panic!("expected root to be a Dir");
        }
    }

    #[test]
    fn scan_unlimited_depth_exposes_all_levels() {
        let tmp = TempDir::new("unlimited");
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("deep.txt"), b"deep content").unwrap();

        let entry = scan(tmp.path(), None).unwrap();
        if let EntryKind::Dir { children } = &entry.kind {
            assert_eq!(children.len(), 1);
            if let EntryKind::Dir { children: gc } = &children[0].kind {
                assert_eq!(gc.len(), 1);
                assert!(matches!(gc[0].kind, EntryKind::File));
            } else {
                panic!("expected subdir to be a Dir");
            }
        } else {
            panic!("expected root to be a Dir");
        }
    }

    #[test]
    fn scan_depth_zero_collapses_root_children() {
        let tmp = TempDir::new("depth_zero");
        fs::write(tmp.path().join("file.txt"), b"data").unwrap();

        let entry = scan(tmp.path(), Some(0)).unwrap();
        // depth=0 means the root itself is at the limit
        if let EntryKind::Dir { children } = &entry.kind {
            assert!(children.is_empty());
            assert_eq!(entry.size, 4);
        } else {
            panic!("expected Dir");
        }
    }
}
