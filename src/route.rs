use std::collections::HashMap;

use crate::request::Request;
use crate::response::HttpResponse;

pub struct Route {
    on: Option<Box<dyn Fn(&Request) -> HttpResponse + Send + Sync + 'static>>,
    any: Option<Box<Route>>,
    distinct: HashMap<&'static str, Route>,
}

impl Route {
    pub fn new() -> Route {
        Route {
            on: None,
            any: None,
            distinct: HashMap::new(),
        }
    }

    fn print_route(&self, indent_by: usize) {
        let indent = " ".repeat(indent_by);
        let handler = if let Some(_) = self.on { "yes" } else { "no" };
        println!("{} handler: {}", indent, handler);

        if let Some(rt) = &self.any {
            println!("{} wildcrd:", indent);
            rt.print_route(indent_by + 2)
        } else {
            println!("{} wildcrd: no", indent);
        };

        if self.distinct.len() > 0 {
            println!("{} statics:", indent);
            for (&path, rt) in &self.distinct {
                println!("{} * {}", indent, path);
                rt.print_route(indent_by + 4);
            }
        }
    }

    pub fn print(&self) {
        self.print_route(0);
    }

    pub fn add(
        &mut self,
        path: &'static str,
        handler: impl Fn(&Request) -> HttpResponse + Send + Sync + 'static,
    ) {
        let mut route = self;
        for seg in Self::segment_vec(path) {
            route = if seg == "*" {
                route.any_route();
                route.any.as_mut().unwrap()
            } else {
                route.distinct_route(seg);
                route.distinct.get_mut(seg).unwrap()
            };
        }
        route.set_handler(handler);
    }

    fn set_handler(&mut self, handler: impl Fn(&Request) -> HttpResponse + Send + Sync + 'static) {
        self.on = Some(Box::new(handler));
    }

    fn any_route(&mut self) {
        if let None = self.any {
            self.any = Some(Box::new(Route::new()));
        }
    }

    fn distinct_route(&mut self, seg: &'static str) {
        if let None = self.distinct.get(seg) {
            self.distinct.insert(seg, Route::new());
        }
    }

    fn segment_vec(path: &str) -> Vec<&str> {
        let path = path.trim().trim_matches('/');
        path.split('/').collect()
    }

    pub fn route(
        &self,
        path: &str,
    ) -> Option<&(dyn Fn(&Request) -> HttpResponse + Send + Sync + 'static)> {
        let mut route = self;
        let mut catch = self;

        let parts = Self::segment_vec(path);

        for part in parts {
            if let Some(_) = &route.any {
                catch = route;
            }

            if let Some(subroute) = route.distinct.get(part) {
                route = subroute;
            } else if let Some(any) = &route.any {
                route = any.as_ref();
            } else {
                break;
            }
        }

        if let Some(handler) = &route.on {
            return Some(handler.as_ref());
        } else if let Some(any) = &catch.any {
            if let Some(handler) = &any.on {
                return Some(handler.as_ref());
            }
        }

        None
    }
}
