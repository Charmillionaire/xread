use xread::parser::js::{eval_explore_script, with_js_lib};

/// 复刻 parse_explore_kinds 的求值流程（走生产函数 eval_explore_script），
/// 验证「平台」控件 chars 非空，且覆盖七猫/塔读/QQ阅读等平台。
/// 需要真实书源样本 /tmp/gy_source.json，缺失时跳过。
#[test]
fn explore_platform_chars_not_empty() {
    let Ok(json) = std::fs::read_to_string("/tmp/gy_source.json") else {
        eprintln!("跳过：未找到本地书源样本 /tmp/gy_source.json");
        return;
    };
    let src: serde_json::Value = serde_json::from_str(&json).expect("解析书源");
    let jslib = src["jsLib"].as_str().unwrap();
    let raw = src["exploreUrl"].as_str().unwrap();
    let base_url = src["bookSourceUrl"].as_str().unwrap();

    let script = raw
        .trim()
        .strip_prefix("<js>")
        .and_then(|v| v.strip_suffix("</js>"))
        .expect("exploreUrl should be a <js> block");

    let text = with_js_lib(Some(jslib), || eval_explore_script(script, base_url))
        .expect("explore script should evaluate");

    let kinds: Vec<serde_json::Value> = serde_json::from_str(&text).expect("explore output is JSON");
    let platform = kinds
        .iter()
        .find(|k| k["title"] == "平台")
        .expect("平台 control exists");
    let chars = platform["chars"].as_array().expect("chars must be an array");

    println!("平台 chars 数量 = {}", chars.len());
    println!("前 8 项 = {:?}", &chars[..chars.len().min(8)]);
    assert!(chars.len() >= 10, "平台候选过少: {}", chars.len());
    for expected in ["七猫", "塔读", "QQ阅读"] {
        assert!(
            chars.iter().any(|c| c.as_str() == Some(expected)),
            "缺少平台 {expected}"
        );
    }
}
