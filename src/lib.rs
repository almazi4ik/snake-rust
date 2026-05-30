use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Element};
use std::collections::HashMap;
use std::cell::RefCell;

// Состояние игры
thread_local! {
    static GAME_STATE: RefCell<GameState> = RefCell::new(GameState::new());
}

struct GameState {
    snake: Vec<(i32, i32)>,      // позиции сегментов
    food: (i32, i32),            // еда
    direction: Direction,        // текущее направление
    next_direction: Direction,
    game_over: bool,
    board_size: i32,
    cell_size: i32,
    segment_elements: HashMap<String, Element>,
    food_element: Option<Element>,
    score_span: Option<Element>,
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up, Down, Left, Right,
}

impl GameState {
    fn new() -> Self {
        let board_size = 25;
        let cell_size = 24;
        
        // Начальная змейка из 3 клеток
        let snake = vec![
            (12, 13),
            (12, 14),
            (12, 15),
        ];
        
        GameState {
            snake,
            food: (15, 15),
            direction: Direction::Up,
            next_direction: Direction::Up,
            game_over: false,
            board_size,
            cell_size,
            segment_elements: HashMap::new(),
            food_element: None,
            score_span: None,
        }
    }
}

#[wasm_bindgen]
pub fn start_game() {
    // Очищаем доску
    clear_board();
    
    // Получаем элементы DOM
    let window = window().unwrap();
    let document = window.document().unwrap();
    let board = document.get_element_by_id("game-board").unwrap();
    
    // Очищаем board
    while let Some(child) = board.first_child() {
        board.remove_child(&child).unwrap();
    }
    
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        *s = GameState::new();
        
        // Сохраняем ссылку на span со счетом
        if let Some(span) = document.get_element_by_id("scoreSpan") {
            s.score_span = Some(span);
        }
        
        // Отрисовываем начальную змейку
        for &(x, y) in &s.snake {
            add_segment_element(x, y, &board);
        }
        
        // Отрисовываем еду
        add_food_element(s.food.0, s.food.1, &board);
        
        update_score_display(&s);
    });
    
    // Запускаем игровой цикл
    start_game_loop();
}

fn clear_board() {
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        for (_, el) in s.segment_elements.drain() {
            if let Some(parent) = el.parent_node() {
                parent.remove_child(&el).unwrap();
            }
        }
        if let Some(food) = &s.food_element {
            if let Some(parent) = food.parent_node() {
                parent.remove_child(food).unwrap();
            }
        }
        s.food_element = None;
    });
}

fn add_segment_element(x: i32, y: i32, board: &Element) {
    let window = window().unwrap();
    let document = window.document().unwrap();
    
    let div = document.create_element("div").unwrap();
    div.set_attribute("style", &format!(
        "position: absolute; width: {}px; height: {}px; left: {}px; top: {}px; \
         background: radial-gradient(circle at 35% 35%, #7ef095, #2e8e47); \
         border-radius: 40% 60% 45% 55% / 50% 40% 60% 50%; \
         box-shadow: 0 0 0 1px #c8ffa0, inset 0 1px 2px rgba(255,255,200,0.6); \
         transition: all 0.03s linear;",
        s.cell_size, s.cell_size, x * s.cell_size, y * s.cell_size
    )).unwrap();
    
    board.append_child(&div).unwrap();
    
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.segment_elements.insert(format!("{},{}", x, y), div);
    });
}

