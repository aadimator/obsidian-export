use super::{Context, MarkdownEvents, PostprocessorResult};
use serde_yaml::Value;


/// This postprocessor author:Aadam to frontmatter
pub fn add_author(
    _context: &mut Context,
    _events: &mut MarkdownEvents,
) -> PostprocessorResult {
    let key = Value::String("author".to_string());
    let value = Value::String("Aadam".to_string());
    
    // Frontmatter can be updated in-place, so we can call insert on it directly.
    _context.frontmatter.insert(key, value);

    PostprocessorResult::Continue
}

/// This postprocessor removes empty aliases from the frontmatter
pub fn remove_empty_aliases(
    _context: &mut Context,
    _events: &mut MarkdownEvents,
) -> PostprocessorResult {
    let key = Value::String("aliases".to_string());
    if _context.frontmatter.contains_key(&key) {
        let value = _context.frontmatter.get(&key).unwrap();
        let seq = value.as_sequence();
        if seq.is_none() || seq.unwrap().is_empty() {
            _context.frontmatter.remove(&key);
            println!("Removed empty alias from {}", _context.current_file().display());
        }
    }
    PostprocessorResult::Continue
}
