use crate::model::{book::Book, book_chapter::BookChapter, book_source::BookSource, search::SearchBook};
use crate::model::rule::{SearchRule, ExploreRule, BookInfoRule, TocRule};
use crate::parser::{html, jsonpath, js::eval_js};
use crate::util::text::apply_regex_replace;
use serde_json::Value;

#[derive(Clone, Default)]
pub struct RuleEngine;

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub fn search_books(&self, source: &BookSource, body: &str, base_url: &str) -> Vec<SearchBook> {
        let rule = source.rule_search.clone().unwrap_or_default();
        if is_json(body) && rule.book_list.as_ref().map(|r| r.trim_start().starts_with('$')).unwrap_or(false) {
            return self.search_books_json(source, body, base_url, &rule);
        }
        self.search_books_html(source, body, base_url, &rule)
    }

    pub fn explore_books(&self, source: &BookSource, body: &str, base_url: &str) -> Vec<SearchBook> {
        let rule = source.rule_explore.clone().unwrap_or_else(|| source.rule_search.clone().unwrap_or_default());
        if is_json(body) && rule.book_list.as_ref().map(|r| r.trim_start().starts_with('$')).unwrap_or(false) {
            return self.search_books_json(source, body, base_url, &rule);
        }
        self.search_books_html(source, body, base_url, &rule)
    }

    pub fn book_info(&self, source: &BookSource, body: &str, base_url: &str, book_url: &str) -> Book {
        let rule = source.rule_book_info.clone().unwrap_or_default();
        let mut context = std::collections::HashMap::new();
        
        if is_json(body) {
            if let Ok(v) = serde_json::from_str::<Value>(body) {
                return parse_book_info_json(source, &v, base_url, &rule, book_url, &mut context);
            }
        }
        parse_book_info_html(source, body, base_url, &rule, book_url, &mut context)
    }

    pub fn chapter_list(&self, source: &BookSource, body: &str, base_url: &str) -> (Vec<BookChapter>, Vec<String>) {
        let rule = source.rule_toc.clone().unwrap_or_default();
        let mut context = std::collections::HashMap::new();
        if is_json(body) && rule.chapter_list.as_ref().map(|r| r.trim_start().starts_with('$')).unwrap_or(false) {
            return parse_chapter_list_json(body, base_url, &rule, &mut context);
        }
        parse_chapter_list_html(body, base_url, &rule, &mut context)
    }

    pub fn content(&self, source: &BookSource, body: &str, base_url: &str) -> String {
        let rule = source.rule_content.clone().unwrap_or_default();
        println!("DEBUG: content rule: {:?}", rule.content);
        if let Some(content_rule) = rule.content.clone() {
            if content_rule.trim_start().starts_with("js:") {
                let script = content_rule.trim_start_matches("js:");
                if let Ok(res) = eval_js(script, body, base_url) {
                    return res;
                }
            }
        }
        let mut content = if is_json(body) && rule.content.as_ref().map(|r| r.trim_start().starts_with('$')).unwrap_or(false) {
            if let Ok(v) = serde_json::from_str::<Value>(body) {
                jsonpath::jsonpath_first_string(&v, rule.content.as_deref().unwrap_or("")).unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            let doc = html::parse_document(body);
            // Use select_all_text for content to get all paragraphs
            let content_rule = rule.content.as_deref().unwrap_or("");
            println!("DEBUG: calling select_all_text with rule: {}", content_rule);
            let result = html::select_all_text(&doc, content_rule);
            println!("DEBUG: select_all_text result length: {:?}", result.as_ref().map(|s| s.len()));
            result.unwrap_or_default()
        };
        if let Some(replace) = rule.replace_regex.as_deref() {
            content = apply_legado_regex(&content, replace);
        }
        println!("DEBUG: final content length: {}", content.len());
        content
    }

    /// Get the next content page URL if pagination exists
    pub fn next_content_url(&self, source: &BookSource, body: &str, base_url: &str) -> Option<String> {
        let rule = source.rule_content.clone().unwrap_or_default();
        let next_rule = rule.next_content_url.as_deref()?;
        if next_rule.is_empty() {
            println!("DEBUG: next_content_url rule is empty");
            return None;
        }

        println!("DEBUG: next_content_url rule: {}", next_rule);

        let doc = html::parse_document(body);
        let next_url = html::select_text(&doc, next_rule);

        println!("DEBUG: next_content_url resolved: {:?}", next_url);

        if next_url.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            return None;
        }

        let next_url = next_url.unwrap();

        // Resolve relative URL
        if next_url.starts_with("http://") || next_url.starts_with("https://") {
            Some(next_url)
        } else if next_url.starts_with("//") {
            Some(format!("https:{}", next_url))
        } else {
            // Resolve relative URL
            let base_parsed = url::Url::parse(base_url).ok()?;
            base_parsed.join(&next_url).ok().map(|u| u.to_string())
        }
    }

    fn search_books_html(&self, source: &BookSource, body: &str, base_url: &str, rule: &SearchRule) -> Vec<SearchBook> {
        let list_sel = match &rule.book_list {
            Some(r) => r,
            None => return vec![],
        };
        println!("DEBUG: search_books_html list_sel: {}", list_sel);
        let doc = html::parse_document(body);
        let items = html::select_list(&doc, list_sel);
        println!("DEBUG: search_books_html found {} items", items.len());
        let mut out = Vec::with_capacity(items.len());
        for el in items {
            let name = rule.name.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let author = rule.author.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let book_url = rule.book_url.as_ref().and_then(|r| eval_field_html(r, &el, base_url)).unwrap_or_default();
            let cover_url = rule.cover_url.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let intro = rule.intro.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let kind = rule.kind.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let last_chapter = rule.last_chapter.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let update_time = rule.update_time.as_ref().and_then(|r| eval_field_html(r, &el, base_url));
            let book_url_abs = resolve_url(base_url, &book_url);
            let cover_url_abs = cover_url.map(|u| resolve_url(base_url, &u));
            out.push(SearchBook {
                name,
                author,
                book_url: book_url_abs,
                origin: source.book_source_url.clone(),
                cover_url: cover_url_abs,
                intro,
                kind,
                last_chapter,
                update_time,
            });
        }
        out
    }

    fn search_books_json(&self, source: &BookSource, body: &str, base_url: &str, rule: &SearchRule) -> Vec<SearchBook> {
        let v: Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        let list_rule = rule.book_list.as_deref().unwrap_or("");
        let items = jsonpath::jsonpath_query(&v, list_rule);
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let name = eval_field_json(rule.name.as_deref().unwrap_or(""), &item, base_url);
            let author = eval_field_json(rule.author.as_deref().unwrap_or(""), &item, base_url);
            let book_url = eval_field_json(rule.book_url.as_deref().unwrap_or(""), &item, base_url);
            let cover_url = eval_field_json(rule.cover_url.as_deref().unwrap_or(""), &item, base_url);
            let intro = eval_field_json(rule.intro.as_deref().unwrap_or(""), &item, base_url);
            let kind = eval_field_json(rule.kind.as_deref().unwrap_or(""), &item, base_url);
            let last_chapter = eval_field_json(rule.last_chapter.as_deref().unwrap_or(""), &item, base_url);
            let update_time = eval_field_json(rule.update_time.as_deref().unwrap_or(""), &item, base_url);
            out.push(SearchBook {
                name: name.unwrap_or_default(),
                author: author.unwrap_or_default(),
                book_url: book_url.unwrap_or_default(),
                origin: source.book_source_url.clone(),
                cover_url,
                intro,
                kind,
                last_chapter,
                update_time,
            });
        }
        out
    }
}

