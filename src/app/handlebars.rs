use handlebars::{
    no_escape, Context, Handlebars, Helper, HelperDef, JsonValue, RenderContext, RenderError,
    ScopedJson, handlebars_helper
};
use keepass::{db::NodeRef, Database};

pub struct KeepassHelper {
    db: Database,
}

const NOT_FOUND_ERROR: &str = "<Not found keepass entry>";
const NO_PASSWORD_ERROR: &str = "<No password found in entry>";

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
            .map(|x| x.render())
            .collect::<Vec<String>>();
        let path = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        println!("{:?}", args);
        if let Some(node) = self.db.root.get(&path) {
            match node {
                NodeRef::Group(_) => Ok(ScopedJson::Derived(JsonValue::from(NOT_FOUND_ERROR))),
                NodeRef::Entry(entry) => {
                    if let Some(password) = entry.get_password() {
                        Ok(ScopedJson::from(JsonValue::from(password)))
                    } else {
                        Ok(ScopedJson::from(JsonValue::from(NO_PASSWORD_ERROR)))
                    }
                }
            }
        } else {
            Ok(ScopedJson::Derived(JsonValue::from(NOT_FOUND_ERROR)))
        }
    }
}

handlebars_helper!(stringify: |x: String| {
    format!("\"{}\"", x.replace("\"", "\\\"").replace("$", "\\$"))
});

pub fn build_handlebars<'reg>(db: Database) -> Handlebars<'reg> {
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("keepass", Box::new(KeepassHelper { db }));
    handlebars.register_helper("stringify", Box::new(stringify));

    handlebars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_db() -> Database {
        let mut file = File::open("test_resources/test_db.kdbx").expect("Test DB cannot be open");

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

    #[test]
    fn test_handlebars_keepass_with_string_that_needs_encoding() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_NAME=\"My name\"\nVAR_SECRET={{stringify (keepass \"test1\")}}\"";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"8/k,9P`Y\\\"\\\"7)*]CNdM~,\""));
    }

    #[test]
    fn test_handlebars_keepass_unknown_variable() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_SECRET=\"{{keepass \"not-found-group\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"<Not found keepass entry>\""));
    }

    /*#[test]
    fn test_handlebars_keepass_retrieve_username() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_SECRET=\"{{keepass {field: \"username\"} \"not-found-group\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"<Not found keepass entry>\""));
    }*/
}
