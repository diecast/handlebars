extern crate diecast;
extern crate handlebars;
extern crate rustc_serialize;
extern crate typemap;

use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use rustc_serialize::json::Json;
use handlebars::Handlebars;

use diecast::{Handle, Item, Bind};

pub struct Templates;

impl typemap::Key for Templates {
    type Value = Arc<Handlebars>;
}

pub fn register_templates(bind: &mut Bind) -> diecast::Result<()> {
    fn load_template(path: &Path, registry: &mut Handlebars) -> diecast::Result<()> {
        let mut template = String::new();

        let mut f = try!(File::open(path));

        try!(f.read_to_string(&mut template));

        let no_ext = path.with_extension("");

        let name = try! {
            no_ext.file_name()
            .ok_or(format!(
                "[HANDLEBARS] not a regular file: {:?}",
                path))
        };

        let as_str = try! {
            name.to_str()
            .ok_or(format!(
                "[HANDLEBARS] could not convert file name to UTF-8: {:?}",
                path))
        };

        registry.register_template_string(as_str, template).map_err(From::from)
    }

    let mut registry = Handlebars::new();

    // NOTE: this needs access to all of the templates,
    // even if only one changed, so don't use iter!
    for item in bind.items() {
        let source = try! {
            item.source()
            .ok_or(format!(
                "[HANDLEBARS] no source for item {:?}",
                item))
        };

        try!(load_template(&source, &mut registry));
    }

    bind.data().extensions.write().unwrap()
    .insert::<Templates>(Arc::new(registry));

    Ok(())
}

pub struct RenderTemplate<H>
where H: Fn(&Item) -> Json + Sync + Send + 'static {
    binding: String,
    name: String,
    handler: H,
}

impl<H> Handle<Item> for RenderTemplate<H>
where H: Fn(&Item) -> Json + Sync + Send + 'static {
    fn handle(&self, item: &mut Item) -> diecast::Result<()> {
        item.body = {
            let data =
                item.bind().dependencies[&self.binding]
                .data().extensions.read().unwrap();

            let registry = try! {
                data.get::<Templates>()
                .ok_or(format!(
                    "[HANDLEBARS] no template registry found in binding {:?}",
                    self.binding))
            };

            let json = (self.handler)(item);

            try!(registry.render(&self.name, &json))
        };

        Ok(())
    }
}

#[inline]
pub fn render<H, D, N>(binding: D, name: N, handler: H) -> RenderTemplate<H>
where H: Fn(&Item) -> Json + Sync + Send + 'static, D: Into<String>, N: Into<String> {
    RenderTemplate {
        binding: binding.into(),
        name: name.into(),
        handler: handler,
    }
}

