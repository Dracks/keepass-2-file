use handlebars::{
    no_escape, Context, Handlebars, Helper, HelperDef, JsonValue, RenderContext, RenderError, ScopedJson,
};
use keepass::{db::NodeRef, Database};

pub struct KeepassHelper {
    db: Database,
}

impl HelperDef for KeepassHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> std::result::Result<ScopedJson<'rc>, RenderError> {
        let args = h
            .params()
            .iter()
            .map(|x| x.value())
            .collect::<Vec<&serde_json::Value>>();
        let path = args
            .iter()
            .map(|&x| x.as_str().unwrap())
            .collect::<Vec<&str>>();
        println!("{:?}", args);
        if let Some(node) = self.db.root.get(&path) {
            match node {
                NodeRef::Group(_) => {
                    println!("Not found path {:?}", path);
                    Ok(ScopedJson::Derived(JsonValue::from(
                        "Found group not entry",
                    )))
                }
                NodeRef::Entry(entry) => {
                    //println!("Found! {0}", entry.get_title().unwrap())
                    Ok(ScopedJson::Derived(JsonValue::from(
                        entry.get_password().unwrap(),
                    )))
                }
            }
        } else {
            println!("Not found path {:?}", path);
            Ok(ScopedJson::Derived(JsonValue::from("Not found path")))
        }
    }
}

pub fn build_handlebars<'reg>(db: Database) -> Handlebars<'reg> {
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("keepass", Box::new(KeepassHelper { db }));

    return handlebars
}
