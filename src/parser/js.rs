use crate::util::hash::md5_hex;
use crate::util::text::{apply_regex_replace, strip_whitespace};
use aes::Aes128;
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use chrono::{Local, TimeZone};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::Method;
use rquickjs::function::Func;
use rquickjs::{Context, Object, Runtime, Value};
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

static JS_KV: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 书源首次无源变量时的默认值：预置含 hosts 的云端配置，
/// 避免光遇等 jsLib 里 getVariable('云端配置').version / ['hosts'] 访问 null 崩溃。
/// 与 jsLib 头部 let hosts = [...] 保持一致。
static DEFAULT_VARIABLE_JSON: &str = r#"{"云端配置":{"version":"","hosts":["https://v1.qingtian618.com","https://v2.qingtian618.com","https://v3.qingtian618.com","https://v4.qingtian618.com","https://v5.qingtian618.com","https://v6.qingtian618.com","https://v7.qingtian618.com"]}}"#;
static JS_LIB_CACHE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static JS_HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("failed to build JS HTTP client")
});
static JS_DEVICE_ID: Lazy<String> = Lazy::new(|| {
    let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(existing) = map.get("__device_id") {
        return existing.clone();
    }
    let generated = Uuid::new_v4().to_string();
    map.insert("__device_id".to_string(), generated.clone());
    generated
});
type Aes128CbcDecryptor = cbc::Decryptor<Aes128>;
thread_local! {
    static ACTIVE_JS_LIB: RefCell<Option<String>> = const { RefCell::new(None) };
    static ACTIVE_SOURCE_VARIABLES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// 在一次书源请求内预置「发现页筛选变量」（线路/类型/频道/平台等），
/// 供 jsLib 的 getVariable(k) 读取，让筛选条件真正影响请求 URL。
pub fn with_js_source_variables<T>(
    variables: Option<&HashMap<String, String>>,
    f: impl FnOnce() -> T,
) -> T {
    ACTIVE_SOURCE_VARIABLES.with(|cell| {
        let previous = cell.replace(variables.cloned().unwrap_or_default());
        let result = f();
        cell.replace(previous);
        result
    })
}

fn active_source_variable(key: &str) -> Option<String> {
    ACTIVE_SOURCE_VARIABLES.with(|cell| cell.borrow().get(key).cloned())
}

/// jsLib 的 `getVariable(k)` 实现是「无参取整段 JSON → parsed[k]」，
/// 所以发现页筛选变量必须合并进这段 JSON，否则选中值永远被 defaultConfig 覆盖。
fn merge_source_variables_into_json(base: &str) -> String {
    let injected = ACTIVE_SOURCE_VARIABLES.with(|cell| cell.borrow().clone());
    if injected.is_empty() {
        return base.to_string();
    }
    let mut root = serde_json::from_str::<JsonValue>(base)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    for (key, value) in injected {
        let parsed = serde_json::from_str::<JsonValue>(&value)
            .unwrap_or(JsonValue::String(value));
        root.insert(key, parsed);
    }
    JsonValue::Object(root).to_string()
}

pub fn with_js_lib<T>(js_lib: Option<&str>, f: impl FnOnce() -> T) -> T {
    ACTIVE_JS_LIB.with(|cell| {
        let previous = cell.replace(js_lib.map(|value| value.to_string()));
        let result = f();
        cell.replace(previous);
        result
    })
}

pub fn eval_js(script: &str, input: &str, base_url: &str) -> anyhow::Result<String> {
    eval_js_inner(script, Some(input), Some(base_url), None, None, None)
}

pub fn eval_js_with_bindings(
    script: &str,
    input: &str,
    base_url: &str,
    bindings: &HashMap<String, JsonValue>,
) -> anyhow::Result<String> {
    eval_js_inner(
        script,
        Some(input),
        Some(base_url),
        None,
        None,
        Some(bindings),
    )
}

/// Evaluate a script with source-scoped host objects and JSON bindings.
/// This is used by source login scripts, which need the same `source` API as
/// search/content scripts while receiving credentials through `result`.
pub fn eval_js_with_source_bindings(
    script: &str,
    input: &str,
    base_url: &str,
    source_key: &str,
    bindings: &HashMap<String, JsonValue>,
) -> anyhow::Result<String> {
    eval_js_inner_with_source(
        script,
        Some(input),
        Some(base_url),
        None,
        None,
        Some(source_key),
        Some(bindings),
    )
}

/// Read a cookie captured by the JS host. Kept deliberately narrow so the
/// service layer does not depend on the JS KV implementation details.
pub fn get_js_cookie(url: Option<&str>) -> Option<String> {
    let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
    let key = match url {
        Some(url) => format!("__cookie_{}", url),
        None => "__cookie_all".to_string(),
    };
    if let Some(c) = map.get(&key) {
        if !c.trim().is_empty() {
            return Some(c.clone());
        }
    }
    // 兼容 setAllCookies 按 host 逐域写 cookie 的情况：扫描所有 __cookie_ 键，
    // 返回第一个非空 cookie（光遇这类书源登录后 cookie 存在 __cookie_{host} 下）。
    map.iter()
        .filter(|(k, v)| k.starts_with("__cookie_") && !v.trim().is_empty())
        .map(|(_, v)| v.clone())
        .next()
}

pub fn eval_js_search_with_source(
    script: &str,
    key: &str,
    page: i32,
    source_key: &str,
) -> anyhow::Result<String> {
    eval_js_inner_with_source(
        script,
        None,
        None,
        Some(key),
        Some(page),
        Some(source_key),
        None,
    )
}

pub fn eval_js_url(
    script: &str,
    result: &str,
    key: &str,
    page: i32,
    source_key: &str,
    base_url: &str,
) -> anyhow::Result<String> {
    eval_js_inner_with_source(
        script,
        Some(result),
        Some(base_url),
        Some(key),
        Some(page),
        Some(source_key),
        None,
    )
}

fn eval_js_inner(
    script: &str,
    input: Option<&str>,
    base_url: Option<&str>,
    key: Option<&str>,
    page: Option<i32>,
    bindings: Option<&HashMap<String, JsonValue>>,
) -> anyhow::Result<String> {
    eval_js_inner_with_source(script, input, base_url, key, page, None, bindings)
}

fn eval_js_inner_with_source(
    script: &str,
    input: Option<&str>,
    base_url: Option<&str>,
    key: Option<&str>,
    page: Option<i32>,
    source_key: Option<&str>,
    bindings: Option<&HashMap<String, JsonValue>>,
) -> anyhow::Result<String> {
    let rt = Runtime::new()?;
    let ctx = Context::full(&rt)?;
    ctx.with(|ctx| {
        let globals = ctx.globals();
        let input_value = input.unwrap_or("");
        let base_url_value = base_url.unwrap_or("");
        let base_url_owned = base_url_value.to_string();
        let shared_js = active_js_lib_script()?;

        globals.set("input", input_value)?;
        globals.set("src", input_value)?;
        // 如果 input_value 是合法 JSON 对象/数组，尝试挂载 parsed 对象并保留 JSON 字符串特性，
        // 使得无论是 result.book_id 还是 JSON.parse(result) 都能无缝兼容
        if let Ok(js_parsed) = ctx.json_parse(input_value.to_string()) {
            globals.set("result", js_parsed)?;
        } else {
            globals.set("result", input_value)?;
        }
        globals.set("base_url", base_url_value)?;
        globals.set("baseUrl", base_url_value)?;
        if let Some(key) = key {
            globals.set("key", key)?;
        }
        if let Some(page) = page {
            globals.set("page", page)?;
        }

        // Default url variable for Legado compatibility
        globals.set("url", base_url_value)?;

        // Stubs for Legado compatibility
        let source_key_val = source_key.unwrap_or("").to_string();
        let source_obj = Object::new(ctx.clone())?;
        let sk_clone = source_key_val.clone();
        source_obj.set("key", source_key_val)?;
        source_obj.set("getKey", Func::new(move || sk_clone.clone()))?;

        // ---- 光遇聚合等重度书源所需：源级变量 / 登录信息存取（参考 Rimchars BaseSource）----
        // jsLib 内部基于 source.getVariable()（无参整段 JSON）/ source.setVariable(String) 实现
        // key 版 getVariable(k)。首次无变量时返回含默认云端配置的对象，避免
        // getVariable('云端配置').version / ['hosts'] 访问 null 崩溃。
        let source_key_for_var = source_key.unwrap_or("").to_string();
        let var_key = format!("sourceVariable_{}", source_key_for_var);
        let var_key2 = var_key.clone();
        let var_key_get = var_key.clone();
        let login_key = format!("userInfo_{}", source_key_for_var);
        let login_key2 = login_key.clone();
        source_obj.set(
            "getVariable",
            Func::new(move || -> String {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                let raw = map.get(&var_key).cloned();
                drop(map);
                let base = match raw {
                    Some(s) if !s.trim().is_empty() => s,
                    // 首次无变量：预置含 hosts 的云端配置默认对象
                    _ => DEFAULT_VARIABLE_JSON.to_string(),
                };
                merge_source_variables_into_json(&base)
            }),
        )?;
        source_obj.set(
            "setVariable",
            Func::new(move |val: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(var_key2.clone(), val);
                true
            }),
        )?;
        source_obj.set(
            "putVariable",
            Func::new(move |val: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(var_key_get.clone(), val);
                true
            }),
        )?;
        source_obj.set(
            "getLoginInfo",
            Func::new(move || -> Option<String> {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.get(&login_key).cloned()
            }),
        )?;
        source_obj.set(
            "putLoginInfo",
            Func::new(move |val: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(login_key2.clone(), val);
                true
            }),
        )?;
        globals.set("source", source_obj)?;

        // ── Cookie 存储 ──
        // 使用 JS_KV 的 __cookie_{url} 键存储每域 cookie，
        // 保持 JS 层与 service 层低耦合（service 登录后从 JS_KV 读取并落库）。
        let cookie_obj = Object::new(ctx.clone())?;
        cookie_obj.set(
            "setCookie",
            Func::new(move |url: String, cookie: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                let key = format!("__cookie_{}", url);
                map.insert(key, cookie);
                true
            }),
        )?;
        cookie_obj.set(
            "getCookie",
            Func::new(move |url: String| -> Option<String> {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                let key = format!("__cookie_{}", url);
                if let Some(c) = map.get(&key) {
                    if !c.trim().is_empty() {
                        return Some(c.clone());
                    }
                }
                // 若按具体 host 未找到，回退读取 __cookie_all 或任意含 qttoken 的 cookie
                if let Some(c) = map.get("__cookie_all") {
                    if !c.trim().is_empty() {
                        return Some(c.clone());
                    }
                }
                map.iter()
                    .filter(|(k, v)| k.starts_with("__cookie_") && !v.trim().is_empty())
                    .map(|(_, v)| v.clone())
                    .find(|v| v.contains("qttoken"))
            }),
        )?;
        cookie_obj.set(
            "removeCookie",
            Func::new(move |url: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                let key = format!("__cookie_{}", url);
                map.remove(&key);
                true
            }),
        )?;
        globals.set("cookie", cookie_obj)?;

        let cache_obj = Object::new(ctx.clone())?;
        cache_obj.set(
            "get",
            Func::new(|key: String| -> Option<String> {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.get(&key).cloned()
            }),
        )?;
        cache_obj.set(
            "put",
            Func::new(|key: String, val: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(key, val);
                true
            }),
        )?;
        globals.set("cache", cache_obj)?;

        let java_obj = Object::new(ctx.clone())?;
        java_obj.set(
            "ajax",
            Func::new(|spec: String| -> String { java_ajax(&spec).unwrap_or_default() }),
        )?;
        java_obj.set(
            "md5Encode",
            Func::new(|input: String| -> String { md5_hex(&input) }),
        )?;
        java_obj.set(
            "timeFormat",
            Func::new(|timestamp: i64| -> String { java_time_format(timestamp) }),
        )?;
        java_obj.set(
            "androidId",
            Func::new(|| -> String { JS_DEVICE_ID.clone() }),
        )?;
        java_obj.set("deviceID", Func::new(|| -> String { JS_DEVICE_ID.clone() }))?;
        // Legado 语义：java.get(key) / java.put(key, value) 是读写源变量（KV），
        // 不是 HTTP 请求。光遇书源的目录/正文规则依赖它传递 book_id。
        java_obj.set(
            "get",
            Func::new(|key: String| -> Option<String> {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.get(&key).cloned()
            }),
        )?;
        java_obj.set(
            "post",
            Func::new(|url: String, body: String| -> String {
                java_request_simple("POST", &url, Some(body)).unwrap_or_default()
            }),
        )?;
        java_obj.set(
            "put",
            Func::new(|key: String, value: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(key, value);
                true
            }),
        )?;
        java_obj.set(
            "base64Encode",
            Func::new(|input: String| -> String {
                base64::engine::general_purpose::STANDARD.encode(input)
            }),
        )?;
        java_obj.set(
            "base64Decode",
            Func::new(|input: String| -> String {
                base64::engine::general_purpose::STANDARD
                    .decode(input)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_default()
            }),
        )?;
        java_obj.set(
            "aesBase64DecodeToString",
            Func::new(
                |input: String, key: String, algorithm: String, iv: String| -> String {
                    java_aes_base64_decode_to_string(&input, &key, &algorithm, &iv)
                },
            ),
        )?;
        java_obj.set(
            "encodeURIComponent",
            Func::new(|input: String| -> String { urlencoding::encode(&input).into_owned() }),
        )?;
        java_obj.set(
            "decodeURIComponent",
            Func::new(|input: String| -> String {
                urlencoding::decode(&input)
                    .map(|s| s.into_owned())
                    .unwrap_or_default()
            }),
        )?;
        java_obj.set(
            "encodeURI",
            Func::new(|input: String| -> String { urlencoding::encode(&input).into_owned() }),
        )?;
        java_obj.set(
            "decodeURI",
            Func::new(|input: String| -> String {
                urlencoding::decode(&input)
                    .map(|s| s.into_owned())
                    .unwrap_or_default()
            }),
        )?;
        java_obj.set(
            "now",
            Func::new(|| -> i64 { chrono::Utc::now().timestamp_millis() }),
        )?;
        java_obj.set(
            "uuid",
            Func::new(|| -> String { Uuid::new_v4().to_string() }),
        )?;

        // 光遇聚合书源扩展 API
        java_obj.set(
            "hexDecodeToString",
            Func::new(|input: String| -> String {
                let hex = input.trim().replace(" ", "");
                // Legado sources sometimes pass plain UTF-8 data here (notably
                // data: URL intermediates). Preserve it when the input is not
                // valid hex rather than silently turning it into garbage.
                if hex.is_empty() || hex.len() % 2 != 0 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return input;
                }
                let bytes: Vec<u8> = (0..hex.len())
                    .step_by(2)
                    .filter_map(|i| {
                        hex.get(i..i + 2)
                            .and_then(|s| u8::from_str_radix(s, 16).ok())
                    })
                    .collect();
                String::from_utf8(bytes).unwrap_or(input)
            }),
        )?;
        java_obj.set(
            "toast",
            Func::new(|msg: String| -> String {
                info!("[JS toast] {}", msg);
                String::new()
            }),
        )?;
        java_obj.set(
            "longToast",
            Func::new(|msg: String| -> String {
                info!("[JS longToast] {}", msg);
                String::new()
            }),
        )?;
        java_obj.set(
            "log",
            Func::new(|msg: String| -> String {
                info!("[JS log] {}", msg);
                String::new()
            }),
        )?;
        java_obj.set(
            "getCookie",
            Func::new(|url: String| -> String {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                let key = format!("__cookie_{}", url);
                if let Some(c) = map.get(&key) {
                    if !c.trim().is_empty() {
                        return c.clone();
                    }
                }
                if let Some(c) = map.get("__cookie_all") {
                    if !c.trim().is_empty() {
                        return c.clone();
                    }
                }
                map.iter()
                    .filter(|(k, v)| k.starts_with("__cookie_") && !v.trim().is_empty())
                    .map(|(_, v)| v.clone())
                    .find(|v| v.contains("qttoken"))
                    .unwrap_or_default()
            }),
        )?;
        java_obj.set(
            "setAllCookies",
            Func::new(|cookie: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert("__cookie_all".to_string(), cookie);
                true
            }),
        )?;
        java_obj.set(
            "getWebViewUA",
            Func::new(|| -> String {
                "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".to_string()
            }),
        )?;
        java_obj.set(
            "startBrowser",
            Func::new(|url: String, _title: String| -> String {
                info!("[JS startBrowser] url={}", url);
                url
            }),
        )?;
        java_obj.set(
            "showBrowser",
            Func::new(|url: String| -> String {
                info!("[JS showBrowser] url={}", url);
                url
            }),
        )?;
        java_obj.set(
            "refreshExplore",
            Func::new(|| -> String {
                String::new()
            }),
        )?;
        java_obj.set(
            "reLoginView",
            Func::new(|| -> String {
                String::new()
            }),
        )?;
        java_obj.set("qread", Object::new(ctx.clone())?)?;
        java_obj.set("lang", Object::new(ctx.clone())?)?;

        globals.set("java", java_obj)?;

        globals.set(
            "kv_get",
            Func::new(|key: String| -> Option<String> {
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.get(&key).cloned()
            }),
        )?;
        globals.set(
            "kv_put",
            Func::new(|key: String, val: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(key, val);
                true
            }),
        )?;
        globals.set(
            "regex_replace",
            Func::new(
                |input: String, pattern: String, replace: String| -> String {
                    apply_regex_replace(&input, &pattern, &replace)
                },
            ),
        )?;
        globals.set(
            "strip_ws",
            Func::new(|input: String| -> String { strip_whitespace(&input) }),
        )?;

        // 光遇聚合书源扩展全局函数
        // 取不到时返回空对象 {}（而非 null），避免 getVariable('云端配置').version 崩溃
        globals.set(
            "getVariable",
            Func::new(|key: String| -> String {
                if let Some(value) = active_source_variable(&key) {
                    return value;
                }
                let map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.get(&key).cloned().unwrap_or_else(|| "{}".to_string())
            }),
        )?;
        globals.set(
            "setVariable",
            Func::new(|key: String, val: String| -> bool {
                ACTIVE_SOURCE_VARIABLES.with(|cell| {
                    cell.borrow_mut().insert(key.clone(), val.clone());
                });
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.insert(key, val);
                true
            }),
        )?;
        globals.set(
            "deleteVariable",
            Func::new(|key: String| -> bool {
                let mut map = JS_KV.lock().unwrap_or_else(|e| e.into_inner());
                map.remove(&key);
                true
            }),
        )?;
        let base_url_for_request = base_url_owned.clone();
        globals.set(
            "request",
            Func::new(move |url: String, method: Option<String>, body: Option<String>, _req: Option<bool>, _index: Option<i32>| -> String {
                let method = method.unwrap_or_else(|| "GET".to_string()).to_uppercase();
                let full_url = if url.starts_with("http") {
                    url
                } else {
                    format!("{}{}", base_url_for_request, url)
                };
                java_request_simple(&method, &full_url, body).unwrap_or_default()
            }),
        )?;

        globals.set("book", Object::new(ctx.clone())?)?;
        globals.set("chapter", Object::new(ctx.clone())?)?;
        globals.set("title", "")?;
        globals.set("nextChapterUrl", "")?;
        globals.set("rssArticle", Object::new(ctx.clone())?)?;

        if let Some(bindings) = bindings {
            for (key, value) in bindings {
                let js_value = ctx.json_parse(value.to_string())?;
                globals.set(key.as_str(), js_value)?;
            }
        }

        // 注入全局兼容 Shim：
        // 1. 让 java.hexDecodeToString 支持接收 Object（自动 JSON 序列化，避免 QuickJS 抛类型转换异常）
        // 2. 补齐 Legado 书源环境中的 book / chapter 对象内置方法（如 book.getVariable / book.readConfig 等）
        // 3. 确保 globalThis / this 访问一致性
        let shim = r#"
        (function() {
            // Legado 的 java.log / toast / longToast 接受任意类型（含 boolean）。
            // QuickJS 绑定的 Rust 函数只收 String，这里先做类型转换再转发。
            if (typeof java !== 'undefined') {
                ['log', 'toast', 'longToast'].forEach(function(name) {
                    var raw = java[name];
                    if (typeof raw !== 'function') { return; }
                    java[name] = function(val) {
                        var s;
                        if (val !== null && typeof val === 'object') {
                            try { s = JSON.stringify(val); } catch (e) { s = String(val); }
                        } else {
                            s = String(val);
                        }
                        return raw(s);
                    };
                });
                // 标记当前运行环境，让书源走「直接返回媒体直链」的分支，
                // 而不是依赖原生 App 才有的 startBrowser 播放器。
                if (typeof java.getAppVariant !== 'function') {
                    java.getAppVariant = function() { return 'reader-rust'; };
                }
            }
            if (typeof java !== 'undefined' && java.hexDecodeToString) {
                const _rawHex = java.hexDecodeToString;
                java.hexDecodeToString = function(val) {
                    if (val !== null && typeof val === 'object') {
                        return JSON.stringify(val);
                    }
                    return _rawHex(val != null ? String(val) : "");
                };
            }
            if (typeof book !== 'undefined') {
                if (typeof book.getVariable !== 'function') {
                    book.getVariable = function(key) { return ""; };
                }
                if (typeof book.setVariable !== 'function') {
                    book.setVariable = function(key, val) { return true; };
                }
                if (typeof book.setUseReplaceRule !== 'function') {
                    book.setUseReplaceRule = function(val) {};
                }
                if (!book.readConfig) {
                    book.readConfig = { useReplaceRule: false };
                }
                if (typeof book.durChapterIndex === 'undefined') {
                    book.durChapterIndex = 0;
                }
                if (typeof book.durChapterTitle === 'undefined') {
                    book.durChapterTitle = "";
                }
                if (typeof book.order === 'undefined') {
                    book.order = 0;
                }
            }
            if (typeof chapter !== 'undefined') {
                if (typeof chapter.index === 'undefined') {
                    chapter.index = 0;
                }
                if (typeof chapter.title === 'undefined') {
                    chapter.title = "";
                }
            }
        })();
        "#;
        let mut shim_opts = rquickjs::context::EvalOptions::default();
        shim_opts.strict = false;
        let _ = ctx.eval_with_options::<(), _>(shim, shim_opts);

        if !shared_js.trim().is_empty() {
            eval_script(ctx.clone(), &shared_js)?;
        }

        let v = eval_script(ctx.clone(), script)?;

        let result = if v.is_null() || v.is_undefined() {
            String::new()
        } else if let Some(s) = v.clone().into_string() {
            let s: rquickjs::String<'_> = s;
            s.to_string()
                .map(|value| value.to_string())
                .unwrap_or_default()
        } else {
            match ctx.json_stringify(v) {
                Ok(Some(json)) => json.to_string().unwrap_or_default(),
                _ => String::new(),
            }
        };
        Ok(result)
    })
}

