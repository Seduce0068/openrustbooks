//! Diff computation and display for Markdown changes.

use crate::CoreResult;
use similar::{Algorithm, ChangeTag, TextDiff};
use std::fmt;

/// Diff result containing changes between two texts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
    pub original_lines: usize,
    pub modified_lines: usize,
}

/// A single diff hunk (contiguous changes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
    pub content: String,
}

/// Diff line tag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffTag {
    Equal,
    Insert,
    Delete,
    Replace,
}

/// Diff statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub unchanged: usize,
}

/// Compute diff between two texts
pub fn compute(old: &str, new: &str) -> DiffResult {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .timeout(std::time::Duration::from_millis(100))
        .diff_lines(old, new);

    let mut hunks = Vec::new();
    let mut stats = DiffStats::default();
    let mut old_line_num = 0;
    let mut new_line_num = 0;

    for group in diff.grouped_ops(3).iter() {
        let mut hunk_lines = Vec::new();
        let mut hunk_old_start = 0;
        let mut hunk_new_start = 0;
        let mut hunk_old_lines = 0;
        let mut hunk_new_lines = 0;
        let mut first = true;

        for op in group {
            match op {
                ChangeTag::Equal => {
                    for line in op.values() {
                        if first {
                            hunk_old_start = old_line_num + 1;
                            hunk_new_start = new_line_num + 1;
                            first = false;
                        }
                        old_line_num += 1;
                        new_line_num += 1;
                        hunk_old_lines += 1;
                        hunk_new_lines += 1;
                        stats.unchanged += 1;
                        hunk_lines.push(DiffLine {
                            tag: DiffTag::Equal,
                            old_line_number: Some(old_line_num),
                            new_line_number: Some(new_line_num),
                            content: line.to_string(),
                        });
                    }
                }
                ChangeTag::Insert => {
                    for line in op.values() {
                        if first {
                            hunk_old_start = old_line_num + 1;
                            hunk_new_start = new_line_num + 1;
                            first = false;
                        }
                        new_line_num += 1;
                        hunk_new_lines += 1;
                        stats.additions += 1;
                        hunk_lines.push(DiffLine {
                            tag: DiffTag::Insert,
                            old_line_number: None,
                            new_line_number: Some(new_line_num),
                            content: line.to_string(),
                        });
                    }
                }
                ChangeTag::Delete => {
                    for line in op.values() {
                        if first {
                            hunk_old_start = old_line_num + 1;
                            hunk_new_start = new_line_num + 1;
                            first = false;
                        }
                        old_line_num += 1;
                        hunk_old_lines += 1;
                        stats.deletions += 1;
                        hunk_lines.push(DiffLine {
                            tag: DiffTag::Delete,
                            old_line_number: Some(old_line_num),
                            new_line_number: None,
                            content: line.to_string(),
                        });
                    }
                }
                ChangeTag::Replace => {
                    // Handle as delete + insert pair
                    let old_lines: Vec<_> = op.old_values().collect();
                    let new_lines: Vec<_> = op.new_values().collect();

                    for line in &old_lines {
                        if first {
                            hunk_old_start = old_line_num + 1;
                            hunk_new_start = new_line_num + 1;
                            first = false;
                        }
                        old_line_num += 1;
                        hunk_old_lines += 1;
                        stats.deletions += 1;
                        hunk_lines.push(DiffLine {
                            tag: DiffTag::Delete,
                            old_line_number: Some(old_line_num),
                            new_line_number: None,
                            content: line.to_string(),
                        });
                    }

                    for line in &new_lines {
                        new_line_num += 1;
                        hunk_new_lines += 1;
                        stats.additions += 1;
                        hunk_lines.push(DiffLine {
                            tag: DiffTag::Insert,
                            old_line_number: None,
                            new_line_number: Some(new_line_num),
                            content: line.to_string(),
                        });
                    }
                    stats.modifications += 1;
                }
            }
        }

        if !hunk_lines.is_empty() {
            hunks.push(DiffHunk {
                old_start: hunk_old_start,
                old_lines: hunk_old_lines,
                new_start: hunk_new_start,
                new_lines: hunk_new_lines,
                lines: hunk_lines,
            });
        }
    }

    DiffResult {
        hunks,
        stats,
        original_lines: old.lines().count(),
        modified_lines: new.lines().count(),
    }
}

/// Render diff as unified diff format (for terminal/debugging)
pub fn render_unified(diff: &DiffResult, _context_lines: usize) -> String {
    let mut output = String::new();

    for hunk in &diff.hunks {
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ));

        for line in &hunk.lines {
            let prefix = match line.tag {
                DiffTag::Equal => " ",
                DiffTag::Insert => "+",
                DiffTag::Delete => "-",
                DiffTag::Replace => "!",
            };
            output.push_str(&format!("{}{}\n", prefix, line.content));
        }
    }

    output
}

