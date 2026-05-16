//! vfox CLI 文本输出解析器
//!
//! 与 vfox CLI 输出格式耦合的唯一模块。vfox 升级若改了输出，
//! 只需要修改这里。所有 fixture 来自 vfox 1.0.8 在
//! `NO_COLOR=1` 下的实际输出。

use crate::error::{AppError, AppResult};
use crate::vfox::types::{AvailablePlugin, CurrentVersion, PluginInfo, SdkVersions, SearchVersion};

/// 解析 `vfox current` 输出
///
/// 形如：
/// ```text
/// java -> v8.0.482-albba
/// maven -> v3.9.15
/// rust -> v1.90.0
/// ```
///
/// 当 vfox 没有任何已激活 SDK 时输出为空，返回空 Vec 而非错误。
pub fn parse_current(text: &str) -> Vec<CurrentVersion> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        // 容错：跳过提示文本
        if !line.contains("->") {
            continue;
        }
        if let Some((sdk, ver)) = line.split_once("->") {
            let sdk = sdk.trim().to_string();
            let mut version = ver.trim().to_string();
            // vfox 输出版本带前缀 'v'，业务层统一去掉再展示更干净
            if let Some(stripped) = version.strip_prefix('v') {
                version = stripped.to_string();
            }
            if !sdk.is_empty() && !version.is_empty() {
                out.push(CurrentVersion { sdk, version });
            }
        }
    }
    out
}

/// 解析 `vfox list` 输出（树形）
///
/// 形如：
/// ```text
/// All installed sdk versions
/// ├─┬java
/// │ ├──v8.0.482-albba
/// │ └──v21.0.2+13
/// ├─┬maven
/// │ ├──v4.0.0-rc-5
/// │ └──v3.9.15
/// └─┬rust
///   └──v1.90.0
/// ```
///
/// 策略：扫每行，找最后一个 box-drawing 字符，根据其类型判定行类型。
/// - 最后一个为 `┬` 的行 → SDK 头，名字是后续内容
/// - 最后一个为 `─` 的行 → 版本号
pub fn parse_list(text: &str) -> Vec<SdkVersions> {
    let mut result: Vec<SdkVersions> = Vec::new();
    let mut current_sdk: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            continue;
        }
        // 跳过纯文本表头
        if line.contains("installed sdk versions") || line.contains("No SDK installed") {
            continue;
        }

        // 找到最后一个 box-drawing 字符
        let (kind, rest) = match split_after_tree(line) {
            Some(v) => v,
            None => continue,
        };

        let rest = rest.trim();
        if rest.is_empty() {
            continue;
        }

        match kind {
            TreeKind::Branch => {
                // 新 SDK
                let sdk_name = rest.to_string();
                current_sdk = Some(sdk_name.clone());
                result.push(SdkVersions {
                    sdk: sdk_name,
                    versions: Vec::new(),
                });
            }
            TreeKind::Leaf => {
                // 版本号
                if let Some(ref name) = current_sdk {
                    let mut ver = rest.to_string();
                    if let Some(stripped) = ver.strip_prefix('v') {
                        ver = stripped.to_string();
                    }
                    if let Some(entry) = result.iter_mut().find(|e| &e.sdk == name) {
                        entry.versions.push(ver);
                    }
                }
            }
        }
    }
    result
}

#[derive(Debug, PartialEq, Eq)]
enum TreeKind {
    Branch, // 以 ┬ 结尾，意味着下一行起是子节点 → 这是 SDK 名
    Leaf,   // 以 ── 结尾 → 这是版本
}

/// 找到行首的 box-drawing 前缀，根据最后一个非空格 box char 判断类型，
/// 并返回前缀之后的剩余文本。识别失败返回 None（如纯文本表头）。
fn split_after_tree(line: &str) -> Option<(TreeKind, &str)> {
    // box-drawing 字符集合
    let is_tree = |c: char| matches!(c, '├' | '─' | '┬' | '│' | '└' | ' ' | '\t');

    let prefix_len: usize = line
        .char_indices()
        .take_while(|(_, c)| is_tree(*c))
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    if prefix_len == 0 {
        return None;
    }
    let prefix = &line[..prefix_len];
    let rest = &line[prefix_len..];

    // 取 prefix 中最后一个 box 字符（忽略空白）
    let last_box = prefix.chars().rev().find(|c| !c.is_whitespace())?;
    let kind = match last_box {
        '┬' => TreeKind::Branch,
        '─' => TreeKind::Leaf,
        _ => return None,
    };
    Some((kind, rest))
}

