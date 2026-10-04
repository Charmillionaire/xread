use reader_rust::crawler::url_analyzer::analyze_url;
use reader_rust::model::book_source::BookSource;

const FIXTURE: &str = "/tmp/gy_clean.json";

fn load_gy() -> Option<BookSource> {
    let raw = std::fs::read_to_string(FIXTURE).ok()?;
    let arr: Vec<BookSource> = serde_json::from_str(&raw).expect("解析书源");
    arr.into_iter().next()
}

#[test]
fn gy_explore_analyze() {
    let Some(src) = load_gy() else {
        eprintln!("跳过：未找到本地书源样本 {FIXTURE}");
        return;
    };
    let eu = src.explore_url.clone().unwrap_or_default();
    println!("exploreUrl len={}", eu.len());
    match analyze_url(&eu, "", 1, &src.book_source_url, &src) {
        Ok(spec) => println!("OK => url={:?}", spec.url),
        Err(e) => eprintln!("ERR => {:?}", e),
    }
}

#[test]
fn gy_search_analyze() {
    let Some(src) = load_gy() else {
        eprintln!("跳过：未找到本地书源样本 {FIXTURE}");
        return;
    };
    let su = src.search_url.clone().unwrap_or_default();
    println!("searchUrl len={}", su.len());
    match analyze_url(&su, "斗破", 1, &src.book_source_url, &src) {
        Ok(spec) => println!("OK => url={:?}", spec.url),
        Err(e) => eprintln!("ERR => {:?}", e),
    }
}
