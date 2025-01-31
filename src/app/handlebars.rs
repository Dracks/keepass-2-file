use handlebars::{
    no_escape, Context, Handlebars, Helper, HelperDef, JsonRender, JsonValue, RenderContext,
    RenderError, ScopedJson,
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
            .map(|x| x.value().render())
            .collect::<Vec<String>>();
        let path = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        println!("{:?}", args);
        if let Some(node) = self.db.root.get(&path) {
            match node {
                NodeRef::Group(_) => Ok(ScopedJson::Derived(JsonValue::from(
                    "<Not found keepass entry>",
                ))),
                NodeRef::Entry(entry) => {
                    //println!("Found! {0}", entry.get_title().unwrap())
                    if let Some(password) = entry.get_password() {
                        Ok(ScopedJson::from(JsonValue::from(password)))
                    } else {
                        Ok(ScopedJson::from(JsonValue::from(
                            "<No password found in entry>",
                        )))
                    }
                }
            }
        } else {
            Ok(ScopedJson::Derived(JsonValue::from(
                "<Not found keepass entry>",
            )))
        }
    }
}

pub fn build_handlebars<'reg>(db: Database) -> Handlebars<'reg> {
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("keepass", Box::new(KeepassHelper { db }));

    handlebars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_db() -> Database {
        let mut file = File::open("test_Resources/test_db.kdbx").expect("Test DB cannot be open");

        let key = DatabaseKey::new().with_password("MyTestPass");
        Database::open(&mut file, key).expect("Cannot open the DB")
    }

    use keepass::DatabaseKey;
    use std::fs::File;

    #[test]
    fn test_handlebars_keepass_variables() {
        let handlebars = build_handlebars(get_db());

        let template =
            "VAR_NAME=\"My name\"\nVAR_SECRET=\"{{keepass \"group1\" \"Some weird name\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"S$c&<J)=EVm#xo{t]<ml\""));
    }

    /*#[test]
    fn test_handlebars_keepass_with_string_that_needs_encoding() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_NAME=\"My name\"\nVAR_SECRET=\"{{keepass \"group1\" \"test3\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"8/k,9P`Y\\\"\\\"7)*]CNdM~,\""));
    }*/

    #[test]
    fn test_handlebars_keepass_unknown_variable() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_SECRET=\"{{keepass \"not-found-group\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"<Not found keepass entry>\""));
    }
}