fn java_aes_base64_decode_to_string(input: &str, key: &str, algorithm: &str, iv: &str) -> String {
    let algorithm = algorithm.to_ascii_uppercase();
    if algorithm != "AES/CBC/PKCS5PADDING" && algorithm != "AES/CBC/PKCS7PADDING" {
        return String::new();
    }

    let Ok(mut encrypted) = base64::engine::general_purpose::STANDARD.decode(input.trim()) else {
        return String::new();
    };

    let Ok(cipher) = Aes128CbcDecryptor::new_from_slices(key.as_bytes(), iv.as_bytes()) else {
        return String::new();
    };

    cipher
        .decrypt_padded_mut::<Pkcs7>(&mut encrypted)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
        .unwrap_or_default()
}

/// 仅替换字符串字面量（'...' / "..." / `...`）之外的 `{{name}}` 占位符。
/// Legado 发现页脚本里既有 `let {{key}} = ...` 这种语法（需替换），
/// 也有 `"/bookshelf?page={{page}}"` 这种要原样返回给抓取层的 URL 模板（不能动）。
fn replace_bare_legado_placeholders(script: &str) -> String {
    if !script.contains("{{") {
        return script.to_string();
    }

    let chars: Vec<char> = script.chars().collect();
    let mut out = String::with_capacity(script.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];

        if let Some(q) = quote {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
            i += 1;
            continue;
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            quote = Some(ch);
            out.push(ch);
            i += 1;
            continue;
        }

        if ch == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some(end) = (i + 2..chars.len()).find(|&j| chars[j] == '}') {
                let name: String = chars[i + 2..end].iter().collect();
                let is_ident = !name.is_empty()
                    && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if is_ident && end + 1 < chars.len() && chars[end + 1] == '}' {
                    out.push_str(&name);
                    i = end + 2;
                    continue;
                }
            }
        }

        out.push(ch);
        i += 1;
    }

    out
}