fn parse_book_info_html(source: &BookSource, body: &str, base_url: &str, rule: &BookInfoRule, book_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Book {
    let doc = html::parse_document(body);
    // Execute init rule if present
    if let Some(init) = &rule.init {
        let _ = eval_field_html_doc_with_ctx(init, &doc, base_url, ctx);
    }

    let name = rule.name.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).unwrap_or_default();
    let author = rule.author.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).unwrap_or_default();
    let intro = rule.intro.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let kind = rule.kind.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let last_chapter = rule.last_chapter.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let update_time = rule.update_time.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let cover_url = rule.cover_url.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).map(|u| resolve_url(base_url, &u));
    let word_count = rule.word_count.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx));
    let toc_url = rule.toc_url.as_ref().and_then(|r| eval_field_html_doc_with_ctx(r, &doc, base_url, ctx)).map(|u| resolve_url(base_url, &u));
    
    // Fallback toc_url to book_url if missing
    let final_toc_url = toc_url.or_else(|| Some(book_url.to_string()));

    Book {
        name,
        author,
        book_url: book_url.to_string(),
        origin: source.book_source_url.clone(),
        origin_name: Some(source.book_source_name.clone()),
        cover_url,
        toc_url: final_toc_url,
        intro,
        latest_chapter_title: last_chapter,
        word_count,
        info_html: None,
        toc_html: None,
        kind,
        update_time,
        ..Default::default()
    }
}