fn add_food_element(x: i32, y: i32, board: &Element) {
    let window = window().unwrap();
    let document = window.document().unwrap();
    
    let div = document.create_element("div").unwrap();
    div.set_attribute("style", &format!(
        "position: absolute; width: {}px; height: {}px; left: {}px; top: {}px; \
         background: radial-gradient(circle at 40% 40%, #ffb347, #ff6b2b); \
         border-radius: 50%; box-shadow: 0 0 8px #ffb347, inset 0 1px 3px #ffffaa; \
         animation: pulse 0.5s ease-in-out infinite alternate;",
        GAME_STATE.with(|s| s.borrow().cell_size),
        GAME_STATE.with(|s| s.borrow().cell_size),
        x * GAME_STATE.with(|s| s.borrow().cell_size),
        y * GAME_STATE.with(|s| s.borrow().cell_size)
    )).unwrap();
    
    // Добавляем CSS анимацию
    let style = document.create_element("style").unwrap();
    style.set_attribute("type", "text/css").unwrap();
    style.set_text_content(Some(r#"
        @keyframes pulse {
            from { transform: scale(0.95); opacity: 0.9; }
            to { transform: scale(1.15); opacity: 1; }
        }
    "#));
    document.head().unwrap().append_child(&style).ok();
    
    board.append_child(&div).unwrap();
    
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.food_element = Some(div);
    });
}

fn update_score_display(state: &GameState) {
    if let Some(span) = &state.score_span {
        let _ = span.set_text_content(Some(&state.snake.len().to_string()));
    }
}

fn update_visuals() {
    GAME_STATE.with(|state| {
        let s = state.borrow();
        let window = window().unwrap();
        let document = window.document().unwrap();
        let board = document.get_element_by_id("game-board").unwrap();
        
        // Перерисовываем все сегменты (обновляем позиции)
        for (key, el) in s.segment_elements.iter() {
            if let Some(parent) = el.parent_node() {
                parent.remove_child(el).unwrap();
            }
        }
        
        let mut new_map = HashMap::new();
        for (idx, &(x, y)) in s.snake.iter().enumerate() {
            let div = document.create_element("div").unwrap();
            let is_head = idx == 0;
            let gradient = if is_head {
                "radial-gradient(circle at 40% 40%, #b0ff90, #3ca55c)"
            } else {
                "radial-gradient(circle at 35% 35%, #7ef095, #2e8e47)"
            };
            let border_radius = if is_head {
                "45% 55% 50% 50% / 55% 45% 55% 45%"
            } else {
                "40% 60% 45% 55% / 50% 40% 60% 50%"
            };
            
            div.set_attribute("style", &format!(
                "position: absolute; width: {}px; height: {}px; left: {}px; top: {}px; \
                 background: {}; border-radius: {}; \
                 box-shadow: 0 0 0 1px #d4ffb0, inset 0 1px 2px rgba(255,255,200,0.7); \
                 z-index: {};",
                s.cell_size, s.cell_size, x * s.cell_size, y * s.cell_size,
                gradient, border_radius,
                if is_head { 10 } else { 5 }
            )).unwrap();
            
            board.append_child(&div).unwrap();
            new_map.insert(format!("{},{}", x, y), div);
        }
        
        // Обновляем карту элементов
        drop(s);
        let mut s_mut = state.borrow_mut();
        s_mut.segment_elements = new_map;
        
        // Обновляем позицию еды
        if let Some(food_el) = &s_mut.food_element {
            if let Some(parent) = food_el.parent_node() {
                parent.remove_child(food_el).unwrap();
            }
            let new_food_div = document.create_element("div").unwrap();
            new_food_div.set_attribute("style", &format!(
                "position: absolute; width: {}px; height: {}px; left: {}px; top: {}px; \
                 background: radial-gradient(circle at 40% 40%, #ffb347, #ff6b2b); \
                 border-radius: 50%; box-shadow: 0 0 10px #ffaa44; animation: pulse 0.5s infinite alternate;",
                s_mut.cell_size, s_mut.cell_size,
                s_mut.food.0 * s_mut.cell_size,
                s_mut.food.1 * s_mut.cell_size
            )).unwrap();
            board.append_child(&new_food_div).unwrap();
            s_mut.food_element = Some(new_food_div);
        }
    });
}