fn eval_script<'js>(ctx: rquickjs::Ctx<'js>, script: &str) -> anyhow::Result<Value<'js>> {
    // Legado/Rhino accepts `\\{` in source strings as a literal `{`; QuickJS
    // rejects it as an invalid escape. Normalize this narrow compatibility spelling.
    let script = script.replace("\\{", "{");
    // Legado 的发现页脚本会在源码里直接写 `let {{key}} = ...`（把占位符当变量名用），
    // QuickJS 会以 "invalid property name" 报错。这里只在**字符串字面量之外**
    // 把 {{key}} / {{page}} 还原成同名变量，避免破坏 URL 模板（如 "?page={{page}}"）。
    let script = replace_bare_legado_placeholders(&script);
    // QuickJS 默认 strict=true 会让普通函数调用时 this=undefined，
    // 而 Legado(Rhino) 书源 jsLib 大量依赖 this.xxx（this 指向全局）。
    // 设为非严格后，普通函数调用 this 指向 globalThis，兼容光遇等重度书源。
    let mut opts = rquickjs::context::EvalOptions::default();
    opts.strict = false;
    match ctx.eval_with_options::<Value<'js>, String>(script.to_string(), opts) {
        Ok(v) => Ok(v),
        Err(e) => {
            if let Some(exception) = ctx.catch().into_exception() {
                return Err(anyhow::anyhow!("JS Exception: {:?}", exception));
            }
            Err(e.into())
        }
    }
}