fn parse_book_info_json(source: &BookSource, v: &Value, base_url: &str, rule: &BookInfoRule, book_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Book {
    if let Some(init) = &rule.init {
        let _ = eval_field_json_with_ctx(init, v, base_url, ctx);
    }
    let name = eval_field_json_with_ctx(rule.name.as_deref().unwrap_or(""), v, base_url, ctx).unwrap_or_default();
    let author = eval_field_json_with_ctx(rule.author.as_deref().unwrap_or(""), v, base_url, ctx).unwrap_or_default();
    let intro = eval_field_json_with_ctx(rule.intro.as_deref().unwrap_or(""), v, base_url, ctx);
    let kind = eval_field_json_with_ctx(rule.kind.as_deref().unwrap_or(""), v, base_url, ctx);
    let last_chapter = eval_field_json_with_ctx(rule.last_chapter.as_deref().unwrap_or(""), v, base_url, ctx);
    let update_time = eval_field_json_with_ctx(rule.update_time.as_deref().unwrap_or(""), v, base_url, ctx);
    let cover_url = eval_field_json_with_ctx(rule.cover_url.as_deref().unwrap_or(""), v, base_url, ctx);
    let word_count = eval_field_json_with_ctx(rule.word_count.as_deref().unwrap_or(""), v, base_url, ctx);
    let toc_url = eval_field_json_with_ctx(rule.toc_url.as_deref().unwrap_or(""), v, base_url, ctx);
    Book {
        name,
        author,
        book_url: book_url.to_string(),
        origin: source.book_source_url.clone(),
        origin_name: Some(source.book_source_name.clone()),
        cover_url,
        toc_url: toc_url.or_else(|| Some(book_url.to_string())),
        intro,
        latest_chapter_title: last_chapter,
        word_count,
        info_html: None,
        toc_html: None,
        kind,
        update_time,
        ..Default::default()
    }
}

fn parse_chapter_list_html(body: &str, base_url: &str, rule: &TocRule, ctx: &mut std::collections::HashMap<String, String>) -> (Vec<BookChapter>, Vec<String>) {
    println!("DEBUG: parse_chapter_list_html rule: {:?}", rule);
    let list_sel = match &rule.chapter_list {
        Some(r) => r,
        None => return (vec![], vec![]),
    };
    let doc = html::parse_document(body);
    // Execute init rule if present
    if let Some(init) = &rule.init {
        let _ = eval_field_html_doc_with_ctx(init, &doc, base_url, ctx);
    }
    let items = html::select_list(&doc, list_sel);

    // Use a set to deduplicate chapters by URL
    let mut seen_urls = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());

    // Detect if we have "latest chapters" section (descending order) before "content" section
    // by checking if early chapters have higher numbers than later ones
    let mut in_latest_section = false;
    let mut last_chapter_num: Option<i32> = None;

    for el in items {
        let title = rule.chapter_name.as_ref().and_then(|r| eval_field_html_with_ctx(r, &el, base_url, ctx)).unwrap_or_default();
        let url = rule.chapter_url.as_ref().and_then(|r| eval_field_html_with_ctx(r, &el, base_url, ctx)).unwrap_or_default();
        let url_abs = resolve_url(base_url, &url);

        // Skip duplicate chapters (same URL)
        if seen_urls.contains(&url_abs) {
            continue;
        }

        // Detect chapter number from title (e.g., "535.第535章" or "第1章")
        let chapter_num = extract_chapter_number(&title);

        // Detect if we're in a "latest chapters" section (descending order)
        // A "latest chapters" section has high chapter numbers decreasing (535, 534, 533...)
        // followed by "content" section with ascending numbers (1, 2, 3...)
        if let Some(num) = chapter_num {
            if let Some(last_num) = last_chapter_num {
                // If we see a big jump from high to low (descending to ascending),
                // we've found the boundary between "latest" and "content" sections
                if last_num > 100 && num < 10 && !in_latest_section {
                    // This looks like transition from latest to content
                    in_latest_section = true;
                    // Don't add chapters from the "latest" section that we've already processed
                    // Clear the output and start fresh with the content section
                    seen_urls.clear();
                    out.clear();
                }
            }
            last_chapter_num = Some(num);
        }

        seen_urls.insert(url_abs.clone());

        out.push(BookChapter {
            title,
            url: url_abs,
            index: out.len() as i32,
        });
    }

    // Extract next_toc_url(s) - may be multiple (e.g., option dropdown for all pages)
    let rule_str = rule.next_toc_url.as_deref().unwrap_or("");
    println!("DEBUG: nextTocUrl rule: {}", rule_str);
    let raw_urls: Vec<String> = html::select_text_list(&doc, rule_str);
    println!("DEBUG: raw next_urls count: {}, values: {:?}", raw_urls.len(), raw_urls);
    let next_urls: Vec<String> = raw_urls
        .into_iter()
        .filter(|u| !u.is_empty())
        .map(|u| resolve_url(base_url, &u))
        .collect();
    println!("DEBUG: resolved next_urls: {:?}", next_urls);

    (out, next_urls)
}

