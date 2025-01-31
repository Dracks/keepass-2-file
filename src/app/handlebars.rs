use handlebars::{
    handlebars_helper, no_escape, Context, Handlebars, Helper, HelperDef, JsonValue, PathAndJson,
    RenderContext, RenderError, ScopedJson,
};
use keepass::{
    db::{Entry, NodeRef},
    Database,
};
use std::error::Error;

pub struct KeepassHelper {
    db: Database,
}

const NOT_FOUND_ERROR: &str = "<Not found keepass entry>";
const NO_PASSWORD_ERROR: &str = "<No password found in entry>";
const NO_USERNAME_ERROR: &str = "<No username found in entry>";
const NO_URL_ERROR: &str = "<No URL found in entry>";
const INVALID_FIELD_TYPE: &str = "<Invalid field type>";
const ATTRIBUTE_NOT_FOUND_ERROR: &str = "<Attribute not found in entry>";

enum FieldSelect {
    Password,
    Username,
    Url,
    AdditionalAttributes,
}

fn get_field(field_path: Option<&PathAndJson>) -> Result<FieldSelect, Box<dyn Error>> {
    if let Some(field_path) = field_path {
        if let Some(field) = field_path.relative_path() {
            println!("{}", field);
            match field.to_lowercase().as_str() {
                "username" => Ok(FieldSelect::Username),
                "password" => Ok(FieldSelect::Password),
                "url" => Ok(FieldSelect::Url),
                "additional-attributes" => Ok(FieldSelect::AdditionalAttributes),
                _ => Err(format!("Invalid field type {}", field).into()),
            }
        } else {
            Ok(FieldSelect::Password)
        }
    } else {
        Ok(FieldSelect::Password)
    }
}

fn get_additional_fields<'a>(
    entry: &'a Entry,
    attribute_path: Option<&'a PathAndJson<'a>>,
) -> &'a str {
    if let Some(attribute_path) = attribute_path {
        if let Some(attribute) = attribute_path.relative_path() {
            let contents = entry.get(attribute);
            if let Some(contents) = contents {
                return contents;
            }
        }
    }
    ATTRIBUTE_NOT_FOUND_ERROR
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
            .map(|x| x.render())
            .collect::<Vec<String>>();
        let path = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        let field = get_field(h.hash_get("field"));
        println!("{:?}", h);
        println!("{:?}", args);
        if let Some(node) = self.db.root.get(&path) {
            match node {
                NodeRef::Group(_) => Ok(ScopedJson::Derived(JsonValue::from(NOT_FOUND_ERROR))),
                NodeRef::Entry(entry) => {
                    if let Ok(field) = field {
                        let content: Option<&str> = match field {
                            FieldSelect::Password => entry.get_password(),
                            FieldSelect::Username => entry.get_username(),
                            FieldSelect::Url => entry.get_url(),
                            FieldSelect::AdditionalAttributes => {
                                Some(get_additional_fields(entry, h.hash_get("attribute")))
                            }
                        };
                        if let Some(content) = content {
                            Ok(ScopedJson::from(JsonValue::from(content)))
                        } else {
                            Ok(ScopedJson::from(JsonValue::from(match field {
                                FieldSelect::Password => NO_PASSWORD_ERROR,
                                FieldSelect::Username => NO_USERNAME_ERROR,
                                FieldSelect::Url => NO_URL_ERROR,
                                FieldSelect::AdditionalAttributes => ATTRIBUTE_NOT_FOUND_ERROR,
                            })))
                        }
                    } else {
                        Ok(ScopedJson::from(JsonValue::from(INVALID_FIELD_TYPE)))
                    }
                }
            }
        } else {
            Ok(ScopedJson::Derived(JsonValue::from(NOT_FOUND_ERROR)))
        }
    }
}

handlebars_helper!(stringify: |x: String| {
    format!("\"{}\"", x.replace('\"', "\\\"").replace('$', "\\$"))
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

    #[test]
    fn test_handlebars_keepass_retrieve_username() {
        let handlebars = build_handlebars(get_db());

        let template = "VAR_SECRET=\"{{keepass field=username \"test1\"}}\"";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"user1\""));
    }

    #[test]
    fn test_handlebars_keepass_retrieve_other_fields() {
        let handlebars = build_handlebars(get_db());

        let template = "URL=\"{{keepass field=url \"complex\"}}\"
            ADDITIONAL=\"{{keepass field=additional-attributes attribute=attribute-1 \"complex\"}}\"
        ";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("URL=\"http://complex.url\""));
        assert!(rendered.contains("ADDITIONAL=\"protected-attribute\""));
    }

    #[test]
    fn test_handlebars_keepass_missing_fields() {
        let handlebars = build_handlebars(get_db());

        let template = "PASSWORD=\"{{keepass \"missing\"}}\"
            USERNAME=\"{{keepass field=username \"missing\"}}\"
            URL=\"{{keepass field=url \"missing\"}}\"
            ATTRIBUTE=\"{{keepass field=additional-attributes attribute=missing \"missing\"}}\"
        ";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("PASSWORD=\"<No password found in entry>\""));
        assert!(rendered.contains("USERNAME=\"<No username found in entry>\""));
        assert!(rendered.contains("URL=\"<No URL found in entry>\""));
        assert!(rendered.contains("ATTRIBUTE=\"<Attribute not found in entry>\""));
    }
}
