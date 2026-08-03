use reader_rust::parser::js::{eval_js, with_js_lib};

const GY_JSLIB_TRUNCATED: &str = r#"
let hosts = [
    'https://v1.gyks.cf', 'https://v2.gyks.cf', 'http://101.35.133.34:8888'
];
const defaultConfig = { 线路: hosts[0], 发现页来源: "番茄", 发现页类型: "小说" };
function getCloudSettings(r) {
    let c = this.getVariable('云端配置').version;
    let h = this.getVariable('云端配置')['hosts'];
    return "version=" + c + " hosts=" + (h && h.length);
}
function login(flag) {
    return this.getCloudSettings(false);
}
"#;

#[test]
fn gy_jslib_this_bound_login_no_crash() {
    // 通过 .call(globalThis) 让 login 内部 this 指向全局，模拟 Rhino 顶层 this=global 语义
    let script = r#"(function(){ return login.call(globalThis, true); })()"#;
    let result = with_js_lib(Some(GY_JSLIB_TRUNCATED), || {
        eval_js(script, "", "http://v1.gyks.cf")
    });
    match result {
        Ok(v) => {
            println!("result = {:?}", v);
            assert!(v.contains("version="), "期望能读到 version，实际: {:?}", v);
        }
        Err(e) => panic!("仍崩溃: {:?}", e),
    }
}
