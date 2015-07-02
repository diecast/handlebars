extern crate diecast;
extern crate handlebars;
extern crate rustc_serialize;
extern crate typemap;

use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use diecast::{Handle, Item, Bind};
use rustc_serialize::json::Json;
use handlebars::Handlebars;

pub struct Templates;

impl typemap::Key for Templates {
    type Value = Arc<Handlebars>;
}

pub fn register_templates(bind: &mut Bind) -> diecast::Result<()> {
    fn load_template(path: &Path, registry: &mut Handlebars) {
        let mut template = String::new();

        File::open(path)
        .unwrap()
        .read_to_string(&mut template)
        .unwrap();

        let path = path.with_extension("");
        let name = path.file_name().unwrap().to_str().unwrap();

        registry.register_template_string(name, template).unwrap();
    }

    let mut registry = Handlebars::new();

    // NOTE: this needs access to all of the templates,
    // even if only one changed, so don't use iter!
    for item in bind.items() {
        load_template(&item.source().unwrap(), &mut registry);
    }

    bind.data().extensions.write().unwrap().insert::<Templates>(Arc::new(registry));

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
            let registry = data.get::<Templates>().unwrap();

            let json = (self.handler)(item);

            registry.render(&self.name, &json).unwrap()
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


