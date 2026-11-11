//! Drawing a machine from its `show` view (the JSON every host already has): the bridge from
//! the output schema to `delta-render`, used by `deltac`, the Pad and the docs site.

use delta_render::{Edge, Graph, Node, Theme};
use serde_json::Value as Json;
// `BTreeMap` is a map kept sorted by key; here it groups parallel arrows in a stable order.
use std::collections::BTreeMap;

/// The graph of a `show`/`save` value of type `machine`; None for any other value.
pub fn graph_of(value: &Json) -> Option<Graph> {
    if value["type"] != "machine" {
        return None;
    }
    let text = |v: &Json| {
        v.as_str()
            .map(str::to_string)
            .unwrap_or_else(|| v.to_string())
    };
    let names: Vec<String> = value["states"].as_array()?.iter().map(text).collect();
    let index = |v: &Json| names.iter().position(|n| *n == text(v));
    let start = index(&value["start"]);
    let accept: Vec<usize> = value["accept"]
        .as_array()
        .map(|a| a.iter().filter_map(index).collect())
        .unwrap_or_default();
    // Parallel arrows become one arrow: "0,1" for one-symbol labels, one line each otherwise.
    let mut merged: BTreeMap<(usize, usize), Vec<String>> = BTreeMap::new();
    for e in value["edges"].as_array().into_iter().flatten() {
        let (Some(from), Some(to)) = (index(&e[0]), index(&e[2])) else {
            continue;
        };
        merged.entry((from, to)).or_default().push(text(&e[1]));
    }
    let edges = merged
        .into_iter()
        .map(|((from, to), labels)| {
            let short = labels.iter().all(|l| !l.contains([',', ' ', '\n']));
            Edge {
                from,
                to,
                label: labels.join(if short { "," } else { "\n" }),
            }
        })
        .collect();
    let nodes = names
        .iter()
        .enumerate()
        .map(|(i, name)| Node {
            name: name.clone(),
            start: start == Some(i),
            accept: accept.contains(&i),
        })
        .collect();
    let title = value["model"].as_str().unwrap_or("machine").to_string();
    Some(Graph {
        title,
        nodes,
        edges,
    })
}

