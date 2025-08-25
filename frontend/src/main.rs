mod lib;

fn main() {
    // Mount the Yew app into the <body>
    yew::Renderer::<lib::App>::new().render();
}
