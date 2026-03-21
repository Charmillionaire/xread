use scraper::{Html, Selector, ElementRef};

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
        // Split by space and join with dots for multiple classes
        let classes: Vec<&str> = rest.split_whitespace().collect();
        if classes.len() > 1 {
            // Multiple classes on same element: class.mod block book-all-list → .mod.block.book-all-list
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

    // Already CSS selector format - but strip any trailing .number index
    if selector.starts_with('.') {
        // Check for .className.number pattern and strip the index part
        if let Some(last_dot) = selector[1..].rfind('.') {
            let after_dot = &selector[last_dot + 2..]; // +2 because we skipped the first char
            if after_dot.parse::<i32>().is_ok() {
                return selector[..last_dot + 1].to_string();
            }
        }
        return selector.to_string();
    }

    if selector.starts_with('#') || selector.starts_with('[') {
        return selector.to_string();
    }

    // Default: treat as class if it doesn't look like a tag
    if selector.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        // Could be a tag name or class name
        // If it contains multiple words separated by space, treat as classes
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.len() > 1 {
            return format!(".{}", parts.join("."));
        }
        // Single word - could be tag or class, try as-is first
        return selector.to_string();
    }

    selector.to_string()
}

pub fn select_list<'a>(doc: &'a Html, selector: &str) -> Vec<ElementRef<'a>> {
    // Split by @ first to get the base selector
    let sel_text = selector.split('@').next().unwrap_or(selector).trim();
    let (css_selector, index) = parse_selector_with_index(sel_text);
    println!("DEBUG: select_list '{}' -> '{}' (index={})", sel_text, css_selector, index);
    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => {
            println!("DEBUG: select_list failed to parse selector: {}", css_selector);
            return vec![];
        }
    };
    let matches: Vec<ElementRef<'a>> = doc.select(&sel).collect();

    if matches.is_empty() {
        return vec![];
    }

    // If index is 0, return all (unless there's only one element)
    if index == 0 {
        return matches;
    }

    // Return specific element by index
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

