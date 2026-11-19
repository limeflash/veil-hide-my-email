//! Icons from the design handoff: 24×24, stroked with `currentColor`.

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum I {
    Logo,
    Settings,
    Lock,
    Clock,
    Key,
    Shield,
    Box,
    External,
    Refresh,
    Forward,
    List,
    LogOut,
    Copy,
    Fill,
    Plus,
    Check,
    CheckDraw,
    Alert,
    Search,
    ChevronDown,
    ChevronLeft,
    Edit,
    Ban,
    Trash,
    Inbox,
    Sun,
    Moon,
    Monitor,
    Info,
    Github,
    X,
}

impl I {
    fn paths(self) -> &'static str {
        match self {
            I::Logo => r#"<path d="M5 5h14a3 3 0 0 1 2.6 1.5L12 13.2 2.4 6.5A3 3 0 0 1 5 5z"/><path d="M2 9.3l8.9 6.2a1.9 1.9 0 0 0 2.2 0L22 9.3V16a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z"/>"#,
            I::Settings => r#"<circle cx="12" cy="12" r="6.5"/><circle cx="12" cy="12" r="2.5"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M5.3 18.7l2.1-2.1M16.6 7.4l2.1-2.1"/>"#,
            I::Lock => r#"<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#,
            I::Clock => r#"<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>"#,
            I::Key => r#"<path d="m15.5 7.5 2.3 2.3a1 1 0 0 0 1.4 0l2.1-2.1a1 1 0 0 0 0-1.4L19 4"/><path d="m21 2-9.6 9.6"/><circle cx="7.5" cy="15.5" r="5.5"/>"#,
            I::Shield => r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>"#,
            I::Box => r#"<path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/>"#,
            I::External => r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#,
            I::Refresh => r#"<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>"#,
            I::Forward => r#"<path d="m15 14 5-5-5-5"/><path d="M4 20v-7a4 4 0 0 1 4-4h12"/>"#,
            I::List => r#"<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>"#,
            I::LogOut => r#"<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="m16 17 5-5-5-5"/><path d="M21 12H9"/>"#,
            I::Copy => r#"<rect x="8" y="8" width="14" height="14" rx="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
            I::Fill => r#"<path d="M5 4h1a3 3 0 0 1 3 3 3 3 0 0 1 3-3h1"/><path d="M13 20h-1a3 3 0 0 1-3-3 3 3 0 0 1-3 3H5"/><path d="M5 16H4a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2h1"/><path d="M13 8h7a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-7"/><path d="M9 7v10"/>"#,
            I::Plus => r#"<path d="M5 12h14M12 5v14"/>"#,
            I::Check => r#"<path d="M20 6 9 17l-5-5"/>"#,
            I::CheckDraw => r#"<path class="v-check-draw" d="M20 6 9 17l-5-5"/>"#,
            I::Alert => r#"<circle cx="12" cy="12" r="10"/><path d="M12 8v4M12 16h.01"/>"#,
            I::Search => r#"<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>"#,
            I::ChevronDown => r#"<path d="m6 9 6 6 6-6"/>"#,
            I::ChevronLeft => r#"<path d="m15 18-6-6 6-6"/>"#,
            I::Edit => r#"<path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>"#,
            I::Ban => r#"<circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/>"#,
            I::Trash => r#"<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M10 11v6M14 11v6"/>"#,
            I::Inbox => r#"<path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>"#,
            I::Sun => r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>"#,
            I::Moon => r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#,
            I::Monitor => r#"<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>"#,
            I::Info => r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>"#,
            I::Github => r#"<path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/>"#,
            I::X => r#"<path d="M18 6 6 18M6 6l12 12"/>"#,
        }
    }
}

/// `<svg class="v-icon …">` with the icon's shapes.
///
/// Shapes are created with `createElementNS` rather than `innerHTML`, so the
/// bundle passes Mozilla's add-on linter without an unsafe-assignment warning.
#[component]
pub fn Icon(icon: I, #[prop(optional)] class: &'static str) -> impl IntoView {
    let class = if class.is_empty() {
        "v-icon".to_string()
    } else {
        format!("v-icon {class}")
    };
    let node = NodeRef::<leptos::svg::Svg>::new();
    Effect::new(move |_| {
        if let Some(svg) = node.get() {
            if svg.child_element_count() == 0 {
                append_shapes(&svg, icon.paths());
            }
        }
    });
    view! { <svg node_ref=node class=class viewBox="0 0 24 24" aria-hidden="true"></svg> }
}

const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// Appends `<tag a="b" …/>` shapes (the only form the icon table uses).
fn append_shapes(svg: &web_sys::Element, markup: &str) {
    let doc = document();
    for chunk in markup.split('<').filter(|c| !c.trim().is_empty()) {
        let body = chunk.trim_end().trim_end_matches('>').trim_end_matches('/');
        let mut parts = body.splitn(2, char::is_whitespace);
        let tag = parts.next().unwrap_or_default();
        let Ok(el) = doc.create_element_ns(Some(SVG_NS), tag) else { continue };
        let mut rest = parts.next().unwrap_or_default();
        while let Some(eq) = rest.find("=\"") {
            let name = rest[..eq].trim();
            let after = &rest[eq + 2..];
            let Some(end) = after.find('"') else { break };
            let _ = el.set_attribute(name, &after[..end]);
            rest = &after[end + 1..];
        }
        let _ = svg.append_child(&el);
    }
}
