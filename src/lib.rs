use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::fmt;


fn escape_ascii(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }

    result
}

fn un_escape_ascii(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    let mut iter = s.chars();
    while let Some(c) = iter.next() {
        if c == '&' {
            let mut name = String::new();
            while let Some(c) = iter.next() {
                if c == ';' {
                    break;
                }
                name.push(c);
            }
            match name.as_str() {
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "amp" => result.push('&'),
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                _ => result.push('&'),
            }
        } else {
            result.push(c);
        }
    }

    result
}


#[derive(Clone)]
pub struct Element {
    inner: Rc<RefCell<ElementInner>>,
}

struct ElementInner {
    parent: Option<Weak<RefCell<ElementInner>>>,
    children: Vec<Element>,
    tag: String,
    content: String,
    kws: HashMap<&'static str, String>,
    onetag: bool, // 是否为单标签
    pre: bool, // 是否为原文本内容
}

impl Element {
    /// 创建元素
    /// 
    /// tag: 标签名
    /// 
    /// content: 内容
    /// 
    /// ```
    /// let div = Element::new("div", "content");
    /// ```
    pub fn new(tag: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ElementInner {
                parent: None,
                children: Vec::new(),
                tag: tag.into(),
                content: escape_ascii(&content.into()),
                // 默认值
                kws: HashMap::new(),
                onetag: false,
                pre: false,
            }))
        }
    }
    /// 设置全部属性（HashMap）
    /// 
    /// ```
    /// let div = Element::new("div", "content").kws(HashMap::from([("id", "main".to_string())]));
    /// ```
    pub fn kws(self, mut kws: HashMap<&'static str, String>) -> Self {
        for (_, v) in &mut kws {
            *v = escape_ascii(v);
        }
        self.inner.borrow_mut().kws = kws;
        self
    }
    /// 设置全部属性
    /// 
    /// ```
    /// let div = Element::new("div", "content").attrs([("id", "main"), ("class", "test")]);
    /// ```
    pub fn attrs(self, attrs: &[(&'static str, &str)]) -> Self {
        let mut kws: HashMap<&str, String> = HashMap::new();
        for (k, v) in attrs {
            kws.insert(k, escape_ascii(v));
        }
        self.kws(kws)
    }
    /// 设置是否单标签
    /// 
    /// 如果是单标签，输出为字符串时将仅输出标签本身
    pub fn onetag(self, onetag: bool) -> Self {
        self.inner.borrow_mut().onetag = onetag;
        self
    }
    /// 设置是否为原文本内容
    /// 
    /// 如果为原文本内容，则内容将不会被转义
    pub fn pre(self, pre: bool) -> Self {
        {
            let mut inner = self.inner.borrow_mut();
            inner.pre = pre;
            if pre {
                inner.content = un_escape_ascii(&inner.content);
                for (_, v) in &mut inner.kws {
                    *v = un_escape_ascii(v);
                }
            }
        }
        self
    }

    /// 添加子元素
    pub fn add(&self, elem: Element) -> &Self {
        {
            let mut inner = self.inner.borrow_mut();
            elem.inner.borrow_mut().parent = Some(Rc::downgrade(&self.inner));
            inner.children.push(elem);
        }
        self
    }

    /// 添加子元素并返回Self
    pub fn add_with(self, elem: Element) -> Self {
        self.add(elem);
        self
    }

    /// 设置一个属性，不影响原有属性
    pub fn set_attr(&self, name: &'static str, value: impl Into<String>) {
        let mut inner = self.inner.borrow_mut();
        inner.kws.insert(name, escape_ascii(&value.into()));
    }

    /// 批量设置属性，不影响原有属性
    pub fn set_attrs<V>(&self, attrs: &[(&'static str, V)])
    where
        V: AsRef<str>,
    {
        for (k, v) in attrs {
            self.set_attr(k, v.as_ref());
        }
    }

    /// 获取父元素
    pub fn parent(&self) -> Option<Element> {
        self.inner.borrow()
            .parent
            .as_ref()
            .and_then(|weak| weak.upgrade())
            .map(|rc| Element { inner: rc })
    }

    /// 设置内容
    pub fn configcnt(&self, content: impl Into<String>) -> &Self {
        let mut inner = self.inner.borrow_mut();
        if inner.pre {
            inner.content = content.into();
        } else {
            inner.content = escape_ascii(&content.into());
        }
        self
    }

    /// 设置全部属性
    /// 
    /// 当`pre == true`时，内容将不会被转义
    pub fn configkws(&self, mut kws: HashMap<&'static str, String>) -> &Self {
        let mut inner = self.inner.borrow_mut();
        if !inner.pre {
            for (_, v) in &mut kws {
                *v = escape_ascii(v);
            }
        }
        inner.kws = kws;
        self
    }

    /// 获取子元素
    pub fn children(&self) -> Vec<Element> {
        self.inner.borrow().children.clone()
    }

    /// 移除指定位置子元素
    pub fn remove_child(&self, index: usize) -> Option<Element> {
        let mut inner = self.inner.borrow_mut();
        if index < inner.children.len() {
            let child = inner.children.remove(index);
            child.inner.borrow_mut().parent = None;
            Some(child)
        } else {
            None
        }
    }

    /// 删除指定子元素
    pub fn remove_child_by_ref(&self, child: &Element) -> bool {
        let mut inner = self.inner.borrow_mut();
        if let Some(index) = inner.children.iter().position(|x| x == child) {
            inner.children.remove(index);
            child.inner.borrow_mut().parent = None;
            true
        } else {
            false
        }
    }

    /// 删除所有子元素
    pub fn remove_all_children(&self) {
        let mut inner = self.inner.borrow_mut();
        for child in inner.children.drain(..) {
            child.inner.borrow_mut().parent = None;
        }
    }

    /// 渲染为html字符串
    pub fn render(&self, split_s: &str) -> String {
        let inner = self.inner.borrow();
        if inner.tag.is_empty() {
            // 空标签
            return inner.content.clone();
        }
        
        let mut htmltext = format!("<{}", inner.tag);

        // 处理属性
        for (k, v) in &inner.kws {
            htmltext.push_str(&format!(" {}=\"{}\"", k, v));
        }
        htmltext.push('>');

        htmltext.push_str(&inner.content);

        // 处理子元素
        for item in &inner.children {
            let subtext = item.render(split_s);
            htmltext.push_str(split_s);
            htmltext.push_str(&subtext);
        }

        if inner.onetag {
            // 单标签
            htmltext.push_str(split_s);
        } else if !inner.children.is_empty() {
            // 有子标签
            htmltext.push_str(split_s);
            htmltext.push_str(&format!("</{}>", inner.tag))
        } else {
            // 无子标签
            htmltext.push_str(&format!("</{}>", inner.tag))
        }

        htmltext
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Element[{:?}]", self.inner.borrow().tag)?;
        if self.inner.borrow().parent.is_some() {
            writeln!(f, "  parent: HAS")?;
        } else {
            writeln!(f, "  parent: None")?;
        }
        if !self.inner.borrow().content.is_empty() {
            writeln!(f, "  content: {:?}", self.inner.borrow().content)?;
        }
        if !self.inner.borrow().kws.is_empty() {
            writeln!(f, "  kws: {:?}", self.inner.borrow().kws)?;
        }
        if !self.inner.borrow().children.is_empty() {
            writeln!(f, "  children<{}>", self.inner.borrow().children.len())?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_file(filename: &str, content: &str) {
        let mut file = File::create(filename).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn it_works() {
        let root = Element::new("html", "");

        // 短元素用add_with()方法添加
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
        div.set_attrs(&[("id", "main"), ("class", "container<>")]);
        div.configcnt("&<html><div>content内容&");
        
        // 输出父元素此刻的html代码
        if let Some(parent) = div.parent() {
            println!("{}", parent.render("\n"));
        }

        div.add(Element::new("h1", "rusthtmlbuilder"));

        // 添加列表
        let ul = Element::new("ul", "");
        // let ul = Element::new("ol", "");
        div.add(ul.clone());
        
        for i in 0..10 {
            ul.add(Element::new("li", &i.to_string()));
        }
        
        // 删除倒数第二个li
        {
            let children_count = ul.children().len();
            if children_count >= 2 {
                ul.remove_child(children_count - 2);
            }
        }

        div.add(Element::new("", "content内容，只要标签名为空即可"));

        let result = root.render("\n");
        println!("{}", result);

        write_file("test.html", &result);
    }

    #[test]
    fn test_eq() {
        let a = Element::new("div", "");
        let b = Element::new("div", "");
        assert_ne!(a, b);

        let a = Element::new("div", "");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_debug() {
        let a = Element::new("div", "");
        println!("{:?}", a);

        let b = Element::new("div", "content");
        b.set_attr("id", "main");
        a.add(b.clone());
        println!("{:?}", a);

        let c = Element::new("ul", "");
        for i in 0..10 {
            c.add(Element::new("li", &i.to_string()));
        }
        b.add(c.clone());
        println!("{:?}", b);
        println!("{:?}", c);

        println!("{}", a.render("\n"));
    }

    #[test]
    fn test_attrs() {
        // 设置初始属性
        let a = Element::new("a", "content").attrs(&[("id", "main"), ("class", "test")]);
        println!("{:?}", a);
        // 以下更改不会影响原有属性
        a.set_attrs(&[("href", "https://www.rust-lang.org/"), ("target", "_blank")]);
        println!("{:?}", a);
        // 以下更改会修改全部，相当于自身调用一次kws()
        a.configkws(HashMap::from([
            ("href", "https://www.rust-lang.org/zh-CN/".to_string()),
            ("target", "_self".to_string()),
        ]));
        println!("{:?}", a);
    }

    #[test]
    fn test_delete() {
        let a = Element::new("div", "");
        let b = Element::new("div", "");
        let c = Element::new("div", "");
        a.add(b.clone());
        a.add(c.clone());
        assert_eq!(a.children().len(), 2);
        assert_eq!(a.remove_child(1), Some(c.clone()));
        assert_eq!(a.children().len(), 1);
        assert_eq!(a.remove_child(0), Some(b.clone()));
        assert_eq!(a.children().len(), 0);
        assert_eq!(a.remove_child(0), None);
        a.add(b.clone());
        a.add(c.clone());
        assert_eq!(a.remove_child_by_ref(&b), true);
        assert_eq!(a.remove_child_by_ref(&b), false);
        a.remove_all_children();
        assert_eq!(a.children().len(), 0);
    }
}
