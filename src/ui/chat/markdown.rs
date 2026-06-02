use ammonia::Builder;
use pulldown_cmark::{Options, Parser, html};

pub fn render_markdown(input: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, opts);
    let mut unsafe_html = String::new();
    html::push_html(&mut unsafe_html, parser);
    Builder::default()
        .url_schemes(["http", "https", "mailto"].into_iter().collect())
        .clean(&unsafe_html)
        .to_string()
}
