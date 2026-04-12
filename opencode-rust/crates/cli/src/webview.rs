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
pub struct WebViewInstance {
    _webview: WebView,
}

#[cfg(feature = "desktop")]
impl WebViewInstance {
    pub fn new(url: &str, title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let event_loop: EventLoop<()> = EventLoop::new();
        let window = WindowBuilder::new().with_title(title).build(&event_loop)?;

        let webview = WebViewBuilder::new(window)?.with_url(url)?.build()?;

        Ok(WebViewInstance { _webview: webview })
    }

    pub fn run(self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit;
            }
        });
    }
}

#[cfg(not(feature = "desktop"))]
pub struct WebViewInstance;

#[cfg(not(feature = "desktop"))]
impl WebViewInstance {
    pub fn new(_url: &str, _title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(WebViewInstance)
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(feature = "desktop")]
pub fn spawn_webview_thread(
    url: String,
    title: String,
) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error>> {
    let handle = thread::spawn(move || {
        let event_loop: EventLoop<()> = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&title)
            .build(&event_loop)
            .expect("Failed to create window");

        let webview = WebViewBuilder::new(window)
            .expect("Failed to create WebViewBuilder")
            .with_url(&url)
            .expect("Failed to set URL")
            .build()
            .expect("Failed to build WebView");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *control_flow = ControlFlow::Exit;
            }
        });
    });

    Ok(handle)
}

#[cfg(not(feature = "desktop"))]
pub fn spawn_webview_thread(
    _url: String,
    _title: String,
) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error>> {
    Ok(thread::spawn(|| {}))
}
