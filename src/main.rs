mod ui;
use macroquad::prelude::*;
use std::collections::HashMap;
use ui::Value;

// id объекта -> имя параметра -> значение
type Params = HashMap<String, HashMap<String, Value>>;

// id кнопки -> текущий масштаб (для анимации)
type Anim = HashMap<String, f32>;
type Fonts = HashMap<String, Font>;

const NIFI_DIR: &str = "ui/";

/// "fonts/x.otf" -> "ui/fonts/x.otf". Абсолютный путь (с /) не трогаем.
fn resolve(path: &str) -> String {
    if path.starts_with('/') { path.to_string() } else { format!("{NIFI_DIR}{path}") }
}

type Textures = HashMap<String, Texture2D>;

async fn get_texture<'a>(cache: &'a mut Textures, path: &str) -> Option<&'a Texture2D> {
    if !cache.contains_key(path) {
        match load_texture(path).await {
            Ok(t) => { cache.insert(path.to_string(), t); }
            Err(e) => {
                eprintln!("не удалось загрузить картинку {path}: {e}");
                return None;
            }
        }
    }
    cache.get(path)
}

async fn get_font<'a>(cache: &'a mut Fonts, path: &str) -> Option<&'a Font> {
    if !cache.contains_key(path) {
        match load_ttf_font(path).await {
            Ok(f) => { cache.insert(path.to_string(), f); }
            Err(e) => {
                eprintln!("не удалось загрузить шрифт {path}: {e}");
                return None;
            }
        }
    }
    cache.get(path)
}

fn set(p: &mut Params, id: &str, name: &str, v: Value) {
    p.entry(id.into()).or_default().insert(name.into(), v);
}

#[macroquad::main("Nifi")]
async fn main() {
    let mut params = Params::new();
    let mut anim = Anim::new();
    let mut events: Vec<String> = vec![];
    let mut fonts = Fonts::new();
    let mut n = 0;
    set(&mut params, "theWonNum", "number", Value::Int(0));
    loop {
        if is_key_pressed(KeyCode::Space) {
            n += 1;
            set(&mut params, "theWonNum", "number", Value::Int(n));
        }
        clear_background(WHITE);
        // файл перечитывается каждый кадр: изменил и сохранил, окно обновилось
        match std::fs::read_to_string("ui/main.nifi")
            .map_err(|e| e.to_string())
            .and_then(|s| ui::parse(&s))
        {
            Ok(nodes) => draw(&nodes, &params, &mut anim, &mut events, &mut fonts).await,
            Err(e) => { draw_text(&e, 10., 30., 24., RED); }
        }
        // сюда приходят клики по кнопкам (id кнопки)
        for id in events.drain(..) {
            if id == "play_button" {
                n += 1;
                set(&mut params, "theWonNum", "number", Value::Int(n));
            }
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

fn draw_styled(t: &str, x: f32, y: f32, size: f32, color: Color, stroke: Option<Color>, font: Option<&Font>) {
    let params = TextParams { font, font_size: size as u16, color, ..Default::default() };
    if let Some(s) = stroke {
        for (dx, dy) in [(-1., 0.), (1., 0.), (0., -1.), (0., 1.), (-1., -1.), (1., 1.), (-1., 1.), (1., -1.)] {
            draw_text_ex(t, x + dx, y + dy, TextParams { color: s, ..params.clone() });
        }
    }
    draw_text_ex(t, x, y, params);
}

async fn draw(nodes: &[ui::Node], params: &Params, anim: &mut Anim, events: &mut Vec<String>, fonts: &mut Fonts) {
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

        let (mut x, mut y, mut w, mut h) = (x, y, w, h);
        if o.name == "button" {
            let id = o.props.get("id").cloned().unwrap_or_default();
            let (mx, my) = mouse_position();
            let hover = mx >= x && mx <= x + w && my >= y && my <= y + h;
            let down = hover && is_mouse_button_down(MouseButton::Left);
            if hover && is_mouse_button_released(MouseButton::Left) {
                events.push(id.clone());
            }
            let target = if down { 0.93 } else if hover { 1.06 } else { 1.0 };
            let s = anim.entry(id).or_insert(1.0);
            *s += (target - *s) * (12. * get_frame_time()).min(1.);
            x += w * (1. - *s) / 2.;
            y += h * (1. - *s) / 2.;
            w *= *s;
            h *= *s;
        }

        let color = |k: &str| style
            .and_then(|s| s.props.get(k))
            .and_then(|v| u32::from_str_radix(v.trim_start_matches('#'), 16).ok())
            .map(Color::from_hex);

        if let Some(c) = color("background-color") { draw_rectangle(x, y, w, h, c); }
        if let Some(c) = color("stroke-color") { draw_rectangle_lines(x, y, w, h, 2., c); }

        if o.name == "text" || o.name == "button" {
            if let Some(t) = o.children.iter().find(|c| c.kind == "text").and_then(|c| c.text.as_ref()) {
                let id = o.props.get("id").map(|s| s.as_str()).unwrap_or("");
                let text = ui::fill(t, |name| params.get(id)?.get(name).cloned());

                let size: f32 = style.and_then(|s| s.props.get("text-size")).and_then(|v| v.parse().ok()).unwrap_or(24.);
                let tcolor = color("text-color").unwrap_or(BLACK);
                let stroke = color("text-stroke-color");
                let font = match style.and_then(|s| s.props.get("font")) {
                    Some(p) => get_font(fonts, p).await,
                    None => None,
                };

                if o.name == "button" {
                    let d = measure_text(&text, font, size as u16, 1.0);
                    draw_styled(&text, x + (w - d.width) / 2., y + (h + d.height) / 2., size, tcolor, stroke, font);
                } else {
                    draw_styled(&text, x, y + size, size, tcolor, stroke, font);
                }
            }
        }
    }
}