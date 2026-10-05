use reader_rust::model::book_source::BookSource;
use reader_rust::parser::js::{eval_js, with_js_lib};
use std::fs;

fn run(body: &str, label: &str) {
    // 本地样本（从书源库导出的光遇聚合 JSON）；CI 上没有则跳过。
    let Ok(source_json) = fs::read_to_string("/tmp/gy_source.json") else {
        println!("[{}] skipped: no local source sample", label);
        return;
    };
    let source: BookSource = serde_json::from_str(&source_json).unwrap();
    let rule = source.rule_content.clone().unwrap();
    let content_rule = rule.content.clone().unwrap();
    let rest = content_rule.trim().strip_prefix("<js>").expect("js rule");
    let end = rest.find("</js>").expect("close tag");
    let script = rest[..end].to_string();
    with_js_lib(source.js_lib.as_deref(), || {
        match eval_js(&script, body, "光遇聚合") {
            Ok(res) => println!("[{}] OK len={} out={}", label, res.len(), &res[..res.len().min(400)]),
            Err(e) => println!("[{}] ERR {:?}", label, e),
        }
    });
}

#[test]
fn probe_drama_content() {
    let body = r#"{"book_id":"NzY0MTI0NjUxNzU5NTk5OTI1Nw","item_id":"7641248554081602585","title":"第1集","sources":"番茄","tab":"短剧","url":""}"#;
    run(body, "drama");
}

#[test]
fn probe_audio_content() {
    let body = r#"{"book_id":"NzQ3ODcwNzI5MzMzNzQyMjg3Mw","item_id":"7479614004168639001","title":"001 黄皮葫芦","sources":"番茄","tab":"听书","url":""}"#;
    run(body, "audio");
}
