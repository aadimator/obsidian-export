//! A collection of officially maintained [postprocessors][crate::Postprocessor].

use crate::references::ObsidianNoteReference;

use super::{Context, MarkdownEvents, PostprocessorResult};
use pulldown_cmark::{Event, CowStr};
use serde_yaml::Value;

/// This postprocessor converts all soft line breaks to hard line breaks. Enabling this mimics
/// Obsidian's _'Strict line breaks'_ setting.
pub fn softbreaks_to_hardbreaks(
    _context: &mut Context,
    events: &mut MarkdownEvents<'_>,
) -> PostprocessorResult {
    for event in events.iter_mut() {
        if event == &Event::SoftBreak {
            *event = Event::HardBreak;
        }
    }
    PostprocessorResult::Continue
}


/// This postprocessor adds `div` tags with classes containing the info of the embedded
/// document. This can then be used later on.
pub fn add_embed_info(
    context: &mut Context,
    events: &mut MarkdownEvents,
) -> PostprocessorResult {

    let key = Value::String("embed_link".to_string());
    let link_text = context.frontmatter.get(&key).unwrap().as_str().unwrap().to_owned();

    let note_ref = ObsidianNoteReference::from_str(&link_text);

    let id_value = context.frontmatter.get(&Value::String("id".to_string())).unwrap().as_str().unwrap().to_owned();

    let mut link = id_value;
    let mut title = note_ref.display().to_owned();

    if let Some(section) = note_ref.section {
        link.push('#');
        link.push_str(&slug::slugify(section));
    }

    if let Some(label) = note_ref.label {
        title = label.to_string();
    }

    events.insert(0, Event::Text(CowStr::from(format!("\n<div class=\"markdown-embed\">\n<div class=\"markdown-embed-title\" style=\"display:none;\">{}</div>\n<div class=\"markdown-embed-content\">\n\n\n", title))));
    events.push(
        Event::Text(
            CowStr::from(
                format!(
                    "\n</div>\n<div class=\"markdown-embed-link\" style=\"display:none;\">\n\n<a href=\"\\{}\" title=\"Open Link\">\n<svg viewBox=\"0 0 100 100\" class=\"link\" width=\"20\" height=\"20\"><path fill=\"currentColor\" stroke=\"currentColor\" d=\"M74,8c-4.8,0-9.3,1.9-12.7,5.3l-10,10c-2.9,2.9-4.7,6.6-5.1,10.6C46,34.6,46,35.3,46,36c0,2.7,0.6,5.4,1.8,7.8l3.1-3.1 C50.3,39.2,50,37.6,50,36c0-3.7,1.5-7.3,4.1-9.9l10-10c2.6-2.6,6.2-4.1,9.9-4.1s7.3,1.5,9.9,4.1c2.6,2.6,4.1,6.2,4.1,9.9 s-1.5,7.3-4.1,9.9l-10,10C71.3,48.5,67.7,50,64,50c-1.6,0-3.2-0.3-4.7-0.8l-3.1,3.1c2.4,1.1,5,1.8,7.8,1.8c4.8,0,9.3-1.9,12.7-5.3 l10-10C90.1,35.3,92,30.8,92,26s-1.9-9.3-5.3-12.7C83.3,9.9,78.8,8,74,8L74,8z M62,36c-0.5,0-1,0.2-1.4,0.6l-24,24 c-0.5,0.5-0.7,1.2-0.6,1.9c0.2,0.7,0.7,1.2,1.4,1.4c0.7,0.2,1.4,0,1.9-0.6l24-24c0.6-0.6,0.8-1.5,0.4-2.2C63.5,36.4,62.8,36,62,36 z M36,46c-4.8,0-9.3,1.9-12.7,5.3l-10,10c-3.1,3.1-5,7.2-5.2,11.6c0,0.4,0,0.8,0,1.2c0,4.8,1.9,9.3,5.3,12.7 C16.7,90.1,21.2,92,26,92s9.3-1.9,12.7-5.3l10-10C52.1,73.3,54,68.8,54,64c0-2.7-0.6-5.4-1.8-7.8l-3.1,3.1 c0.5,1.5,0.8,3.1,0.8,4.7c0,3.7-1.5,7.3-4.1,9.9l-10,10C33.3,86.5,29.7,88,26,88s-7.3-1.5-9.9-4.1S12,77.7,12,74 c0-3.7,1.5-7.3,4.1-9.9l10-10c2.6-2.6,6.2-4.1,9.9-4.1c1.6,0,3.2,0.3,4.7,0.8l3.1-3.1C41.4,46.6,38.7,46,36,46L36,46z\"></path></svg> \n\n  </a></div>\n</div>\n", 
                    link
                )
            )
        )
    );
    PostprocessorResult::Continue
}


pub fn flat_hierarchy (
    context: &mut Context,
    _: &mut MarkdownEvents,
) -> PostprocessorResult {
    let dest_key = Value::String("destination".to_string());
    let destination_root = context.frontmatter.get(&dest_key).unwrap().as_str().unwrap();

    let full_path = context.destination.clone();
    let path_without_root = full_path.strip_prefix(destination_root).ok().unwrap();
    let ext = path_without_root.extension().unwrap();

    let sanitized_path = path_without_root.with_extension("").to_string_lossy().replace(".", "-").replace("/", "-");
    let full_dest_path = std::path::PathBuf::from(&destination_root).join(sanitized_path).with_extension(ext);

    // println!("Full path: {}", full_dest_path.display());
    context.destination = full_dest_path;
    context.frontmatter.remove(&dest_key);

    PostprocessorResult::Continue

}
pub fn filter_by_tags(
    skip_tags: Vec<String>,
    only_tags: Vec<String>,
) -> impl Fn(&mut Context, &mut MarkdownEvents<'_>) -> PostprocessorResult {
    move |context: &mut Context, _events: &mut MarkdownEvents<'_>| -> PostprocessorResult {
        match context.frontmatter.get("tags") {
            None => filter_by_tags_(&[], &skip_tags, &only_tags),
            Some(Value::Sequence(tags)) => filter_by_tags_(tags, &skip_tags, &only_tags),
            _ => PostprocessorResult::Continue,
        }
    }
}

fn filter_by_tags_(
    tags: &[Value],
    skip_tags: &[String],
    only_tags: &[String],
) -> PostprocessorResult {
    let skip = skip_tags
        .iter()
        .any(|tag| tags.contains(&Value::String(tag.to_string())));
    let include = only_tags.is_empty()
        || only_tags
            .iter()
            .any(|tag| tags.contains(&Value::String(tag.to_string())));

    if skip || !include {
        PostprocessorResult::StopAndSkipNote
    } else {
        PostprocessorResult::Continue
    }
}

#[test]
fn test_filter_tags() {
    let tags = vec![
        Value::String("skip".into()),
        Value::String("publish".into()),
    ];
    let empty_tags = vec![];
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &[]),
        PostprocessorResult::Continue,
        "When no exclusion & inclusion are specified, files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["exclude".into()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &["exclude".into()], &[]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files without tags are included"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["publish".into()]),
        PostprocessorResult::Continue,
        "When exclusion tags don't match files with tags are included"
    );
    assert_eq!(
        filter_by_tags_(&empty_tags, &[], &["include".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When inclusion tags are specified files without tags are excluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &[], &["include".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When exclusion tags don't match files with tags are exluded"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".into()], &["skip".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags are the same exclusion wins"
    );
    assert_eq!(
        filter_by_tags_(&tags, &["skip".into()], &["publish".into()]),
        PostprocessorResult::StopAndSkipNote,
        "When both inclusion and exclusion tags match exclusion wins"
    );
}
