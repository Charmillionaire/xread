use scraper::{Html, Selector, ElementRef};
use std::collections::HashSet;

pub fn parse_document(html: &str) -> Html {
    Html::parse_document(html)
}

/// Convert Legado selector format to CSS selector
/// Legado formats:
/// - class.xxx yyy zzz → .xxx.yyy.zzz (multiple classes on one element)
/// - class.xxx or .xxx → .xxx
/// - tag.xxx → xxx (tag name)
/// - id.xxx or #xxx → #xxx
/// - tag.xxx@tag.yyy → nested selectors (split by @)
fn legado_to_css(selector: &str) -> String {
    let selector = selector.trim();

    // Handle special "class." prefix - multiple classes separated by space
    if selector.starts_with("class.") {
        let rest = &selector[6..];
        let classes: Vec<&str> = rest.split_whitespace().collect();
        if classes.len() > 1 {
            return format!(".{}", classes.join("."));
        } else {
            return format!(".{}", rest.trim());
        }
    }

    // Handle id. prefix
    if selector.starts_with("id.") {
        return format!("#{}", &selector[3..]);
    }

    // Handle tag. prefix
    if selector.starts_with("tag.") {
        return selector[4..].to_string();
    }

    // Already CSS selector format (index will be stripped in parse_selector_with_index)
    if selector.starts_with('.') {
        return selector.to_string();
    }

    if selector.starts_with('#') || selector.starts_with('[') {
        return selector.to_string();
    }

    // Default: treat as class if it doesn't look like a tag
    if selector.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.len() > 1 {
            return format!(".{}", parts.join("."));
        }
        return selector.to_string();
    }

    selector.to_string()
}

/// Parse index shortcuts from selector
/// Supports:
/// - .class.0 → first element
/// - .class.-1 → last element
/// - .class!0 → exclude first element
fn parse_selector_with_index(s: &str) -> (String, Vec<i32>, Option<Vec<i32>>) {
    let converted = legado_to_css(s);

    // Check for exclusion syntax (!)
    if let Some(pos) = converted.rfind('!') {
        let before = &converted[..pos];
        let after = &converted[pos + 1..];
        if let Ok(idx) = after.parse::<i32>() {
            return (before.to_string(), vec![0], Some(vec![idx]));
        }
    }

    // Check for range syntax (start:end)
    if let Some(pos) = converted.rfind('.') {
        let last_part = &converted[pos + 1..];

        if last_part.contains(':') {
            let parts: Vec<&str> = last_part.split(':').collect();
            if parts.len() >= 2 {
                let start: i32 = parts[0].parse().unwrap_or(0);
                let end: i32 = parts[1].parse().unwrap_or(-1);
                return (converted[..pos].to_string(), vec![start, end], None);
            }
        }

        if let Ok(idx) = last_part.parse::<i32>() {
            return (converted[..pos].to_string(), vec![idx], None);
        }
    }

    (converted, vec![0], None)
}

/// Select elements with Legado rule syntax
pub fn select_list<'a>(doc: &'a Html, selector: &str) -> Vec<ElementRef<'a>> {
    let selector = selector.trim();

    // Split by @ first to get the base selector (handle @@ separately in text extraction)
    let sel_text = selector.split("@@").next().unwrap_or(selector).trim();
    let sel_text = sel_text.split('@').next().unwrap_or(sel_text).trim();

    // Handle list combination operators at the top level
    if sel_text.contains("&&") || sel_text.contains("||") || sel_text.contains("%%") {
        return select_with_combination(doc, sel_text);
    }

    let (css_selector, indices, exclude) = parse_selector_with_index(sel_text);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let matches: Vec<ElementRef<'a>> = doc.select(&sel).collect();

    if matches.is_empty() {
        return vec![];
    }

    // Handle exclusion
    if let Some(exclude_indices) = exclude {
        let exclude_set: HashSet<i32> = exclude_indices.into_iter().collect();
        return matches.into_iter().enumerate()
            .filter(|(i, _)| !exclude_set.contains(&(*i as i32)))
            .map(|(_, el)| el)
            .collect();
    }

    // Handle range selection (start:end)
    if indices.len() == 2 {
        let start = indices[0] as usize;
        let end = if indices[1] < 0 { matches.len() } else { indices[1] as usize + 1 };
        return matches.into_iter().skip(start).take(end.saturating_sub(start)).collect();
    }

    // Single index (0 means all)
    let index = indices.get(0).copied().unwrap_or(0);
    if index == 0 {
        return matches;
    }

    let idx = if index < 0 {
        let n = matches.len() as i32 + index;
        if n < 0 { return vec![]; }
        n as usize
    } else {
        index as usize
    };

    if idx < matches.len() {
        vec![matches[idx]]
    } else {
        vec![]
    }
}

