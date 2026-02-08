# htmlbuilder

A flexible html generator for rust.

```rust
use htmlbuilder::Element

fn main() 
    let root = Element::new("html", "");

    let head = Element::new("head", "")
        .add_with(Element::new("title", "My Page"))
        .add_with(
            Element::new("meta", "")
                .kws(HashMap::from([("charset", "utf-8".to_string())]))
            );
    root.add(head);

    let body = Element::new("body", "");
    root.add(body.clone());

    let div = Element::new("div", "");
    body.add(div.clone());
    let mut attrs = HashMap::new();
    attrs.insert("id", "main".to_string());
    attrs.insert("class", "container<>".to_string());
    div.configkws(attrs);
    div.configcnt("&<html><div>content&");

    // print parent element
    if let Some(parent) = div.parent() {
        println!("{}", parent.render("\n"));
    }

    div.add(Element::new("h1", "cpphtmlbuilder"));

    // append <ul>
    let ul = Element::new("ul", "");
    div.add(ul.clone());

    for i in 0..10 {
        ul.add(Element::new("li", &i.to_string()));
    }

    // Delete the penultimate li
    {
        let children_count = ul.children().len();
        if children_count >= 2 {
            ul.remove_child(children_count - 2);
        }
    }

    div.add(Element::new("", "content with blank tag name"));

    let result = root.render("\n");
    println!("{}", result);
}
```

The final output:

```html
<html>
<head>
<title>My Page</title>
<meta charset="utf-8"></meta>
</head>
<body>
<div class="container&lt;&gt;" id="main">&amp;&lt;html&gt;&lt;div&gt;content&amp;
<h1>cpphtmlbuilder</h1>
<ul>
<li>0</li>
<li>1</li>
<li>2</li>
<li>3</li>
<li>4</li>
<li>5</li>
<li>6</li>
<li>7</li>
<li>9</li>
</ul>
content with blank tag name
</div>
</body>
</html>
```

