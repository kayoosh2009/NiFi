mod ui;
use macroquad::prelude::*;
use std::collections::HashMap;

#[macroquad::main("Nifi")]
async fn main() {
    loop {
        clear_background(WHITE);
        // файл перечитывается каждый кадр: изменил и сохранил, окно обновилось
        match std::fs::read_to_string("ui/main.nifi")
            .map_err(|e| e.to_string())
            .and_then(|s| ui::parse(&s))
        {
            Ok(nodes) => draw(&nodes),
            Err(e) => { draw_text(&e, 10., 30., 24., RED); }
        }
        next_frame().await;
    }
}

fn anchor(a: &str) -> (f32, f32) {
    match a {
        "top-left" => (0., 0.),   "top" => (0.5, 0.),    "top-right" => (1., 0.),
        "left" => (0., 0.5),      "center" => (0.5, 0.5), "right" => (1., 0.5),
        "bottom-left" => (0., 1.), "bottom" => (0.5, 1.), "bottom-right" => (1., 1.),
        _ => (0., 0.),
    }
}

fn draw(nodes: &[ui::Node]) {
    let styles: HashMap<&str, &ui::Node> = nodes.iter()
        .filter(|n| n.kind == "style")
        .map(|n| (n.name.as_str(), n))
        .collect();

    for o in nodes.iter().filter(|n| n.kind == "object") {
        let style = o.props.get("style").and_then(|s| styles.get(s.as_str()));
        let loc = o.children.iter().find(|c| c.kind == "location");
        let num = |k: &str, d: f32| loc.and_then(|l| l.props.get(k)).and_then(|v| v.parse().ok()).unwrap_or(d);

        let (w, h, m) = (num("width", 100.), num("height", 40.), num("margin", 0.));
        let (x, y) = match loc.and_then(|l| l.props.get("align")) {
            Some(a) => {
                let (fx, fy) = anchor(a);
                (m + (screen_width() - w - 2. * m) * fx, m + (screen_height() - h - 2. * m) * fy)
            }
            None => (num("set-x", 0.), num("set-y", 0.)),
        };

        let color = |k: &str| style
            .and_then(|s| s.props.get(k))
            .and_then(|v| u32::from_str_radix(v.trim_start_matches('#'), 16).ok())
            .map(Color::from_hex);

        if let Some(c) = color("background-color") { draw_rectangle(x, y, w, h, c); }
        if let Some(c) = color("stroke-color") { draw_rectangle_lines(x, y, w, h, 2., c); }

        if o.name == "text" {
            if let Some(t) = o.children.iter().find(|c| c.kind == "text").and_then(|c| c.text.as_ref()) {
                draw_text(t, x, y + 24., 24., BLACK);
            }
        }
    }
}