/// Handle list combination operators
fn select_with_combination<'a>(doc: &'a Html, rule: &str) -> Vec<ElementRef<'a>> {
    let mut rules: Vec<&str> = Vec::new();
    let mut operators: Vec<char> = Vec::new();

    let mut current = rule;
    while !current.is_empty() {
        let mut found = None;
        for (i, c) in current.char_indices() {
            if c == '&' && current.chars().nth(i + 1) == Some('&') {
                found = Some((i, "&&"));
                break;
            } else if c == '|' && current.chars().nth(i + 1) == Some('|') {
                found = Some((i, "||"));
                break;
            } else if c == '%' && current.chars().nth(i + 1) == Some('%') {
                found = Some((i, "%%"));
                break;
            }
        }

        if let Some((pos, op)) = found {
            rules.push(current[..pos].trim());
            operators.push(op.chars().next().unwrap());
            current = current[pos + 2..].trim();
        } else {
            rules.push(current.trim());
            break;
        }
    }

    if rules.is_empty() {
        return vec![];
    }

    let mut result = select_list_simple(doc, rules[0]);

    for (i, op) in operators.into_iter().enumerate() {
        let next_results = select_list_simple(doc, rules[i + 1]);

        match op {
            '&' => {
                result.extend(next_results);
            }
            '|' => {
                if result.is_empty() {
                    result = next_results;
                }
            }
            '%' => {
                let mut zipped = Vec::new();
                let max_len = result.len().max(next_results.len());
                for j in 0..max_len {
                    if j < result.len() {
                        zipped.push(result[j]);
                    }
                    if j < next_results.len() {
                        zipped.push(next_results[j]);
                    }
                }
                result = zipped;
            }
            _ => {}
        }
    }

    result
}

/// Simple select without combination operators
fn select_list_simple<'a>(doc: &'a Html, selector: &str) -> Vec<ElementRef<'a>> {
    let (css_selector, indices, exclude) = parse_selector_with_index(selector);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let matches: Vec<ElementRef<'a>> = doc.select(&sel).collect();

    if matches.is_empty() {
        return vec![];
    }

    if let Some(exclude_indices) = exclude {
        let exclude_set: HashSet<i32> = exclude_indices.into_iter().collect();
        return matches.into_iter().enumerate()
            .filter(|(i, _)| !exclude_set.contains(&(*i as i32)))
            .map(|(_, el)| el)
            .collect();
    }

    if indices.len() == 2 {
        let start = indices[0] as usize;
        let end = if indices[1] < 0 { matches.len() } else { indices[1] as usize + 1 };
        return matches.into_iter().skip(start).take(end.saturating_sub(start)).collect();
    }

    let index = indices.get(0).copied().unwrap_or(0);
    if index == 0 {
        return matches;
    }

    let idx = if index < 0 {
        let n = matches.len() as i32 + index;
        if n < 0 { return vec![]; }
        n as usize
    } else {
        index as usize
    };

    if idx < matches.len() {
        vec![matches[idx]]
    } else {
        vec![]
    }
}