fn parse_chapter_list_json(body: &str, base_url: &str, rule: &TocRule, ctx: &mut std::collections::HashMap<String, String>) -> (Vec<BookChapter>, Vec<String>) {
    let v: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return (vec![], vec![]),
    };
    // Execute init rule if present
    if let Some(init) = &rule.init {
        let _ = eval_field_json_with_ctx(init, &v, base_url, ctx);
    }
    let list_rule = rule.chapter_list.as_deref().unwrap_or("");
    let items = jsonpath::jsonpath_query(&v, list_rule);

    // Use a set to deduplicate chapters by URL
    let mut seen_urls = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let title = eval_field_json_with_ctx(rule.chapter_name.as_deref().unwrap_or(""), &item, base_url, ctx).unwrap_or_default();
        let url = eval_field_json_with_ctx(rule.chapter_url.as_deref().unwrap_or(""), &item, base_url, ctx).unwrap_or_default();

        // Skip duplicate chapters (same URL)
        if seen_urls.contains(&url) {
            continue;
        }
        seen_urls.insert(url.clone());

        out.push(BookChapter { title, url, index: out.len() as i32 });
    }

    // Extract next_toc_url(s) from JSON
    let next_urls: Vec<String> = rule.next_toc_url.as_ref()
        .map(|r| jsonpath::jsonpath_query(&v, r))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .filter(|u| !u.is_empty())
        .map(|u| resolve_url(base_url, &u))
        .collect();

    (out, next_urls)
}