fn active_js_lib_script() -> anyhow::Result<String> {
    let js_lib = ACTIVE_JS_LIB.with(|cell| cell.borrow().clone());
    let Some(js_lib) = js_lib.filter(|value| !value.trim().is_empty()) else {
        return Ok(String::new());
    };
    let cache_key = md5_hex(&js_lib);
    if let Some(cached) = JS_LIB_CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&cache_key)
        .cloned()
    {
        return Ok(cached);
    }

    let compiled = compile_js_lib(&js_lib)?;
    JS_LIB_CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(cache_key, compiled.clone());
    Ok(compiled)
}

fn compile_js_lib(js_lib: &str) -> anyhow::Result<String> {
    let trimmed = js_lib.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) {
            if let Some(map) = value.as_object() {
                let mut scripts = Vec::new();
                for entry in map.values() {
                    if let Some(raw) = entry.as_str() {
                        scripts.push(resolve_js_lib_entry(raw)?);
                    }
                }
                return Ok(scripts.join("\n"));
            }
        }
    }
    Ok(trimmed.to_string())
}

fn resolve_js_lib_entry(entry: &str) -> anyhow::Result<String> {
    let value = entry.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        let response = JS_HTTP_CLIENT.get(value).send()?;
        return Ok(response.text().unwrap_or_default());
    }
    Ok(value.to_string())
}