/// Extract text from element with various Legado extractors
pub fn extract_text(el: &ElementRef, extractor: &str) -> Option<String> {
    let extractor = extractor.trim();

    match extractor {
        "text" | "@text" => {
            let text = el.text().collect::<Vec<_>>().join(" ");
            let text = text.trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        "textNodes" | "@textNodes" => {
            let text = get_text_nodes(el);
            if text.is_empty() { None } else { Some(text) }
        }
        "ownText" | "@ownText" => {
            let mut own_text = String::new();
            for node in el.children() {
                if let Some(text_node) = node.value().as_text() {
                    own_text.push_str(text_node.text.trim());
                    own_text.push(' ');
                }
            }
            let text = own_text.trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        "html" | "@html" => {
            Some(el.html())
        }
        "all" | "@all" => {
            Some(el.html())
        }
        _ => {
            if extractor.starts_with('@') {
                el.value().attr(&extractor[1..]).map(|v| v.to_string())
            } else {
                el.value().attr(extractor).map(|v| v.to_string())
            }
        }
    }
}

/// Get all text nodes from an element, preserving structure
fn get_text_nodes(el: &ElementRef) -> String {
    let mut texts = Vec::new();
    collect_text_nodes(*el, &mut texts);
    texts.join("\n")
}

fn collect_text_nodes(el: ElementRef, texts: &mut Vec<String>) {
    for node in el.children() {
        if let Some(text_node) = node.value().as_text() {
            let text = text_node.text.trim().to_string();
            if !text.is_empty() {
                texts.push(text);
            }
        }
        if let Some(child_el) = ElementRef::wrap(node) {
            collect_text_nodes(child_el, texts);
        }
    }
}

pub fn select_text_from_element(el: &ElementRef, rule: &str) -> Option<String> {
    let parts: Vec<&str> = rule.split('@').collect();
    let mut current_el = el.clone();

    for i in 0..parts.len() {
        let part = parts[i].trim();
        if part.is_empty() { continue; }

        if i == parts.len() - 1 {
            return extract_text(&current_el, part);
        }

        let (css_selector, indices, exclude) = parse_selector_with_index(part);
        let sel = match Selector::parse(&css_selector) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let matches: Vec<_> = current_el.select(&sel).collect();
        if matches.is_empty() { return None; }

        if let Some(exclude_indices) = exclude {
            let exclude_set: HashSet<i32> = exclude_indices.into_iter().collect();
            let filtered: Vec<_> = matches.into_iter().enumerate()
                .filter(|(j, _)| !exclude_set.contains(&(*j as i32)))
                .map(|(_, e)| e)
                .collect();
            if filtered.is_empty() { return None; }
            current_el = filtered.into_iter().next()?;
            continue;
        }

        if indices.len() == 2 {
            let start = indices[0] as usize;
            let end = if indices[1] < 0 { matches.len() } else { indices[1] as usize + 1 };
            if start >= end || start >= matches.len() { return None; }
            current_el = matches[start];
            continue;
        }

        let index = indices.get(0).copied().unwrap_or(0);
        if index == 0 && matches.len() == 1 {
            current_el = matches.into_iter().next().unwrap();
        } else {
            let idx = if index < 0 {
                let n = matches.len() as i32 + index;
                if n < 0 { return None; }
                n as usize
            } else {
                index as usize
            };

            if idx < matches.len() {
                current_el = matches[idx];
            } else {
                return None;
            }
        }
    }

    extract_text(&current_el, "text")
}

/// Select all matching elements and collect their text, joined by newlines
pub fn select_all_text(doc: &Html, rule: &str) -> Option<String> {
    let parts: Vec<&str> = rule.split('@').collect();
    if parts.is_empty() { return None; }

    let first_part = parts[0].trim();
    let (css_selector, indices, exclude) = parse_selector_with_index(first_part);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let roots: Vec<_> = doc.select(&sel).collect();
    if roots.is_empty() { return None; }

    let roots: Vec<_> = if let Some(exclude_indices) = exclude {
        let exclude_set: HashSet<i32> = exclude_indices.into_iter().collect();
        roots.into_iter().enumerate()
            .filter(|(i, _)| !exclude_set.contains(&(*i as i32)))
            .map(|(_, el)| el)
            .collect()
    } else {
        roots
    };

    let roots: Vec<_> = if indices.len() == 2 {
        let start = indices[0] as usize;
        let end = if indices[1] < 0 { roots.len() } else { indices[1] as usize + 1 };
        roots.into_iter().skip(start).take(end.saturating_sub(start)).collect()
    } else {
        roots
    };

    if parts.len() > 1 {
        let mut all_texts = Vec::new();
        let sub_rule = parts[1..].join("@");

        for root in roots {
            let last_part = sub_rule.trim();
            if let Some(text) = extract_text(&root, last_part) {
                if !text.is_empty() {
                    all_texts.push(text);
                }
                continue;
            }

            let (sub_css, _, _) = parse_selector_with_index(sub_rule.trim());
            if let Ok(sub_sel) = Selector::parse(&sub_css) {
                for el in root.select(&sub_sel) {
                    if let Some(text) = extract_text(&el, "text") {
                        if !text.is_empty() {
                            all_texts.push(text);
                        }
                    }
                }
            };
        }

        if all_texts.is_empty() { return None; }
        return Some(all_texts.join("\n"));
    }

    let mut texts = Vec::new();
    for root in roots {
        if let Some(text) = extract_text(&root, "textNodes") {
            if !text.is_empty() {
                texts.push(text);
            }
        }
    }
    if texts.is_empty() { return None; }
    Some(texts.join("\n"))
}

pub fn select_text(doc: &Html, rule: &str) -> Option<String> {
    select_text_list(doc, rule).into_iter().next()
}

pub fn select_text_list(doc: &Html, rule: &str) -> Vec<String> {
    // Handle rule chaining with @@
    if rule.contains("@@") {
        let rules: Vec<&str> = rule.split("@@").collect();
        if rules.is_empty() {
            return vec![];
        }

        // Start with first rule - get all matching texts
        let mut current_texts = select_text_list(doc, rules[0]);

        // Apply subsequent rules
        for r in rules.iter().skip(1) {
            if current_texts.is_empty() {
                break;
            }
            let mut new_texts = Vec::new();
            for text in &current_texts {
                let sub_doc = Html::parse_document(text);
                new_texts.extend(select_text_list(&sub_doc, r));
            }
            current_texts = new_texts;
        }

        return current_texts;
    }

    let parts: Vec<&str> = rule.split('@').collect();
    if parts.is_empty() { return vec![]; }

    let first_part = parts[0].trim();

    // Handle Legado's text.XXX selector format
    if first_part.starts_with("text.") {
        let text_content = &first_part[5..];
        let mut results = Vec::new();
        if let Ok(sel) = Selector::parse("a") {
            for el in doc.select(&sel) {
                let el_text = el.text().collect::<Vec<_>>().join("").trim().to_string();
                if el_text.contains(text_content) {
                    if parts.len() > 1 {
                        let attr = parts[1].trim();
                        if let Some(v) = extract_text(&el, attr) {
                            results.push(v);
                        }
                    } else {
                        results.push(el_text);
                    }
                }
            }
        }
        return results;
    }

    let (css_selector, indices, exclude) = parse_selector_with_index(first_part);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let matches: Vec<_> = doc.select(&sel).collect();
    if matches.is_empty() { return vec![]; }

    let matches: Vec<_> = if let Some(exclude_indices) = exclude {
        let exclude_set: HashSet<i32> = exclude_indices.into_iter().collect();
        matches.into_iter().enumerate()
            .filter(|(i, _)| !exclude_set.contains(&(*i as i32)))
            .map(|(_, el)| el)
            .collect()
    } else {
        matches
    };

    let matches: Vec<_> = if indices.len() == 2 {
        let start = indices[0] as usize;
        let end = if indices[1] < 0 { matches.len() } else { indices[1] as usize + 1 };
        matches.into_iter().skip(start).take(end.saturating_sub(start)).collect()
    } else {
        matches
    };

    let index = indices.get(0).copied().unwrap_or(0);

    if index != 0 || matches.len() == 1 {
        let idx = if index < 0 {
            let n = matches.len() as i32 + index;
            if n < 0 { return vec![]; }
            n as usize
        } else if index == 0 {
            0
        } else {
            index as usize
        };

        if idx < matches.len() {
            let el = matches[idx];
            if parts.len() > 1 {
                let rest = parts[1..].join("@");
                if let Some(v) = select_text_from_element(&el, &rest) {
                    return vec![v];
                }
            } else {
                return vec![extract_text(&el, "text").unwrap_or_default()];
            }
        }
        return vec![];
    }

    let mut results = Vec::new();
    for el in matches {
        if parts.len() > 1 {
            let rest = parts[1..].join("@");
            if let Some(v) = select_text_from_element(&el, &rest) {
                results.push(v);
            }
        } else {
            results.push(extract_text(&el, "text").unwrap_or_default());
        }
    }
    results
}

/// XPath support using sxd-xpath
pub fn select_xpath(html: &str, xpath: &str) -> Vec<String> {
    let package = match sxd_document::parser::parse(html) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let document = package.as_document();
    let context = sxd_xpath::Context::new();

    match sxd_xpath::Factory::new().build(xpath) {
        Ok(Some(xpath_expr)) => {
            match xpath_expr.evaluate(&context, document.root()) {
                Ok(value) => {
                    match value {
                        sxd_xpath::Value::Nodeset(ns) => {
                            ns.into_iter()
                                .map(|n| n.string_value())
                                .collect()
                        }
                        sxd_xpath::Value::String(s) => vec![s],
                        sxd_xpath::Value::Number(n) => vec![n.to_string()],
                        sxd_xpath::Value::Boolean(b) => vec![b.to_string()],
                    }
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legado_to_css() {
        assert_eq!(legado_to_css("class.mod block"), ".mod.block");
        assert_eq!(legado_to_css("class.test"), ".test");
        assert_eq!(legado_to_css("id.main"), "#main");
        assert_eq!(legado_to_css("tag.div"), "div");
    }

    #[test]
    fn test_parse_selector_with_index() {
        // .test.0 -> index 0 (which means all)
        let (sel, idx, _) = parse_selector_with_index(".test.0");
        assert_eq!(sel, ".test");
        assert_eq!(idx, vec![0]);

        // .test.-1 -> last element
        let (sel, idx, _) = parse_selector_with_index(".test.-1");
        assert_eq!(sel, ".test");
        assert_eq!(idx, vec![-1]);

        // .test!0 -> exclude first element
        let (sel, _, exclude) = parse_selector_with_index(".test!0");
        assert_eq!(sel, ".test");
        assert_eq!(exclude, Some(vec![0]));
    }
}