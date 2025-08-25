use ligames::{Tango, TangoRestriction, TangoTile};
use reqwasm::http::Request;
use web_sys::console;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let board = use_state(|| None::<Tango>);

    // Load board from backend
    {
        let board = board.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let resp =
                    Request::get("http://localhost:8081/api/tango-board")
                        .send()
                        .await
                        .expect("request failed");
                let data: Tango = resp.json().await.expect("invalid JSON");
                board.set(Some(data));
            });
            || ()
        });
    }

    html! {
        <div>
            <h1>{ "Tango Solver (Rust + Yew)" }</h1>
            if let Some(board) = (*board).clone() {
                <Board board={board} />
            } else {
                <p>{ "Loading board..." }</p>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct BoardProps {
    board: Tango,
}

struct Board {
    board: Tango,
}

struct TileClick {
    row: usize,
    col: usize,
}

impl Component for Board {
    type Message = TileClick;
    type Properties = BoardProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            board: ctx.props().board.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.board.cycle_tile(msg.col, msg.row);
        console::log_1(
            &format!(
                "Clicked on tile ({}, {}) - {:?}",
                msg.row,
                msg.col,
                self.board.get_tile(msg.row, msg.col)
            )
            .into(),
        );
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let width = self.board.grid.width;
        let height = self.board.grid.height;

        // Prepare tiles as a 2D vector for easy indexing
        let mut tiles_2d: Vec<Vec<&TangoTile>> = Vec::new();
        for row in 0..height {
            tiles_2d.push(
                (0..width)
                    .map(|col| &self.board.grid.tiles[row * width + col])
                    .collect(),
            );
        }

        // Build the grid with optional connectors
        let mut grid_html = Vec::new();

        for row in 0..height {
            let mut row_html = Vec::new();
            for col in 0..width {
                let onclick = ctx
                    .link()
                    .callback(move |_event: MouseEvent| TileClick { row, col });

                // TangoTile
                let tile = tiles_2d[row][col];
                let label = match tile {
                    TangoTile::Empty => "⬜",
                    TangoTile::Red => "🟥",
                    TangoTile::Blue => "🟦",
                };
                row_html.push(
                    html! { <div class="tile" {onclick} >{ label }</div> },
                );

                // Horizontal restriction
                if col + 1 < width {
                    let mut conn = "".to_string();
                    for r in &self.board.restrictions {
                        match r {
                            TangoRestriction::Same(a, b)
                                if (a.0, a.1) == (row, col)
                                    && (b.0, b.1) == (row, col + 1) =>
                            {
                                conn = "=".to_string()
                            }
                            TangoRestriction::Different(a, b)
                                if (a.0, a.1) == (row, col)
                                    && (b.0, b.1) == (row, col + 1) =>
                            {
                                conn = "×".to_string()
                            }
                            _ => {}
                        }
                    }
                    if !conn.is_empty() {
                        row_html.push(
                            html! { <div class="connector">{ conn }</div> },
                        );
                    } else {
                        row_html.push(html! { <div class="connector"></div> });
                    }
                }
            }

            grid_html.push(html! { <div style={format!("display:grid; grid-template-columns: repeat({}, 40px 20px); gap:4px;", width)}>{ row_html }</div>});
            row_html = Vec::new();
            // Optional: add a row of vertical connectors here if needed
            // TODO: Implement vertical connectors between rows
            if row + 1 < height {
                for col in 0..width {
                    let mut conn = "".to_string();
                    for r in &self.board.restrictions {
                        match r {
                            TangoRestriction::Same(a, b)
                                if (a.0, a.1) == (row, col)
                                    && (b.0, b.1) == (row + 1, col) =>
                            {
                                conn = "=".to_string()
                            }
                            TangoRestriction::Different(a, b)
                                if (a.0, a.1) == (row, col)
                                    && (b.0, b.1) == (row + 1, col) =>
                            {
                                conn = "×".to_string()
                            }
                            _ => {}
                        }
                    }
                    row_html.push(
                    html! { <div class="connector-vertical">{ conn }</div> },
                );

                    // filler for horizontal spacing (skip after last column)
                    if col + 1 < width {
                        row_html.push(
                            html! { <div class="connector-space"></div> },
                        );
                    }
                }
            }

            grid_html.push(html! { <div style={format!("display:grid; grid-template-columns: repeat({}, 40px 20px); gap:4px;", width)}>{ row_html }</div>});
        }

        html! {
            <div>
                <div
                    class="grid"
                >
                    { for grid_html }
                </div>
            </div>
        }
    }
}