fn java_time_format(timestamp: i64) -> String {
    let secs = if timestamp > 1_000_000_000_000 {
        timestamp / 1000
    } else {
        timestamp
    };
    match Local.timestamp_opt(secs, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => String::new(),
    }
}

fn java_ajax(spec: &str) -> anyhow::Result<String> {
    let (url, options) = split_ajax_spec(spec);
    if url.trim().is_empty() {
        return Ok(String::new());
    }

    let options_json = options
        .and_then(|raw| serde_json::from_str::<JsonValue>(raw).ok())
        .unwrap_or(JsonValue::Null);

    let method = options_json
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);

    let mut req = JS_HTTP_CLIENT.request(method, url.trim());

    if let Some(headers) = options_json.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in headers {
            if let Some(value) = value.as_str() {
                req = req.header(key, value);
            } else if !value.is_null() {
                req = req.header(key, value.to_string());
            }
        }
    }

    if let Some(body) = options_json.get("body") {
        if let Some(body) = body.as_str() {
            req = req.body(body.to_string());
        } else if !body.is_null() {
            req = req.body(body.to_string());
        }
    }

    let response = req.send()?;
    Ok(response.text().unwrap_or_default())
}

fn java_request_simple(method: &str, url: &str, body: Option<String>) -> anyhow::Result<String> {
    let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);
    let mut req = JS_HTTP_CLIENT.request(method, url.trim());
    if let Some(body) = body {
        req = req.body(body);
    }
    let response = req.send()?;
    Ok(response.text().unwrap_or_default())
}