/// 解析 `vfox available` 输出
///
/// 形如：
/// ```text
/// AVAILABLE PLUGINS
///
///   bun                 ✗  https://github.com/arfaWong/vfox-bun
///   clang               ✓  https://github.com/version-fox/vfox-clang
/// ```
///
/// 注意：`✓` 是"version-fox 官方维护"标记，**不是**"已安装"。
/// 真实是否已安装由调用方扫 `~/.vfox/plugin/` 后回填。
/// 这里把 ✓/✗ 解析到 `official` 字段，`installed` 留 `false` 占位。
pub fn parse_available(text: &str) -> Vec<AvailablePlugin> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("AVAILABLE PLUGINS") {
            continue;
        }

        // 关键标志：✓ / ✗
        let official = if line.contains('✓') {
            true
        } else if line.contains('✗') {
            false
        } else {
            continue;
        };

        // 按空白分词，去掉勾叉，剩下 name + url
        let tokens: Vec<&str> = line
            .split_whitespace()
            .filter(|t| *t != "✓" && *t != "✗")
            .collect();
        if tokens.len() < 2 {
            continue;
        }
        let name = tokens[0].to_string();
        let homepage = tokens[tokens.len() - 1].to_string();
        if name.is_empty() || !homepage.starts_with("http") {
            continue;
        }
        out.push(AvailablePlugin {
            name,
            official,
            installed: false, // 占位；由调用方扫 plugin/ 回填
            homepage,
        });
    }
    out
}

/// 解析 `vfox info <plugin>` 输出
///
/// 形如：
/// ```text
/// Plugin Info:
/// Name     -> java
/// Version  -> 0.5.3
/// Homepage -> https://github.com/version-fox/vfox-java
/// Desc     ->
/// Support for multiple JDK distributions, ...
/// Notes:
/// ======
///    Listed below are the supported distributions ...
/// ```
pub fn parse_info(text: &str) -> AppResult<PluginInfo> {
    let mut info = PluginInfo::default();
    let mut lines = text.lines().peekable();

    // 跳过表头 "Plugin Info:"
    while let Some(l) = lines.peek() {
        if l.trim().starts_with("Plugin Info") {
            lines.next();
            break;
        }
        // 没有表头也能继续
        if l.contains("->") {
            break;
        }
        lines.next();
    }

    // 解析 key -> value 区块；Desc 可能是空 value + 下一行内容
    let mut desc_pending = false;
    let mut in_notes = false;
    let mut notes_buf = String::new();

    for raw in lines {
        let line = raw.trim_end();

        if in_notes {
            // 跳过分隔符 ======
            if line.trim().chars().all(|c| c == '=') && !line.is_empty() {
                continue;
            }
            if !notes_buf.is_empty() {
                notes_buf.push('\n');
            }
            notes_buf.push_str(line);
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("Notes") {
            in_notes = true;
            continue;
        }

        if let Some((k, v)) = trimmed.split_once("->") {
            desc_pending = false;
            let key = k.trim();
            let val = v.trim();
            match key {
                "Name" => info.name = val.to_string(),
                "Version" => info.version = val.to_string(),
                "Homepage" => info.homepage = val.to_string(),
                "Desc" => {
                    if val.is_empty() {
                        desc_pending = true;
                    } else {
                        info.description = val.to_string();
                    }
                }
                _ => {}
            }
        } else if desc_pending {
            // Desc 紧跟一行作为正文
            info.description = trimmed.to_string();
            desc_pending = false;
        }
    }

    info.notes = notes_buf.trim_end().to_string();
    info.distributions = extract_distributions(&info.notes);
    if info.name.is_empty() {
        return Err(AppError::ParseError("info 输出缺少 Name 字段".into()));
    }
    Ok(info)
}

/// 从 `vfox info <plugin>` 的 Notes 字段里启发式抽取"可用筛选词"
///
/// 约定的写法形如：
/// ```text
///  - Oracle:     x.y.z-oracle
///  - OpenJDK:    x.y.z-open
///  - GraalVM:    x.y.z-graal
/// ```
/// 抽 `x.y.z-(\w+)` 的尾部小写词作为候选筛选词。
/// 多次出现去重，保持首次出现的顺序。
fn extract_distributions(notes: &str) -> Vec<String> {
    use regex::Regex;
    use std::sync::OnceLock;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        // 匹配 "x.y.z-<word>" 或 "<digits>.<digits>-<word>"
        Regex::new(r"(?:x\.y\.z|\d+\.\d+(?:\.\d+)?)-([a-z][a-z0-9]*)").expect("regex")
    });

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for cap in re.captures_iter(notes) {
        if let Some(m) = cap.get(1) {
            let s = m.as_str().to_string();
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    out
}

/// 解析 `vfox search <sdk>` 在 non-TTY 下的输出
///
/// 形如：
/// ```text
/// Available versions:
///  - 26.0.1+8
///  - 25.0.2+10 (LTS)
///  - 21.0.2+13 (LTS) (installed)
/// ```
///
/// 当 stdin 不是 TTY（我们的 runner 已强制 `Stdio::null()`）时，
/// vfox 自动 fallback 到这个纯文本模式，无需 TUI 处理。
pub fn parse_search(text: &str) -> Vec<SearchVersion> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        // 形如 "- 21.0.2+13 (LTS) (installed)"
        let body = match line.strip_prefix("- ") {
            Some(b) => b.trim(),
            None => continue,
        };
        // 第一个空白前是版本号，后面是 (LTS) / (installed) 等标签
        let mut parts = body.splitn(2, char::is_whitespace);
        let version = match parts.next() {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => continue,
        };
        let tail = parts.next().unwrap_or("");
        out.push(SearchVersion {
            version,
            is_lts: tail.contains("(LTS)"),
            is_installed: tail.contains("(installed)"),
        });
    }
    out
}

