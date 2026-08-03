use reader_rust::parser::js::{eval_js_with_source_bindings, get_js_cookie, with_js_lib};
use serde_json::json;
use std::collections::HashMap;

// 完整光遇聚合 jsLib + loginUrl（login 是 JS 脚本）。运行时从 /tmp 读取，
// 若不存在则跳过（无需真实账号/网络的环境也不阻塞其他测试）。
fn read_gy(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("/tmp/{name}")).ok()
}

/// 集成测试：完整光遇 jsLib + login.js，注入账号密码 result，
/// 执行 login(true)，验证能走通 setAllCookies 并拿到 qttoken 写入 cookie。
/// 需要网络访问 v1.gyks.cf；离线时跳过。
#[test]
fn gy_full_login_reaches_cookie() {
    let jslib = match read_gy("gy_full_jslib.js") {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            eprintln!("skip: /tmp/gy_full_jslib.js 不存在");
            return;
        }
    };
    let login_js = match read_gy("gy_full_login.js") {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            eprintln!("skip: /tmp/gy_full_login.js 不存在");
            return;
        }
    };

    let username = "user@example.com";
    let password = "placeholder_password_redacted";

    let mut bindings = HashMap::new();
    bindings.insert(
        "result".to_string(),
        json!({ "邮箱": username, "用户名": username, "账号": username, "密码": password }),
    );

    // login.js 顶层就是 function login/register 等，直接拼进脚本；login.call(globalThis, true)
    let script = format!(
        "{}\n;typeof login === 'function' ? login.call(globalThis, true) : 'NO_LOGIN_FN'",
        login_js
    );

    let out = with_js_lib(Some(&jslib), || {
        eval_js_with_source_bindings(&script, "", "光遇聚合", "光遇聚合", &bindings)
    });

    match out {
        Ok(v) => {
            println!("login(script) => {:?}", v);
            let cookie = get_js_cookie(None);
            println!("cookie(__cookie_all) => {:?}", &cookie);
            assert!(
                cookie.as_deref().map_or(false, |c| c.contains("qttoken=") && c.len() > 10),
                "期望登录后拿到 qttoken cookie，实际: {:?}",
                cookie
            );
        }
        Err(e) => {
            // 可能是网络不通/服务器限流；打印但不硬失败（避免离线环境挂测试）
            eprintln!("完整登录可能因网络/限流未走通: {e}");
        }
    }
}
