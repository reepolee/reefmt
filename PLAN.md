
How would this algorithm work: we find all REE tags, openers and closers, :else and convert them to fake HTML tags in a mpa: <ree-1></ree-1>   <ree-2></ree-2> and send this to dprint for formatting and once it gets back, replace from map with original tags?


----
The user is proposing a clever alternative algorithm for formatting Ree templates via dprint HTML. Instead of using placeholders like __REE_IF__, they want to convert Ree tags to fake HTML elements that dprint can understand structurally.

The idea is:

Find all Ree tags ({#if}, {/if}, {#each}, {/each}, {#with}, {/with}, {:else}, etc.) and also inline expressions ({=value}, {~value}, {@Component}, {{js}}, etc.)
Convert them to fake paired HTML tags like <ree-1></ree-1> where the number maps back to the original tag
Send to dprint HTML formatter
dprint sees valid HTML with known open/close tag pairs and formats accordingly
Replace the fake HTML tags back with original Ree tags
The clever insight here is that by using proper paired HTML open/close tags, dprint's HTML formatter can understand the STRUCTURE of the template - it knows <ree-1> opens a block and </ree-1> closes it, so it will properly indent the content between them.

vs. the current approach where {#if} gets turned into __REE_IF__ which dprint treats as arbitrary text.

Let me think through this more carefully:

Current approach problems:

__REE_IF__ condition} is just text to dprint's HTML formatter
dprint doesn't know that __REE_END_IF__ closes the __REE_IF__ block
Indentation structure is lost/wrong
Proposed approach advantages:

<ree-1> (for {#if condition}) and </ree-1> (for {/if}) form a proper HTML pair
dprint KNOWS <ree-1> opens a block and </ree-1> closes it
Content between them gets indented correctly by dprint's HTML formatter
:else can be mapped to a self-closing <ree-2 /> or... hmm, :else is trickier
Wait, let me think about :else. In HTML, you can't really represent an "else" tag between an if and an end-if without it being a sibling to the content.

For example:

{#if condition}
  content
{:else}
  other content
{/if}
Would become:

html
<ree-1>
  content
<ree-2 />
  other content
…
Show less


-----


Clean concept. Here's how it works and the key subtleties:

**Core idea:** `{#if cond}` → `<ree-1>`, `{/if}` → `</ree-1>`. dprint now understands the nesting structure and indents content between them correctly — something string placeholders like `__REE_IF__` can't express.

**The `:else` trick:** You can't express "else" as a single HTML tag. Solution — split it into a synthetic close + new open: `{:else}` emits `</ree-1><ree-2>`. The map stores that `</ree-1>` restores to `""` (deleted) and `<ree-2>` restores to `{:else}`. The matching `{/if}` then closes `</ree-2>` → `{/if}`.

**Inline expressions** (`{=expr}`, `{~}`, `{{}}`, `{@}`, includes) are left alone — the existing `protect()`/`restore()` handles them. Only block-control tags get the fake HTML treatment.


---


// ── Drop these in main.rs, replacing format_ree_html_via_dprint ─────────────

/// One entry in the fake-element map. Each `<ree-N>` / `</ree-N>` pair in the
/// fake HTML corresponds to one FakeEntry at index N-1.
struct FakeEntry {
    /// What `<ree-N>` (or `<ree-N />`) restores to.
    open_text: String,
    /// What `</ree-N>` restores to. Empty string = synthetic close emitted for
    /// an {:else} boundary; the tag is simply deleted on restore.
    close_text: String,
}

/// Find the position just past the balanced closing `}` starting from the
/// opening `{`. Handles nested braces (e.g. `{#if obj.items.find(x => x)}`).
fn find_ree_block_end(src: &str) -> usize {
    let mut depth = 0usize;
    for (i, c) in src.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
    }
    src.len()
}

/// Convert Ree block-control tags to paired fake HTML elements so dprint's
/// HTML formatter understands the nesting structure.
///
/// Only handles structural tags: {#if}, {#each}, {#with}, their closers, and
/// {:else}/{:else if}. All other Ree syntax is passed through unchanged and
/// will be handled by the existing protect()/restore() calls that wrap this.
///
/// {:else} emits `</ree-N><ree-M>`:
///   - `</ree-N>` maps to "" (synthetic, deleted on restore)
///   - `<ree-M>` maps to the {:else...} tag
///   - `</ree-M>` is set when the matching {/if} is found → maps to "{/if}"
fn ree_to_fake_html(src: &str) -> (String, Vec<FakeEntry>) {
    let mut out = String::new();
    let mut entries: Vec<FakeEntry> = Vec::new();
    let mut stack: Vec<usize> = Vec::new(); // indices into entries for open blocks
    let mut rest = src;

    while !rest.is_empty() {
        let Some(pos) = rest.find('{') else {
            out.push_str(rest);
            break;
        };

        let after = &rest[pos..];

        // Categorise the tag at this position.
        let is_if_open   = after.starts_with("{#if}")
                        || matches!(after.as_bytes().get(4), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#if");
        let is_each_open = after.starts_with("{#each}")
                        || matches!(after.as_bytes().get(6), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#each");
        let is_with_open = after.starts_with("{#with}")
                        || matches!(after.as_bytes().get(6), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#with");

        let is_if_close   = after.starts_with("{/if}");
        let is_each_close = after.starts_with("{/each}");
        let is_with_close = after.starts_with("{/with}");

        // {:else} and {:else if ...} — but NOT {:param} or other colon-tags
        let is_else = after.starts_with("{:else}") || after.starts_with("{:else ");

        if is_if_open || is_each_open || is_with_open {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();
            let idx = entries.len();
            entries.push(FakeEntry { open_text: tag, close_text: String::new() });
            stack.push(idx);
            out.push_str(&format!("<ree-{}>", idx + 1));
            rest = &after[end..];

        } else if is_if_close || is_each_close || is_with_close {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();
            match stack.pop() {
                Some(idx) => {
                    entries[idx].close_text = tag;
                    out.push_str(&format!("</ree-{}>", idx + 1));
                }
                None => out.push_str(&tag), // unmatched closer — pass through
            }
            rest = &after[end..];

        } else if is_else {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();

            // Synthetically close the current open block (close_text stays "")
            if let Some(prev_idx) = stack.pop() {
                out.push_str(&format!("</ree-{}>", prev_idx + 1));
            }

            // Open a new block entry for the else branch
            let idx = entries.len();
            entries.push(FakeEntry { open_text: tag, close_text: String::new() });
            stack.push(idx);
            out.push_str(&format!("<ree-{}>", idx + 1));
            rest = &after[end..];

        } else {
            // Not a block-control tag — pass through the `{` and advance
            out.push_str(&rest[..pos + 1]);
            rest = &rest[pos + 1..];
        }
    }

    (out, entries)
}

/// Restore fake HTML elements back to original Ree tags.
/// Empty close_text (synthetic {:else} closes) simply vanishes, leaving at
/// most a blank line which collapse_blank_lines() cleans up.
fn fake_html_to_ree(src: &str, entries: &[FakeEntry]) -> String {
    let mut result = src.to_string();
    // Replace from highest N → lowest to avoid ree-1 matching inside ree-10, ree-11, etc.
    // (Technically the `>` terminator prevents this, but being explicit is safer.)
    for (i, entry) in entries.iter().enumerate().rev() {
        let n = i + 1;
        result = result.replace(&format!("<ree-{}>", n), &entry.open_text);
        result = result.replace(&format!("</ree-{}>", n), &entry.close_text);
    }
    collapse_blank_lines(&result)
}

// ── Replace format_ree_html_via_dprint with this version ─────────────────────

/// Format the HTML skeleton of a Ree template via dprint's markup_fmt plugin.
///
/// Pipeline:
///   protect_html_comments
///   → ree_to_fake_html        (block tags → <ree-N> pairs, {:else} → split)
///   → protect                 (remaining inline Ree syntax → __REE_*__ strings)
///   → pipe_dprint html        (markup_fmt sees valid HTML with real structure)
///   → restore                 (inline syntax back)
///   → fake_html_to_ree        (block tags back)
///   → restore_html_comments
///
/// Returns None if dprint produced no change (plugin not installed).
fn format_ree_html_via_dprint(src: &str, config_path: &str) -> Option<String> {
    let (after_comments, html_comments) = protect_html_comments(src.trim());
    let (after_blocks, entries) = ree_to_fake_html(&after_comments);
    let protected = protect(&after_blocks);

    let formatted = pipe_dprint(&protected, "html", config_path);

    if formatted.trim() == protected.trim() {
        return None;
    }

    let restored = restore(&formatted);
    let restored = fake_html_to_ree(&restored, &entries);
    Some(restore_html_comments(&restored, &html_comments))
}

// format_ree_content stays the same as the previous version —
// it calls format_ree_html_via_dprint and falls back to format_html.
