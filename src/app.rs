use std::collections::HashMap;

use anyhow::Result;
use crossterm::event::Event;
use ratatui::{backend::Backend, Frame};

use crate::views::{home::Home, View, keyboard::Keyboard};

pub enum AppCommand {
    ChangeRoute(&'static str),
    Shutdown,
}

pub struct App<B: Backend> {
    is_running: bool,
    route_map: HashMap<&'static str, Box<dyn View<B>>>,
    current_route: &'static str,
}

impl<B: Backend> App<B> {
    pub fn init() -> Result<Self> {
        let routes: Vec<(&'static str, Box<dyn View<B>>)> = vec![
                ("/", Box::new(Home::init())),
                ("/keyboard", Box::new(Keyboard::init()?))
            ];

        let route_map: HashMap<&'static str, Box<dyn View<B>>> = routes.into_iter().collect();

        Ok(        Self {
            is_running: true,
            route_map,
            current_route: "/",
        })
    }
    pub fn render(&mut self, frame: &mut Frame<B>) {
        let view = self.route_map.get_mut(self.current_route).unwrap();

        view.render(frame)
    }
    pub fn on_event(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            let view = self.route_map.get_mut(self.current_route).unwrap();

            let handler_result = view.on_event(key_event);

            if handler_result.is_none() {
                return;
            }

            match handler_result.unwrap() {
                AppCommand::Shutdown => {
                    self.is_running = false;
                }
                AppCommand::ChangeRoute(route) => self.current_route = route,
            }
        }
    }
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