fn split_ajax_spec(spec: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut quote = '\0';
    let mut escaped = false;

    for (idx, ch) in spec.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escaped = true;
            }
            '"' | '\'' if in_string && ch == quote => {
                in_string = false;
                quote = '\0';
            }
            '"' | '\'' if !in_string => {
                in_string = true;
                quote = ch;
            }
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                let left = &spec[..idx];
                let right = &spec[idx + ch.len_utf8()..];
                return (left, Some(right.trim()));
            }
            _ => {}
        }
    }

    (spec, None)
}

#[cfg(test)]
mod tests {
    use super::{
        eval_js, replace_bare_legado_placeholders, with_js_source_variables,
    };
    use std::collections::HashMap;

    /// jsLib 的 `getVariable(k)` 是「无参取整段 JSON → parsed[k]」，
    /// 所以发现页筛选变量必须合并进这段 JSON；否则用户选的「女频/漫画」
    /// 会被 defaultConfig 里的「男频/小说」静默覆盖，筛选看起来完全失灵。
    #[test]
    fn injects_source_variables_into_noarg_get_variable() {
        let mut vars = HashMap::new();
        vars.insert("频道".to_string(), "女频".to_string());
        vars.insert("发现页类型".to_string(), "漫画".to_string());

        let result = with_js_source_variables(Some(&vars), || {
            eval_js(
                r#"
                const parsed = JSON.parse(source.getVariable());
                parsed['频道'] + '/' + parsed['发现页类型'];
                "#,
                "",
                "https://example.com",
            )
            .unwrap()
        });

        assert_eq!(result, "女频/漫画");
    }