pub fn select_text_from_element(el: &ElementRef, rule: &str) -> Option<String> {
    let parts: Vec<&str> = rule.split('@').collect();
    let mut current_el = el.clone();

    for i in 0..parts.len() {
        let part = parts[i].trim();
        if part.is_empty() { continue; }

        // Final part could be an attribute or special extractor (text, html)
        if i == parts.len() - 1 {
            return match part {
                "text" => Some(current_el.text().collect::<Vec<_>>().join(" ").trim().to_string()),
                "html" => Some(current_el.html()),
                attr => {
                    if attr.is_empty() {
                         Some(current_el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    } else {
                        current_el.value().attr(attr).map(|v| v.to_string())
                    }
                },
            };
        }

        // Intermediate part is a selector
        // Support index shortcut like .class.0 or .class.-1
        let (css_selector, index) = parse_selector_with_index(part);
        let sel = match Selector::parse(&css_selector) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let matches: Vec<_> = current_el.select(&sel).collect();
        if matches.is_empty() { return None; }

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

    // Default if no parts or only one part that is not an attribute
    Some(current_el.text().collect::<Vec<_>>().join(" ").trim().to_string())
}

/// Select all matching elements and collect their text, joined by newlines
/// Used for content extraction where we want all paragraphs
pub fn select_all_text(doc: &Html, rule: &str) -> Option<String> {
    let parts: Vec<&str> = rule.split('@').collect();
    if parts.is_empty() { return None; }

    let first_part = parts[0].trim();
    let (css_selector, _index) = parse_selector_with_index(first_part);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let roots: Vec<_> = doc.select(&sel).collect();
    if roots.is_empty() { return None; }

    // If there are more parts, we need to select within each root
    if parts.len() > 1 {
        let mut all_texts = Vec::new();
        let sub_rule = parts[1..].join("@");

        for root in roots {
            // Check if sub_rule is a terminal selector (text, html) - no further CSS needed
            let last_part = sub_rule.trim();
            if last_part == "text" {
                // Return text content of the root element
                let text = root.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() {
                    all_texts.push(text);
                }
                continue;
            } else if last_part == "html" {
                // Return HTML content of the root element
                all_texts.push(root.html());
                continue;
            }

            // Otherwise parse as CSS selector
            let (sub_css, sub_index) = parse_selector_with_index(sub_rule.trim());

            if let Ok(sub_sel) = Selector::parse(&sub_css) {
                let matches: Vec<_> = root.select(&sub_sel).collect();
                if sub_index == 0 {
                    // Collect all matches
                    for el in matches {
                        let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        if !text.is_empty() {
                            all_texts.push(text);
                        }
                    }
                } else {
                    // Select specific index
                    let idx = if sub_index < 0 {
                        let n = matches.len() as i32 + sub_index;
                        if n < 0 { continue; }
                        n as usize
                    } else {
                        sub_index as usize
                    };
                    if idx < matches.len() {
                        let text = matches[idx].text().collect::<Vec<_>>().join(" ").trim().to_string();
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

    // Just the root element(s)
    let mut texts = Vec::new();
    for root in roots {
        let text = root.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !text.is_empty() {
            texts.push(text);
        }
    }
    if texts.is_empty() { return None; }
    Some(texts.join("\n"))
}

pub fn select_text(doc: &Html, rule: &str) -> Option<String> {
    select_text_list(doc, rule).into_iter().next()
}

pub fn select_text_list(doc: &Html, rule: &str) -> Vec<String> {
    let parts: Vec<&str> = rule.split('@').collect();
    if parts.is_empty() { return vec![]; }

    // Handle Legado's text.XXX selector format (finds elements containing text)
    let first_part = parts[0].trim();
    if first_part.starts_with("text.") {
        let text_content = &first_part[5..];
        let mut results = Vec::new();
        // Find anchor elements and check if they contain the text
        if let Ok(sel) = Selector::parse("a") {
            for el in doc.select(&sel) {
                let el_text = el.text().collect::<Vec<_>>().join("").trim().to_string();
                if el_text.contains(text_content) {
                    // Found matching element, extract attribute if specified
                    if parts.len() > 1 {
                        let attr = parts[1].trim();
                        if let Some(v) = el.value().attr(attr) {
                            results.push(v.to_string());
                        }
                    } else {
                        results.push(el_text);
                    }
                }
            }
        }
        return results;
    }

    // For root selection, first part MUST be a selector
    let (css_selector, index) = parse_selector_with_index(first_part);

    let sel = match Selector::parse(&css_selector) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let matches: Vec<_> = doc.select(&sel).collect();
    if matches.is_empty() { return vec![]; }

    // If index is specified, only return that one
    if index != 0 || matches.len() == 1 {
        let idx = if index < 0 {
            let n = matches.len() as i32 + index;
            if n < 0 { return vec![]; }
            n as usize
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
                return vec![el.text().collect::<Vec<_>>().join(" ").trim().to_string()];
            }
        }
        return vec![];
    }

    // Return all matches
    let mut results = Vec::new();
    for el in matches {
        if parts.len() > 1 {
            let rest = parts[1..].join("@");
            if let Some(v) = select_text_from_element(&el, &rest) {
                results.push(v);
            }
        } else {
            results.push(el.text().collect::<Vec<_>>().join(" ").trim().to_string());
        }
    }
    results
}

/// Parses a selector string and extracts index shortcut like .class.0
/// Also handles Legado format like class.className
fn parse_selector_with_index(s: &str) -> (String, i32) {
    // First convert Legado format if needed
    let converted = legado_to_css(s);

    // Then check for index
    if let Some(pos) = converted.rfind('.') {
        let last_part = &converted[pos + 1..];
        if let Ok(idx) = last_part.parse::<i32>() {
            return (converted[..pos].to_string(), idx);
        }
    }
    (converted, 0)
}
