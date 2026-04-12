use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[cfg(feature = "desktop")]
use wry::application::event::{Event, WindowEvent};
#[cfg(feature = "desktop")]
use wry::application::event_loop::{ControlFlow, EventLoop};
#[cfg(feature = "desktop")]
use wry::application::window::WindowBuilder;
#[cfg(feature = "desktop")]
use wry::webview::{WebView, WebViewBuilder};

#[cfg(feature = "desktop")]
pub struct WebViewManager {
    stop_tx: Sender<()>,
    join_handle: Option<thread::JoinHandle<()>>,
}

#[cfg(feature = "desktop")]
impl WebViewManager {
    pub fn new(url: &str, title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stop_tx, stop_rx) = channel::<()>();
        let title = title.to_string();
        let url = url.to_string();

        let join_handle = thread::spawn(move || {
            if let Err(e) = run_webview_thread(&url, &title, stop_rx) {
                eprintln!("WebView error: {}", e);
            }
        });

        Ok(WebViewManager {
            stop_tx,
            join_handle: Some(join_handle),
        })
    }

    pub fn stop(&self) {
        let _ = self.stop_tx.send(());
    }

    pub fn wait_until_stopped(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(feature = "desktop")]
fn run_webview_thread(
    url: &str,
    title: &str,
    stop_rx: Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop: EventLoop<()> = EventLoop::new();
    let window = WindowBuilder::new().with_title(title).build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?.with_url(url)?.build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if stop_rx.try_recv().is_ok() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });

    Ok(())
}

#[cfg(not(feature = "desktop"))]
pub struct WebViewManager;

#[cfg(not(feature = "desktop"))]
impl WebViewManager {
    pub fn new(_url: &str, _title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(WebViewManager)
    }

    pub fn stop(&self) {}

    pub fn wait_until_stopped(&mut self) {}
}

#[cfg(feature = "desktop")]
impl Drop for WebViewManager {
    fn drop(&mut self) {
        self.stop();
        self.wait_until_stopped();
    }
}