fn pick_json_field(v: &Value, rule: Option<&str>) -> Option<String> {
    let rule = rule?;
    if rule.trim_start().starts_with('$') {
        return jsonpath::jsonpath_first_string(v, rule);
    }
    if let Some(obj) = v.as_object() {
        if let Some(val) = obj.get(rule) {
            return jsonpath::value_to_string(val);
        }
    }
    None
}

fn is_json(body: &str) -> bool {
    let t = body.trim_start();
    t.starts_with('{') || t.starts_with('[')
}

fn resolve_url(base: &str, url: &str) -> String {
    // Strip Legado config suffix like ##$##,{'webView': true}
    let url = strip_url_config(url);

    if url.is_empty() { return base.to_string(); }
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with("//") {
        return format!("https:{}", url);
    }

    let base_url = match url::Url::parse(base) {
        Ok(u) => u,
        Err(_) => return url.to_string(), // Fallback
    };

    if url.starts_with('/') {
        let mut out = base_url.clone();
        out.set_path(url);
        return out.to_string();
    }

    match base_url.join(url) {
        Ok(u) => u.to_string(),
        Err(_) => {
            // Manual fallback if url join fails
            let base = base.trim_end_matches('/');
            format!("{}/{}", base, url.trim_start_matches('/'))
        }
    }
}

/// Strip Legado config suffix from URL (e.g., ##$##,{'webView': true})
fn strip_url_config(url: &str) -> &str {
    if let Some(idx) = url.find("##$##") {
        &url[..idx]
    } else if let Some(idx) = url.find(",{'webView'") {
        &url[..idx]
    } else if let Some(idx) = url.find(",{\"webView\"") {
        &url[..idx]
    } else {
        url
    }
}

fn extract_js(rule: &str) -> (&str, Option<&str>) {
    if let Some(idx) = rule.find("<js>") {
        if let Some(end_idx) = rule.rfind("</js>") {
            if end_idx > idx {
                let pure = rule[..idx].trim();
                let js = &rule[idx + 4..end_idx];
                return (pure, Some(js));
            }
        }
    }
    if let Some(idx) = rule.find("@js:") {
        let pure = rule[..idx].trim();
        let js = &rule[idx + 4..];
        return (pure, Some(js));
    }
    (rule, None)
}

fn eval_field_html(rule: &str, el: &scraper::ElementRef, base_url: &str) -> Option<String> {
    eval_field_html_with_ctx(rule, el, base_url, &mut std::collections::HashMap::new())
}

fn eval_field_html_with_ctx(rule: &str, el: &scraper::ElementRef, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(res) = try_put_get_html(rule, el, base_url, ctx) {
        return Some(res);
    }
    let (pure_rule, regex_part) = split_legado_regex(rule);
    let (pure, js) = extract_js(&pure_rule);
    let mut text = if pure.is_empty() {
        "".to_string()
    } else {
        html::select_text_from_element(el, pure).unwrap_or_default()
    };
    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            text = res;
        }
    }
    if let Some(reg) = regex_part {
        text = apply_legado_regex(&text, reg);
    }
    if text.is_empty() { None } else { Some(text) }
}

fn eval_field_html_doc(rule: &str, doc: &scraper::Html, base_url: &str) -> Option<String> {
    eval_field_html_doc_with_ctx(rule, doc, base_url, &mut std::collections::HashMap::new())
}

fn eval_field_html_doc_with_ctx(rule: &str, doc: &scraper::Html, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(res) = try_put_get_html_doc(rule, doc, base_url, ctx) {
        return Some(res);
    }
    let (pure, js) = extract_js(rule);
    let text = if pure.is_empty() {
        "".to_string()
    } else {
        html::select_text(doc, pure).unwrap_or_default()
    };
    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            return Some(res);
        }
        return Some(text);
    }
    if text.is_empty() { None } else { Some(text) }
}

