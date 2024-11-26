use leptos::*;

#[component]
pub fn Footer(#[prop(default = "".to_string(), into)] style: String) -> impl IntoView {
    view! {
        <footer style="
            display: flex;
            justify-content: center;
            align-items: center;
            background-color: lightgray;
        ".to_string() + &style>
            <p>
                "Made with ❤️ by "
                <a href="https://lioqing.com" target="_blank">
                    " Lio Qing"
                </a>
                " | "
                <a href="https://github.com/lioqing/wgpu-leptos-template" target="_blank">
                    "GitHub Repository"
                </a>
            </p>
        </footer>
    }
}
