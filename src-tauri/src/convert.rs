//! 巡检报告格式转换工具
//! 内置巡检脚本只产出 HTML 报告，导出 TXT / Markdown 时在此做轻量转换。
//! 不依赖第三方 HTML 解析库，采用字符扫描实现，足够覆盖本项目的报告结构。

/// 解码常见 HTML 实体
pub fn decode_entity(ent: &str) -> String {
    let e = ent.trim_start_matches('&').trim_end_matches(';');
    if let Some(hex) = e.strip_prefix('#') {
        if let Some(h) = hex.strip_prefix('x').or_else(|| hex.strip_prefix('X')) {
            if let Ok(cp) = u32::from_str_radix(h, 16) {
                if let Some(ch) = char::from_u32(cp) {
                    return ch.to_string();
                }
            }
        } else if let Ok(cp) = e.parse::<u32>() {
            if let Some(ch) = char::from_u32(cp) {
                return ch.to_string();
            }
        }
    }
    let s = match e {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        "copy" => "©",
        "reg" => "®",
        "times" => "×",
        "middot" => "·",
        "mdash" => "—",
        "ndash" => "–",
        _ => ent,
    };
    s.to_string()
}

/// 移除所有 HTML 标签并解码实体，块级标签替换为换行
pub fn html_to_text(html: &str) -> String {
    let block = [
        "p", "div", "br", "tr", "li", "h1", "h2", "h3", "h4", "h5", "h6", "table", "ul", "ol",
        "section", "article", "thead", "tbody", "font",
    ];
    let mut out = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();
    let mut skip_depth: i32 = 0; // 用于跳过 <style>/<script>
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '>' {
                    chars.next();
                    break;
                }
                tag.push(nc);
                chars.next();
            }
            let lower = tag.to_lowercase();
            let name = lower.split_whitespace().next().unwrap_or("").to_string();
            let is_close = name.starts_with('/');
            let bare = name.trim_start_matches('/');
            if bare == "style" || bare == "script" {
                if is_close {
                    skip_depth = skip_depth.saturating_sub(1);
                } else {
                    skip_depth += 1;
                }
                continue;
            }
            if skip_depth > 0 {
                continue;
            }
            if !is_close && block.contains(&bare) {
                out.push('\n');
            }
        } else if c == '&' {
            let mut ent = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == ';' {
                    chars.next();
                    break;
                }
                ent.push(nc);
                chars.next();
            }
            out.push_str(&decode_entity(&ent));
        } else if skip_depth == 0 {
            out.push(c);
        }
    }
    // 折叠多余空行
    let mut result = String::new();
    let mut last_blank = false;
    for ch in out.chars() {
        if ch == '\n' {
            if last_blank {
                continue;
            }
            last_blank = true;
            result.push('\n');
        } else {
            last_blank = false;
            result.push(ch);
        }
    }
    result.trim().to_string()
}

/// 提取标签之间的纯文本（用于表格单元格、标题等）
fn inner_text(fragment: &str) -> String {
    html_to_text(fragment)
}

/// 将 HTML 报告转换为 Markdown（处理标题、表格、加粗、段落）
pub fn html_to_markdown(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let bytes = html;
    let mut i = 0;
    let n = bytes.len();
    let mut skip_depth: i32 = 0;

    while i < n {
        if bytes[i..].starts_with('<') {
            // 读取完整标签
            let end = bytes[i..].find('>').unwrap_or(bytes[i..].len() - 1) + i;
            let tag = &bytes[i + 1..end];
            let lower = tag.to_lowercase();
            let name = lower.split_whitespace().next().unwrap_or("").to_string();
            let is_close = name.starts_with('/');
            let bare = name.trim_start_matches('/');

            if bare == "style" || bare == "script" {
                if is_close {
                    skip_depth = skip_depth.saturating_sub(1);
                } else {
                    skip_depth += 1;
                }
                i = end + 1;
                continue;
            }
            if skip_depth > 0 {
                i = end + 1;
                continue;
            }

            match bare {
                "h1" if !is_close => {
                    out.push_str("\n# ");
                }
                "h2" if !is_close => {
                    out.push_str("\n## ");
                }
                "h3" if !is_close => {
                    out.push_str("\n### ");
                }
                "h4" if !is_close => {
                    out.push_str("\n#### ");
                }
                "table" if !is_close => {
                    if let Some(close) = bytes[i..].find("</table>") {
                        let table_html = &bytes[i..i + close + "</table>".len()];
                        out.push_str("\n\n");
                        out.push_str(&table_to_markdown(table_html));
                        out.push_str("\n\n");
                        i = i + close + "</table>".len();
                        continue;
                    }
                }
                "b" | "strong" if !is_close => out.push_str("**"),
                "b" | "strong" if is_close => out.push_str("**"),
                "br" => out.push('\n'),
                "p" | "div" | "li" | "tr" | "ul" | "ol" | "section" if !is_close => {
                    out.push('\n')
                }
                _ => {}
            }
            i = end + 1;
        } else if bytes[i..].starts_with('&') {
            let end = bytes[i..].find(';').unwrap_or(0) + i;
            if end > i {
                let ent = &bytes[i..=end];
                out.push_str(&decode_entity(ent));
                i = end + 1;
            } else {
                out.push(bytes.as_bytes()[i] as char);
                i += 1;
            }
        } else {
            // 跳过标签内部的裸文本？这里 i 指向 '<' 之外，取一个字符
            let ch = bytes[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    let collapsed: Vec<&str> = out.split('\n').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    collapsed.join("\n")
}

/// 将一段 <table>...</table> 转换为 Markdown 表格
fn table_to_markdown(table_html: &str) -> String {
    // 收集所有行
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rest = table_html;
    while let Some(tr_start) = rest.find("<tr") {
        let tr_end = match rest[tr_start..].find("</tr>") {
            Some(p) => tr_start + p + "</tr>".len(),
            None => break,
        };
        let row_html = &rest[tr_start..tr_end];
        let mut cells: Vec<String> = Vec::new();
        let mut r = row_html;
        while let Some(td_start) = find_cell_start(r) {
            let (cell_tag, cell_end) = td_start;
            let close_tag = format!("</{}>", cell_tag);
            if let Some(p) = r[cell_end..].find(&close_tag) {
                let cell_html = &r[cell_end..cell_end + p];
                cells.push(inner_text(cell_html).replace('|', "\\|").replace('\n', " "));
                r = &r[cell_end + p + close_tag.len()..];
            } else {
                break;
            }
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
        rest = &rest[tr_end..];
    }
    if rows.is_empty() {
        return String::new();
    }
    let mut md = String::new();
    let cols = rows[0].len();
    // 表头
    md.push_str("| ");
    md.push_str(&rows[0].join(" | "));
    md.push_str(" |\n");
    md.push_str("| ");
    md.push_str(&vec!["---"; cols].join(" | "));
    md.push_str(" |\n");
    for row in rows.iter().skip(1) {
        md.push_str("| ");
        md.push_str(&row.join(" | "));
        md.push_str(" |\n");
    }
    md
}

/// 查找下一个单元格（td / th）起始，返回 (标签名, 内容起始索引)
fn find_cell_start(s: &str) -> Option<(&'static str, usize)> {
    let td = s.find("<td");
    let th = s.find("<th");
    match (td, th) {
        (Some(a), Some(b)) => {
            if a < b {
                Some(("td", a + "<td".len()))
            } else {
                Some(("th", a + "<th".len()))
            }
        }
        (Some(a), None) => Some(("td", a + "<td".len())),
        (None, Some(b)) => Some(("th", b + "<th".len())),
        (None, None) => None,
    }
}
