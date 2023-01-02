use wgpus::window::Window;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();
    Window::new().await.spawn().await;
}
