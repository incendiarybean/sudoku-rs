use std::collections::HashSet;

use egui::{vec2, Color32, Layout, Vec2};
use rand::Rng;

const SUDOKU_SIZE: f32 = 40. * 11. - 20.;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SudokuApp {
    #[serde(skip)]
    sudoku_grid: Vec<Vec<String>>,

    sudoku_solution: Vec<Vec<String>>,
    hints: bool,
}

impl Default for SudokuApp {
    fn default() -> Self {
        Self {
            sudoku_grid: vec![vec![String::default(); 9]; 9],
            sudoku_solution: vec![vec![String::default(); 9]; 9],
            hints: false,
        }
    }
}

fn is_three(index: usize) -> bool {
    index % 3 == 0 && index != 0
}

impl SudokuApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn build_sudoku(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(ui.available_height() / 2. - SUDOKU_SIZE / 2.);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2. - SUDOKU_SIZE / 2.);
                ui.group(|ui| {
                    egui::Grid::new("some_unique_id")
                        .min_col_width(0.)
                        .min_row_height(0.)
                        .spacing(vec2(5., 5.))
                        .show(ui, |ui| {
                            for (row_index, row) in self.sudoku_grid.iter_mut().enumerate() {
                                if is_three(row_index) {
                                    ui.end_row();
                                }

                                for (column_index, value) in row.iter_mut().enumerate() {
                                    if is_three(column_index) {
                                        ui.allocate_space(Vec2 { x: 0., y: 0. });
                                    }

                                    let text_color = if self.hints {
                                        if self.sudoku_solution[row_index][column_index] == *value {
                                            Color32::GREEN
                                        } else {
                                            Color32::RED
                                        }
                                    } else {
                                        Color32::WHITE
                                    };

                                    ui.add(
                                        egui::TextEdit::singleline(value)
                                            .min_size(Vec2 { x: 40., y: 40. })
                                            .horizontal_align(egui::Align::Center)
                                            .vertical_align(egui::Align::Center)
                                            .text_color(text_color),
                                    );

                                    if column_index == 8 {
                                        ui.end_row();
                                    }
                                }
                            }
                        });
                });
            });

            ui.add_space(10.);
            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                egui::Grid::new("button_area")
                    .num_columns(1)
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.button("Validate!");
                        });
                    });
            });
        });
    }
}

impl eframe::App for SudokuApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "control_panel").show(ctx, |ui| {
            ui.add_space(10.);

            ui.horizontal(|ui| {
                ui.label("Enable Hints");
                toggle_ui(ui, &mut self.hints);
            });

            ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
                egui::warn_if_debug_build(ui);
                ui.add_space(10.);
                if ui.button("Generate Sudoku!").clicked() {
                    let (puzzle, solution) = create_puzzle();

                    self.sudoku_solution = solution;
                    self.sudoku_grid = puzzle;
                };
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_sudoku(ctx, ui);
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([SUDOKU_SIZE * 2., SUDOKU_SIZE * 1.2]),
        ..Default::default()
    };
    eframe::run_native(
        "Sudoku Generator",
        native_options,
        Box::new(|cc| Ok(Box::new(SudokuApp::new(cc)))),
    )
}

fn try_make_solution() -> Result<Vec<Vec<String>>, String> {
    let mut created_puzzle: Vec<Vec<String>> = vec![vec![String::default(); 9]; 9];
    let mut created_rows: Vec<HashSet<i8>> = vec![HashSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]); 9];
    let mut created_cols: Vec<HashSet<i8>> = vec![HashSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]); 9];
    let mut created_squares: Vec<HashSet<i8>> = vec![HashSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]); 9];
    let mut rng = rand::thread_rng();

    for row_index in 0..9 {
        for column_index in 0..9 {
            // Get the position of the current row & column in the squares
            let row_square_index = ((row_index / 3) as f32).floor();
            let col_square_index = ((column_index / 3) as f32).floor();
            let current_square_index = (row_square_index * 3. + col_square_index) as usize;

            // Get the current row, column and square
            let current_row: HashSet<i8> = created_rows[row_index].clone();
            let current_column: HashSet<i8> = created_cols[column_index].clone();
            let current_square = created_squares[current_square_index].clone();

            // Get the available values for each field
            // It is no longer an option if it is already in the current row/column/square
            let options: Vec<i8> = current_row
                .clone()
                .into_iter()
                .filter(|x| current_column.clone().contains(x))
                .filter(|x| current_square.clone().contains(x))
                .collect();

            // If the puzzle has no possible options, throw an error
            if options.len() == 0 {
                return Err(String::from("Invalid Values"));
            }

            // Get by index, shrinks every time
            let random_variation: f32 = rng.gen();
            let option: i8 = loop {
                let option_index = (random_variation * (options.len() as f32)).floor() as usize;
                match options.get(option_index) {
                    Some(option) => break option.to_owned(),
                    None => {}
                }
            };

            // Set the current field's value
            created_puzzle[row_index][column_index] = option.to_string();

            // Remove the value as an available option
            created_cols[column_index].remove(&option);
            created_rows[row_index].remove(&option);
            created_squares[current_square_index].remove(&option);
        }
    }

    // A successfully generated Sudoku
    Ok(created_puzzle)
}

fn generate_solution() -> Vec<Vec<String>> {
    let solution: Vec<Vec<String>> = loop {
        if let Ok(solution) = try_make_solution() {
            break solution;
        }
    };
    solution
}

fn try_make_puzzle(row: &mut Vec<String>) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut items_removed = 0;

    loop {
        let random_variation: f32 = rng.gen();

        let index_to_remove = (random_variation * (row.len() as f32)).floor() as usize;

        if row[index_to_remove] == String::default() {
            continue;
        }

        row[index_to_remove] = String::default();
        items_removed += 1;

        if items_removed == 6 {
            break;
        }
    }

    row.to_vec()
}

fn generate_puzzle(solution: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut solution = solution;

    let puzzle = loop {
        let mut puzzle: Vec<Vec<String>> = Vec::new();
        for row in solution.iter_mut() {
            puzzle.push(try_make_puzzle(row));
        }

        break puzzle;
    };

    puzzle
}

fn create_puzzle() -> (Vec<Vec<String>>, Vec<Vec<String>>) {
    let solution = generate_solution();
    let puzzle = generate_puzzle(solution.clone());

    (puzzle, solution)
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);

    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, false, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();

        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);

        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);

        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
