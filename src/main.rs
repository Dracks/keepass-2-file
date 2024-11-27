use handlebars::{
    no_escape, Context, Handlebars, Helper, HelperDef, JsonValue, RenderContext, RenderError, ScopedJson,
};
use keepass::{db::NodeRef, Database, DatabaseKey};
use std::error::Error;
use std::fs::File;

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut handlebars = Handlebars::new();
    println!("Hello!");

    let mut file = File::open("test_resources/test_db.kdbx")?;
    let template = std::fs::read_to_string("test_resources/.env.example").unwrap();
    let key = DatabaseKey::new().with_password("MyTestPass");
    let db = Database::open(&mut file, key)?;

    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("keepass", Box::new(KeepassHelper { db }));

    println!(
        "{}",
        handlebars.render_template(&template, &())?
    );
    Ok(())
}