    /// 未注入变量时不能凭空造值，应回落到书源自带的默认配置。
    #[test]
    fn leaves_variables_untouched_without_injection() {
        let result = eval_js(
            r#"
            const parsed = JSON.parse(source.getVariable());
            String(parsed['云端配置'] !== undefined && parsed['频道'] === undefined);
            "#,
            "",
            "https://example.com",
        )
        .unwrap();

        assert_eq!(result, "true");
    }

    /// 光遇聚合等发现页脚本会把模板占位符当变量名直接用（`let {{key}} = ...`），
    /// QuickJS 会以 "invalid property name" 报错；该拼写必须被还原成普通变量。
    #[test]
    fn replaces_placeholder_outside_string_literals() {
        let cleaned = replace_bare_legado_placeholders("let {{key}} = 5; String({{key}} + 1);");
        assert_eq!(cleaned, "let key = 5; String(key + 1);");
    }

    /// 字符串字面量里的占位符是抓取层要消费的 URL 模板，绝不能在 JS 求值阶段被改掉，
    /// 否则 `{{page}}` 会被替换成变量而丢失翻页参数。
    #[test]
    fn preserves_placeholder_inside_string_literals() {
        let cleaned = replace_bare_legado_placeholders(r#"let u = "/bookshelf?page={{page}}"; u;"#);
        assert_eq!(cleaned, r#"let u = "/bookshelf?page={{page}}"; u;"#);
    }

    /// 端到端：带占位符的脚本现在能在 QuickJS 里正常求值。
    #[test]
    fn eval_js_accepts_bare_legado_placeholder() {
        let result = eval_js("let {{key}} = 5; String({{key}} + 1);", "", "https://example.com").unwrap();
        assert_eq!(result, "6");
    }
}