/// The Pad's diagram geometry for a `MachineView` (states with `name`/`start`/`accept`, edges
/// already merged with `from`/`to`/`label`): `delta-render`'s layout as plain JSON, so the Pad
/// draws exactly the picture a download has, with its own highlighting on top.
pub fn layout_json(view: &Json) -> Option<Json> {
    let states = view["states"].as_array()?;
    let nodes = states
        .iter()
        .map(|s| Node {
            name: s["name"].as_str().unwrap_or("").to_string(),
            start: s["start"] == true,
            accept: s["accept"] == true,
        })
        .collect();
    let edges = view["edges"]
        .as_array()?
        .iter()
        .map(|e| Edge {
            from: e["from"].as_u64().unwrap_or(0) as usize,
            to: e["to"].as_u64().unwrap_or(0) as usize,
            label: e["label"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    let g = Graph {
        title: view["kind"].as_str().unwrap_or("machine").to_string(),
        nodes,
        edges,
    };
    let l = delta_render::layout(&g);
    let pt = |p: delta_render::Point| serde_json::json!({ "x": p.x, "y": p.y });
    let curve = |c: &delta_render::Curve| serde_json::json!({ "path": delta_render::arrow_path(c), "tip": pt(c.end()), "base": pt(c.before_end()) });
    Some(serde_json::json!({
        "width": l.width,
        "height": l.height,
        "nodes": l.nodes.iter().map(|n| serde_json::json!({ "x": n.at.x, "y": n.at.y, "rx": n.rx, "ry": n.ry })).collect::<Vec<_>>(),
        "edges": l.edges.iter().map(|e| {
            let mut c = curve(&e.curve);
            c["label"] = serde_json::json!({ "x": e.label_at.x, "y": e.label_at.y, "w": e.label_w, "h": e.label_h });
            c
        }).collect::<Vec<_>>(),
        "start": l.start.as_ref().map(curve),
    }))
}

/// The machine as an SVG document in the light theme, or None if the value isn't a machine.
pub fn svg_of(value: &Json, dark: bool) -> Option<String> {
    let g = graph_of(value)?;
    let theme = if dark { Theme::dark() } else { Theme::light() };
    Some(delta_render::svg(&g, &delta_render::layout(&g), &theme))
}

/// The machine as a TikZ picture.
pub fn tikz_of(value: &Json) -> Option<String> {
    let g = graph_of(value)?;
    Some(delta_render::tikz(&g, &delta_render::layout(&g)))
}

/// The machine as plain text (for terminals without images).
pub fn text_of(value: &Json) -> Option<String> {
    graph_of(value).map(|g| delta_render::text(&g))
}

/// A value's text: its `text` field, else its scalar `value`, else compact JSON.
pub fn value_text(v: &Json) -> String {
    if let Some(t) = v["text"].as_str() {
        return t.to_string();
    }
    match &v["value"] {
        Json::Null => v.to_string(),
        Json::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The contents of a text file for a `save` (`svg`, `tex`/`tikz`, `txt`, `json`): the same in
/// `deltac` and the Pad. `text` is the save output's `print` text, used for `.txt`. An `svg`
/// file carries the "Delta by Nandish" strip; PNG and PDF are drawn from that same SVG.
pub fn text_file(
    format: &str,
    value: &Json,
    text: Option<&str>,
    dark: bool,
) -> Result<String, String> {
    let not_drawable = || "only machines can be drawn".to_string();
    match format {
        "svg" => svg_of(value, dark)
            .map(|s| delta_render::watermark(&s))
            .ok_or_else(not_drawable),
        "tex" | "tikz" => tikz_of(value).ok_or_else(not_drawable),
        "txt" => Ok(format!(
            "{}\n",
            text.map(str::to_string)
                .or_else(|| text_of(value))
                .unwrap_or_else(|| value_text(value))
        )),
        "json" => serde_json::to_string_pretty(value)
            .map(|s| s + "\n")
            .map_err(|e| e.to_string()),
        other => Err(format!("unknown format `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shown_dfa_becomes_a_graph_with_merged_labels() {
        let out = crate::run(
            "dfa M {\n start q0; accept q1\n q0 -0-> q0; q0 -1-> q1\n q1 -0,1-> q1\n}\nshow M",
        );
        let g = graph_of(&out["outputs"][0]["value"]).unwrap_or_default();
        assert_eq!(g.nodes.len(), 2, "{out}");
        assert!(g.nodes[0].start && g.nodes[1].accept);
        let labels: Vec<&str> = g.edges.iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels, ["0", "1", "0,1"]);
        assert!(svg_of(&out["outputs"][0]["value"], false).is_some_and(|s| s.contains("<circle")));
        assert!(graph_of(&serde_json::json!({ "type": "set" })).is_none());
    }

    #[test]
    fn the_pad_layout_has_one_shape_per_state_and_arrow() {
        let view = serde_json::json!({
            "kind": "dfa",
            "states": [{ "name": "q0", "start": true, "accept": false }, { "name": "q1", "start": false, "accept": true }],
            "edges": [{ "from": 0, "to": 1, "label": "a" }, { "from": 1, "to": 1, "label": "0 → R\ny → R" }],
        });
        let l = layout_json(&view).unwrap_or_default();
        assert_eq!(l["nodes"].as_array().map(Vec::len), Some(2));
        assert_eq!(l["edges"].as_array().map(Vec::len), Some(2));
        assert!(
            l["start"]["path"]
                .as_str()
                .is_some_and(|p| p.starts_with('M'))
        );
        // The two-line label gets a taller box than the one-line one.
        assert!(l["edges"][1]["label"]["h"].as_f64() > l["edges"][0]["label"]["h"].as_f64());
        assert!(layout_json(&serde_json::json!({})).is_none());
    }
}
