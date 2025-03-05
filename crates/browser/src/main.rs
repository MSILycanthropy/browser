use html::parse_html_document;

const HTML: &str = r#"
<!DOCTYPE html>
<html>
<body>
<h1>My First Heading</h1>
<p>My first paragraph.</p>
</body>
</html>
"#;

fn main() {
    let document = parse_html_document(HTML);

    println!("{}", document.html());
}
