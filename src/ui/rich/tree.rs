//! P2 — tree view (Rich-style).
//!
//! Renders hierarchical data with guide lines (`├──`, `└──`, `│   `).
//! Display-only: `file_glob` and friends keep their flat tool-result text
//! for the model; this is the human-facing alternative for CLI output.

use crate::ui::theme;

/// One node in a display tree.
#[derive(Debug, Default, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn leaf(label: &str) -> Self {
        Self::new(label)
    }

    /// Build a tree from `/`-separated paths (e.g. `file_glob` hits).
    pub fn from_paths(paths: &[&str]) -> Self {
        let mut root = TreeNode::new(".");
        for p in paths {
            root.insert(&p.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>());
        }
        root
    }

    fn insert(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }
        let slot = self.children.iter_mut().find(|c| c.label == parts[0]);
        match slot {
            Some(node) => node.insert(&parts[1..]),
            None => {
                let mut node = TreeNode::new(parts[0]);
                node.insert(&parts[1..]);
                self.children.push(node);
            }
        }
    }

    /// Render with guides. Plain indented text when `decorate` is false.
    pub fn render(&self, decorate: bool) -> String {
        let mut out = String::new();
        out.push_str(&self.label);
        out.push('\n');
        let n = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            child.render_into(&mut out, "", i + 1 == n, decorate);
        }
        out
    }

    fn render_into(&self, out: &mut String, prefix: &str, last: bool, decorate: bool) {
        let (branch, cont) = if last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        if decorate {
            out.push_str(prefix);
            out.push_str(&theme::faint(branch).to_string());
            out.push_str(&theme::accent(&self.label).to_string());
        } else {
            out.push_str(prefix);
            out.push_str(branch);
            out.push_str(&self.label);
        }
        out.push('\n');
        let child_prefix = format!("{prefix}{cont}");
        let n = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            child.render_into(out, &child_prefix, i + 1 == n, decorate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guides_mark_last_child() {
        let tree = TreeNode::new("src")
            .child(TreeNode::leaf("main.rs"))
            .child(TreeNode::new("ui").child(TreeNode::leaf("theme.rs")));
        let s = tree.render(false);
        assert!(s.contains("├── main.rs"), "{s}");
        assert!(s.contains("└── ui"), "{s}");
        assert!(s.contains("theme.rs"), "{s}");
    }

    #[test]
    fn from_paths_nests_shared_prefixes() {
        let tree = TreeNode::from_paths(&["src/main.rs", "src/ui/theme.rs", "Cargo.toml"]);
        let s = tree.render(false);
        assert!(s.contains("src"), "{s}");
        assert!(s.contains("theme.rs"), "{s}");
        assert!(s.contains("Cargo.toml"), "{s}");
        // shared prefix appears once
        assert_eq!(s.matches("src").count(), 1, "{s}");
    }

    #[test]
    fn tty_path_adds_styling_but_keeps_labels() {
        let tree = TreeNode::new("root").child(TreeNode::leaf("a.rs"));
        let s = tree.render(true);
        assert!(s.contains("a.rs"), "{s}");
    }
}
