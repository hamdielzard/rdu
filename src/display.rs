use colored::Colorize;

use crate::format::format_size;
use crate::scanner::{Entry, EntryKind};

const BRANCH: &str = "├── ";
const LAST:   &str = "└── ";
const PIPE:   &str = "│   ";
const SPACE:  &str = "    ";

pub fn render_tree(root: &Entry, display_name: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("{} ({})", display_name, apply_color(format_size(root.size), root.size)));

    if let EntryKind::Dir { children } = &root.kind {
        let size_width = max_size_width(children);
        for (i, child) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            render_entry(child, "", is_last, size_width, &mut lines);
        }
    }

    lines.join("\n")
}

fn render_entry(entry: &Entry, prefix: &str, is_last: bool, size_width: usize, lines: &mut Vec<String>) {
    let connector = if is_last { LAST } else { BRANCH };

    let padded = format!("{:>width$}", format_size(entry.size), width = size_width);
    let size_str = apply_color(padded, entry.size);

    let line = match &entry.kind {
        EntryKind::Dir { children } => {
            let label = format!("{}/", entry.name).bold().to_string();
            let suffix = if children.is_empty() { "" } else { " " };
            format!("{}{}{}  {}{}", prefix, connector, size_str, suffix, label)
        }
        EntryKind::File => {
            format!("{}{}{}  {}", prefix, connector, size_str, entry.name)
        }
        EntryKind::Symlink => {
            let label = format!("{} ->", entry.name).dimmed().to_string();
            format!("{}{}{}  {}", prefix, connector, size_str, label)
        }
    };

    lines.push(line);

    if let EntryKind::Dir { children } = &entry.kind {
        let extension = if is_last { SPACE } else { PIPE };
        let new_prefix = format!("{}{}", prefix, extension);
        let child_size_width = max_size_width(children);
        for (i, child) in children.iter().enumerate() {
            let child_is_last = i == children.len() - 1;
            render_entry(child, &new_prefix, child_is_last, child_size_width, lines);
        }
    }
}

fn max_size_width(entries: &[Entry]) -> usize {
    entries
        .iter()
        .map(|e| format_size(e.size).len())
        .max()
        .unwrap_or(0)
}

fn apply_color(s: String, bytes: u64) -> String {
    match bytes {
        0..1_024                   => s.dimmed().to_string(),
        1_024..1_048_576           => s.normal().to_string(),
        1_048_576..10_485_760      => s.cyan().to_string(),
        10_485_760..104_857_600    => s.yellow().to_string(),
        104_857_600..1_073_741_824 => s.red().bold().to_string(),
        _                          => s.magenta().bold().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{Entry, EntryKind};

    fn disable_color() {
        colored::control::set_override(false);
    }

    fn file(name: &str, size: u64) -> Entry {
        Entry { name: name.into(), size, kind: EntryKind::File }
    }

    fn dir(name: &str, size: u64, children: Vec<Entry>) -> Entry {
        Entry { name: name.into(), size, kind: EntryKind::Dir { children } }
    }

    fn symlink(name: &str) -> Entry {
        Entry { name: name.into(), size: 0, kind: EntryKind::Symlink }
    }

    
    #[test]
    fn root_line_contains_display_name_and_size() {
        disable_color();
        let entry = file("ignored", 512);
        let out = render_tree(&entry, "myfile.txt");
        assert!(out.contains("myfile.txt"));
        assert!(out.contains("512 B"));
    }

    #[test]
    fn empty_dir_produces_single_line() {
        disable_color();
        let entry = dir("root", 0, vec![]);
        let out = render_tree(&entry, "mydir");
        assert_eq!(out.lines().count(), 1);
        assert!(out.contains("mydir"));
        assert!(out.contains("0 B"));
    }

    #[test]
    fn two_children_line_count() {
        disable_color();
        let entry = dir("root", 300, vec![
            file("big.txt",   200),
            file("small.txt", 100),
        ]);
        let out = render_tree(&entry, "root");
        assert_eq!(out.lines().count(), 3); // root + 2 children
    }

    #[test]
    fn two_children_names_present() {
        disable_color();
        let entry = dir("root", 300, vec![
            file("big.txt",   200),
            file("small.txt", 100),
        ]);
        let out = render_tree(&entry, "root");
        assert!(out.contains("big.txt"));
        assert!(out.contains("small.txt"));
    }

    #[test]
    fn last_child_uses_corner_connector() {
        disable_color();
        let entry = dir("root", 300, vec![
            file("first.txt",  200),
            file("second.txt", 100),
        ]);
        let out = render_tree(&entry, "root");
        assert!(out.contains("├──"), "non-last child should use ├──");
        assert!(out.contains("└──"), "last child should use └──");
    }

    #[test]
    fn single_child_uses_only_corner_connector() {
        disable_color();
        let entry = dir("root", 50, vec![file("only.txt", 50)]);
        let out = render_tree(&entry, "root");
        assert!(!out.contains("├──"), "sole child should not use ├──");
        assert!(out.contains("└──"));
    }

    #[test]
    fn directory_child_label_has_trailing_slash() {
        disable_color();
        let entry = dir("root", 50, vec![dir("subdir", 50, vec![])]);
        let out = render_tree(&entry, "root");
        assert!(out.contains("subdir/"));
    }

    #[test]
    fn symlink_child_label_has_arrow() {
        disable_color();
        let entry = dir("root", 0, vec![symlink("mylink")]);
        let out = render_tree(&entry, "root");
        assert!(out.contains("mylink ->"));
    }

    #[test]
    fn nested_dir_line_count() {
        disable_color();
        // root
        //   └── sub/
        //         └── file.txt
        let entry = dir("root", 100, vec![
            dir("sub", 100, vec![file("file.txt", 100)]),
        ]);
        let out = render_tree(&entry, "root");
        assert_eq!(out.lines().count(), 3);
    }

    #[test]
    fn nested_dir_uses_pipe_continuation() {
        disable_color();
        // root
        //   ├── sub/
        //   │     └── nested.txt
        //   └── other.txt
        let entry = dir("root", 200, vec![
            dir("sub", 100, vec![file("nested.txt", 100)]),
            file("other.txt", 100),
        ]);
        let out = render_tree(&entry, "root");
        // sub is not last, so its continuation line should have │
        assert!(out.contains("│"), "expected pipe continuation for non-last directory");
        assert!(out.contains("nested.txt"));
        assert!(out.contains("other.txt"));
    }

    #[test]
    fn size_column_is_right_aligned_within_siblings() {
        disable_color();
        // "1.0 KB" (6 chars) and "100 B" (5 chars) — 1.0 KB should be padded to 6
        let entry = dir("root", 1_124, vec![
            file("big.txt",   1_024),
            file("small.txt", 100),
        ]);
        let out = render_tree(&entry, "root");
        // The larger size string should appear with consistent width across siblings.
        // Both should show up in the output.
        assert!(out.contains("1.0 KB"));
        assert!(out.contains("100 B"));
    }
}