// ----------------------------------------------------------------------------
// 单元测试
// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_FIXTURE: &str = "\
java -> v8.0.482-albba
maven -> v3.9.15
rust -> v1.90.0
";

    const LIST_FIXTURE: &str = "\
All installed sdk versions
├─┬java
│ ├──v8.0.482-albba
│ └──v21.0.2+13
├─┬maven
│ ├──v4.0.0-rc-5
│ └──v3.9.15
└─┬rust
  └──v1.90.0
";

    const AVAILABLE_FIXTURE: &str = "\
AVAILABLE PLUGINS

  bun                 ✗  https://github.com/arfaWong/vfox-bun
  clang               ✓  https://github.com/version-fox/vfox-clang
  cmake               ✓  https://github.com/version-fox/vfox-cmake
  crystal             ✗  https://github.com/yanecc/vfox-crystal
";

    const INFO_FIXTURE: &str = "\
Plugin Info:
Name     -> java
Version  -> 0.5.3
Homepage -> https://github.com/version-fox/vfox-java
Desc     ->
Support for multiple JDK distributions, such as: Oracle, Graalvm, Eclipse & more.
Notes:
======
   Listed below are the supported distributions and their short names.
    - Oracle:     x.y.z-oracle
    - OpenJDK:    x.y.z-open
";

    const SEARCH_FIXTURE: &str = "\
Available versions:
 - 26.0.1+8
 - 25.0.2+10 (LTS)
 - 25+36 (LTS)
 - 21.0.2+13 (LTS) (installed)
 - 8.0.482-albba (installed)
";

    #[test]
    fn current_basic() {
        let r = parse_current(CURRENT_FIXTURE);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].sdk, "java");
        assert_eq!(r[0].version, "8.0.482-albba");
        assert_eq!(r[2].sdk, "rust");
        assert_eq!(r[2].version, "1.90.0");
    }

    #[test]
    fn current_empty() {
        assert!(parse_current("").is_empty());
        assert!(parse_current("\n  \n").is_empty());
    }

    #[test]
    fn list_basic() {
        let r = parse_list(LIST_FIXTURE);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].sdk, "java");
        assert_eq!(r[0].versions, vec!["8.0.482-albba", "21.0.2+13"]);
        assert_eq!(r[1].sdk, "maven");
        assert_eq!(r[1].versions.len(), 2);
        assert_eq!(r[2].sdk, "rust");
        assert_eq!(r[2].versions, vec!["1.90.0"]);
    }

    #[test]
    fn available_basic() {
        let r = parse_available(AVAILABLE_FIXTURE);
        assert_eq!(r.len(), 4);
        assert_eq!(r[0].name, "bun");
        // ✗ = 第三方，official = false
        assert!(!r[0].official);
        // 解析阶段 installed 都是占位 false
        assert!(!r[0].installed);

        // clang ✓ = 官方
        assert!(r[1].official);
        assert_eq!(r[1].name, "clang");
        assert!(r[1].homepage.starts_with("https://"));
    }

    #[test]
    fn info_basic() {
        let info = parse_info(INFO_FIXTURE).expect("info parse ok");
        assert_eq!(info.name, "java");
        assert_eq!(info.version, "0.5.3");
        assert_eq!(info.homepage, "https://github.com/version-fox/vfox-java");
        assert!(info.description.contains("Oracle"));
        assert!(info.notes.contains("Temurin").not() || info.notes.contains("Oracle"));
        // java notes 里应该抽到 oracle 和 open 两个 distributions
        assert!(info.distributions.contains(&"oracle".to_string()));
        assert!(info.distributions.contains(&"open".to_string()));
    }

    #[test]
    fn info_no_distributions() {
        // rust / maven 等没有供应商概念的插件，notes 不含 x.y.z-xxx 模式
        let text = "\
Plugin Info:
Name     -> rust
Version  -> 1.0.0
Homepage -> https://github.com/XZzYassin/vfox-rust
Desc     ->
Rust plugin https://releases.rs/docs/
";
        let info = parse_info(text).expect("parse ok");
        assert!(info.distributions.is_empty());
    }

    #[test]
    fn search_basic() {
        let r = parse_search(SEARCH_FIXTURE);
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].version, "26.0.1+8");
        assert!(!r[0].is_lts);
        assert!(!r[0].is_installed);

        assert_eq!(r[1].version, "25.0.2+10");
        assert!(r[1].is_lts);
        assert!(!r[1].is_installed);

        assert_eq!(r[3].version, "21.0.2+13");
        assert!(r[3].is_lts);
        assert!(r[3].is_installed);

        // 含 `-` 的版本号（如 java legacy 命名）
        assert_eq!(r[4].version, "8.0.482-albba");
        assert!(!r[4].is_lts);
        assert!(r[4].is_installed);
    }

    // 给 bool 加个工具方法，让上面的断言更明确
    trait BoolExt {
        fn not(self) -> bool;
    }
    impl BoolExt for bool {
        fn not(self) -> bool {
            !self
        }
    }
}