fn game_step() {
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if s.game_over {
            return;
        }
        
        // Применяем накопленное направление
        s.direction = s.next_direction;
        
        // Вычисляем новую голову
        let head = s.snake[0];
        let new_head = match s.direction {
            Direction::Up => (head.0, head.1 - 1),
            Direction::Down => (head.0, head.1 + 1),
            Direction::Left => (head.0 - 1, head.1),
            Direction::Right => (head.0 + 1, head.1),
        };
        
        // Проверка столкновения со стеной
        if new_head.0 < 0 || new_head.0 >= s.board_size || new_head.1 < 0 || new_head.1 >= s.board_size {
            s.game_over = true;
            show_game_over();
            return;
        }
        
        // Проверка, съела ли еду
        let ate_food = new_head == s.food;
        
        // Вставляем новую голову
        s.snake.insert(0, new_head);
        
        if !ate_food {
            s.snake.pop();
        } else {
            // Генерируем новую еду
            let mut free_cells = Vec::new();
            for x in 0..s.board_size {
                for y in 0..s.board_size {
                    if !s.snake.contains(&(x, y)) {
                        free_cells.push((x, y));
                    }
                }
            }
            if let Some(new_food) = free_cells.choose(&mut js_sys::Math::random) {
                s.food = *new_food;
            } else {
                // Победа! Заполнили всё поле
                s.game_over = true;
                show_win();
                return;
            }
        }
        
        // Проверка столкновения с собой
        if s.snake[1..].contains(&new_head) {
            s.game_over = true;
            show_game_over();
            return;
        }
        
        update_score_display(&s);
        drop(s);
        update_visuals();
    });
}

fn show_game_over() {
    let window = window().unwrap();
    let document = window.document().unwrap();
    if let Some(div) = document.get_element_by_id("game-board") {
        let msg = document.create_element("div").unwrap();
        msg.set_attribute("style", "position: absolute; top: 45%; left: 0; right: 0; text-align: center; color: white; background: #000000aa; padding: 16px; font-size: 24px; font-weight: bold; backdrop-filter: blur(4px); z-index: 100;").unwrap();
        msg.set_text_content(Some("💀 ИГРА ОКОНЧЕНА 💀"));
        div.append_child(&msg).unwrap();
    }
}

fn show_win() {
    let window = window().unwrap();
    let document = window.document().unwrap();
    if let Some(div) = document.get_element_by_id("game-board") {
        let msg = document.create_element("div").unwrap();
        msg.set_attribute("style", "position: absolute; top: 45%; left: 0; right: 0; text-align: center; color: gold; background: #1a3f2acc; padding: 16px; font-size: 24px; font-weight: bold;").unwrap();
        msg.set_text_content(Some("🏆 ПОБЕДА! ВСЁ ПОЛЕ ЗАПОЛНЕНО 🏆"));
        div.append_child(&msg).unwrap();
    }
}

fn start_game_loop() {
    let closure = Closure::wrap(Box::new(move || {
        game_step();
    }) as Box<dyn FnMut()>);
    
    let interval = window().unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(&closure, 150)
        .unwrap();
    
    // Сохраняем interval чтобы не потерять, но для простоты оставим
    std::mem::forget(closure);
}

#[wasm_bindgen]
pub fn change_direction(dir_str: &str) {
    GAME_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if s.game_over {
            return;
        }
        
        let new_dir = match dir_str {
            "up" => Direction::Up,
            "down" => Direction::Down,
            "left" => Direction::Left,
            "right" => Direction::Right,
            _ => return,
        };
        
        // Запрещаем разворот на 180 градусов
        match (s.direction, new_dir) {
            (Direction::Up, Direction::Down) => return,
            (Direction::Down, Direction::Up) => return,
            (Direction::Left, Direction::Right) => return,
            (Direction::Right, Direction::Left) => return,
            _ => {}
        }
        
        s.next_direction = new_dir;
    });
}

#[wasm_bindgen]
pub fn reset_game() {
    start_game();
                    }