/// Render diff as HTML (for web UI)
pub fn render_html(diff: &DiffResult) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"diff\">\n");

    for hunk in &diff.hunks {
        html.push_str("  <div class=\"diff-hunk\">\n");
        html.push_str(&format!(
            "    <div class=\"diff-header\">@@ -{},{} +{},{} @@</div>\n",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ));

        for line in &hunk.lines {
            let (class, prefix) = match line.tag {
                DiffTag::Equal => ("diff-equal", " "),
                DiffTag::Insert => ("diff-insert", "+"),
                DiffTag::Delete => ("diff-delete", "-"),
                DiffTag::Replace => ("diff-replace", "!"),
            };

            let old_num = line
                .old_line_number
                .map(|n| n.to_string())
                .unwrap_or_else(|| "".to_string());
            let new_num = line
                .new_line_number
                .map(|n| n.to_string())
                .unwrap_or_else(|| "".to_string());

            html.push_str(&format!(
                "    <div class=\"diff-line {}\" data-old=\"{}\" data-new=\"{}\">\n",
                class, old_num, new_num
            ));
            html.push_str(&format!(
                "      <span class=\"diff-prefix\">{}</span>\n",
                prefix
            ));
            html.push_str(&format!(
                "      <span class=\"diff-content\">{}</span>\n",
                html_escape(&line.content)
            ));
            html.push_str("    </div>\n");
        }
        html.push_str("  </div>\n");
    }

    html.push_str("</div>\n");
    html
}

/// Render diff as side-by-side HTML (for split view)
pub fn render_side_by_side(diff: &DiffResult) -> (String, String) {
    let mut left = String::new();
    let mut right = String::new();

    left.push_str("<div class=\"diff-side diff-old\">\n");
    right.push_str("<div class=\"diff-side diff-new\">\n");

    let mut old_line = 0;
    let mut new_line = 0;

    for hunk in &diff.hunks {
        for line in &hunk.lines {
            match line.tag {
                DiffTag::Equal => {
                    old_line += 1;
                    new_line += 1;
                    left.push_str(&format!(
                        "<div class=\"diff-line equal\" data-line=\"{}\"><span class=\"lineno\">{}</span><span class=\"content\">{}</span></div>\n",
                        old_line, old_line, html_escape(&line.content)
                    ));
                    right.push_str(&format!(
                        "<div class=\"diff-line equal\" data-line=\"{}\"><span class=\"lineno\">{}</span><span class=\"content\">{}</span></div>\n",
                        new_line, new_line, html_escape(&line.content)
                    ));
                }
                DiffTag::Insert => {
                    new_line += 1;
                    left.push_str(&format!(
                        "<div class=\"diff-line empty\" data-line=\"\"><span class=\"lineno\"></span><span class=\"content\"></span></div>\n"
                    ));
                    right.push_str(&format!(
                        "<div class=\"diff-line insert\" data-line=\"{}\"><span class=\"lineno\">{}</span><span class=\"content\">{}</span></div>\n",
                        new_line, new_line, html_escape(&line.content)
                    ));
                }
                DiffTag::Delete => {
                    old_line += 1;
                    left.push_str(&format!(
                        "<div class=\"diff-line delete\" data-line=\"{}\"><span class=\"lineno\">{}</span><span class=\"content\">{}</span></div>\n",
                        old_line, old_line, html_escape(&line.content)
                    ));
                    right.push_str(&format!(
                        "<div class=\"diff-line empty\" data-line=\"\"><span class=\"lineno\"></span><span class=\"content\"></span></div>\n"
                    ));
                }
                DiffTag::Replace => {
                    // Show as delete on left, insert on right
                    old_line += 1;
                    new_line += 1;
                    left.push_str(&format!(
                        "<div class=\"diff-line delete\" data-line=\"{}\"><span class=\"lineno\">{}</span><span class=\"content\">{}</span></div>\n",
                        old_line, old_line, html_escape(&line.content)
                    ));
                    // Note: Replace is modeled as delete+insert, so next line will be insert
                }
            }
        }
    }

    left.push_str("</div>\n");
    right.push_str("</div>\n");

    (left, right)
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&")
        .replace('<', "<")
        .replace('>', ">")
        .replace('"', "\"")
        .replace('\'', "'")
}

/// Inline diff for small changes (word-level)
pub fn inline_diff(old: &str, new: &str) -> Vec<InlineDiffSegment> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_chars(old, new);

    let mut segments = Vec::new();

    for op in diff.iter() {
        match op {
            ChangeTag::Equal => {
                for ch in op.values() {
                    segments.push(InlineDiffSegment {
                        tag: DiffTag::Equal,
                        content: ch.to_string(),
                    });
                }
            }
            ChangeTag::Insert => {
                for ch in op.values() {
                    segments.push(InlineDiffSegment {
                        tag: DiffTag::Insert,
                        content: ch.to_string(),
                    });
                }
            }
            ChangeTag::Delete => {
                for ch in op.values() {
                    segments.push(InlineDiffSegment {
                        tag: DiffTag::Delete,
                        content: ch.to_string(),
                    });
                }
            }
            ChangeTag::Replace => {
                for ch in op.old_values() {
                    segments.push(InlineDiffSegment {
                        tag: DiffTag::Delete,
                        content: ch.to_string(),
                    });
                }
                for ch in op.new_values() {
                    segments.push(InlineDiffSegment {
                        tag: DiffTag::Insert,
                        content: ch.to_string(),
                    });
                }
            }
        }
    }

    // Merge adjacent segments with same tag
    let mut merged = Vec::new();
    for seg in segments {
        if let Some(last) = merged.last_mut() {
            if last.tag == seg.tag {
                last.content.push_str(&seg.content);
                continue;
            }
        }
        merged.push(seg);
    }

    merged
}

/// Inline diff segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineDiffSegment {
    pub tag: DiffTag,
    pub content: String,
}
