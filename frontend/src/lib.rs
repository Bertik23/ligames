use ligames::{Tango, TangoRestriction, TangoTile};
use reqwasm::http::Request;
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

#[function_component(Board)]
fn board(props: &BoardProps) -> Html {
    let mut tiles = Vec::new();

    for tile in props.board.grid.tiles.iter() {
        let label = match tile {
            TangoTile::Empty => "⬜",
            TangoTile::Red => "🟥",
            TangoTile::Blue => "🟦",
        };
        tiles.push(html! {
            <div class="tile">{ label }</div>
        });
    }

    let restrictions = props.board.restrictions.iter().map(|r| match r {
        TangoRestriction::Same(a, b) => {
            html! { <li>{ format!("Same {:?} ↔ {:?}", a, b) }</li> }
        }
        TangoRestriction::Different(a, b) => {
            html! { <li>{ format!("Different {:?} ↔ {:?}", a, b) }</li> }
        }
    });

    html! {
        <div>
            <div
                class="grid"
                style={format!("display:grid; grid-template-columns: repeat({}, 40px); gap:4px;", props.board.grid.width)}
            >
                { for tiles }
            </div>
            <h3>{ "Restrictions:" }</h3>
            <ul>
                { for restrictions }
            </ul>
        </div>
    }
}