fn eval_field_json(rule: &str, v: &Value, base_url: &str) -> Option<String> {
    eval_field_json_with_ctx(rule, v, base_url, &mut std::collections::HashMap::new())
}

fn eval_field_json_with_ctx(rule: &str, v: &Value, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(res) = try_put_get_json(rule, v, base_url, ctx) {
        return Some(res);
    }
    let (pure_rule, regex_part) = split_legado_regex(rule);
    let (pure, js) = extract_js(&pure_rule);
    let mut text = if pure.is_empty() {
        "".to_string()
    } else {
        pick_json_field(v, Some(pure)).unwrap_or_default()
    };
    if let Some(script) = js {
        if let Ok(res) = eval_js(script, &text, base_url) {
            text = res;
        }
    }
    if let Some(reg) = regex_part {
        text = apply_legado_regex(&text, reg);
    }
    if text.is_empty() { None } else { Some(text) }
}

fn try_put_get_html(rule: &str, el: &scraper::ElementRef, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            // inner: key1:"rule1", key2:"rule2"
            // Simple comma split (might fail with nested commas, better regex needed)
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_html_with_ctx(val_rule, el, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn try_put_get_html_doc(rule: &str, doc: &scraper::Html, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_html_doc_with_ctx(val_rule, doc, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn try_put_get_json(rule: &str, v: &Value, base_url: &str, ctx: &mut std::collections::HashMap<String, String>) -> Option<String> {
    if rule.starts_with("@put:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len()-1];
            for part in inner.split(',') {
                if let Some(idx) = part.find(':') {
                    let key = part[..idx].trim();
                    let val_rule = part[idx+1..].trim().trim_matches('"');
                    let val = eval_field_json_with_ctx(val_rule, v, base_url, ctx).unwrap_or_default();
                    ctx.insert(key.to_string(), val);
                }
            }
        }
        return Some("".to_string());
    }
    if rule.starts_with("@get:") {
        let content = &rule[5..];
        if content.starts_with('{') && content.ends_with('}') {
            let key = &content[1..content.len()-1].trim();
            return ctx.get(*key).cloned();
        }
    }
    None
}

fn split_legado_regex(rule: &str) -> (String, Option<&str>) {
    if let Some(idx) = rule.find("##") {
        let (pure, reg) = rule.split_at(idx);
        return (pure.trim().to_string(), Some(reg));
    }
    (rule.to_string(), None)
}

fn apply_legado_regex(text: &str, regex_part: &str) -> String {
    if regex_part.trim().is_empty() { return text.to_string(); }
    let mut out = text.to_string();
    let parts: Vec<&str> = regex_part.split("##").collect();

    // Support both format: ##regex##replace and regex##replace
    let start_idx = if regex_part.starts_with("##") { 1 } else { 0 };

    for i in (start_idx..parts.len()).step_by(2) {
        let regex = parts[i];
        if regex.is_empty() { continue; }
        let replace = if i + 1 < parts.len() { parts[i + 1] } else { "" };
        out = apply_regex_replace(&out, regex, replace);
    }
    out
}

/// Extract chapter number from title like "535.第535章 xxx" or "第1章 xxx"
fn extract_chapter_number(title: &str) -> Option<i32> {
    // Try to match patterns like "535.第535章" or "第1章" or "第141章"
    let re = regex::Regex::new(r"(?:^|\D)(\d+)(?:\.|章|回|节)").ok()?;
    if let Some(caps) = re.captures(title) {
        if let Some(m) = caps.get(1) {
            return m.as_str().parse::<i32>().ok();
        }
    }
    // Also try just numbers at the start
    let re2 = regex::Regex::new(r"^(\d+)\.").ok()?;
    if let Some(caps) = re2.captures(title) {
        if let Some(m) = caps.get(1) {
            return m.as_str().parse::<i32>().ok();
        }
    }
    None
}
