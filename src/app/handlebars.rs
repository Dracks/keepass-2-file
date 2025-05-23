use handlebars::{
    handlebars_helper, no_escape, Context, Handlebars, Helper, HelperDef, JsonRender, JsonValue,
    PathAndJson, RenderContext, RenderError, ScopedJson,
};
use keepass::{
    db::{Entry, NodeRef},
    Database,
};

use super::errors_and_warnings::{ErrorCode, ErrorRecord};
use super::tools::convert_vecs;

pub type LibHandlebars<'reg> = Handlebars<'reg>;

pub struct KeepassHelper<'a> {
    db: Database,
    errors: &'a dyn ErrorRecord,
}

const NOT_FOUND_ERROR: &str = "<Not found keepass entry>";
const NO_PASSWORD_ERROR: &str = "<No password found in entry>";
const NO_USERNAME_ERROR: &str = "<No username found in entry>";
const NO_URL_ERROR: &str = "<No URL found in entry>";
const ATTRIBUTE_NOT_FOUND_ERROR: &str = "<Attribute ({field-name}) not found in entry>";

#[derive(Debug)]
enum FieldSelect {
    Password,
    Username,
    Url,
    AdditionalAttributes { field_name: String },
}

fn extract_field_value(field_path: Option<&PathAndJson>) -> Option<String> {
    if let Some(field_path) = field_path {
        if let Some(field_value) = field_path.relative_path() {
            return Some(field_value.to_string());
        }
        let json_value = field_path.value();
        let value = json_value.render();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn extract_field_type(field_path: Option<&PathAndJson>) -> FieldSelect {
    if let Some(field) = extract_field_value(field_path) {
        return match field.to_lowercase().as_str() {
            "username" => FieldSelect::Username,
            "password" => FieldSelect::Password,
            "url" => FieldSelect::Url,
            _ => FieldSelect::AdditionalAttributes { field_name: field },
        };
    }
    FieldSelect::Password
}

fn get_additional_fields(entry: &Entry, field_name: String) -> Option<String> {
    let contents = entry.get(field_name.as_str());
    if let Some(contents) = contents {
        return Some(contents.to_string());
    }
    None
}

impl ErrorCode {
    fn to_hb_entry(&self) -> String {
        match self {
            ErrorCode::MissingEntry(_) => NOT_FOUND_ERROR.into(),
            ErrorCode::MissingField(_, field_name) => {
                ATTRIBUTE_NOT_FOUND_ERROR.replace("{field-name}", field_name.as_str())
            }
            ErrorCode::NoPassword(_) => NO_PASSWORD_ERROR.into(),
            ErrorCode::NoUsername(_) => NO_USERNAME_ERROR.into(),
            ErrorCode::NoUrl(_) => NO_URL_ERROR.into(),
        }
    }
}

impl KeepassHelper<'_> {
    fn extract_entry(&self, path: Vec<&str>, field: FieldSelect) -> Result<String, ErrorCode> {
        let path_str: Vec<String> = convert_vecs(path.clone());
        if let Some(node) = self.db.root.get(&path) {
            match node {
                NodeRef::Group(_) => Err(ErrorCode::MissingEntry(path_str)),
                NodeRef::Entry(entry) => match field {
                    FieldSelect::Password => match entry.get_password() {
                        Some(pwd) => Ok(pwd.into()),
                        None => Err(ErrorCode::NoPassword(path_str)),
                    },
                    FieldSelect::Username => match entry.get_username() {
                        Some(username) => Ok(username.into()),
                        None => Err(ErrorCode::NoUsername(path_str)),
                    },
                    FieldSelect::Url => match entry.get_url() {
                        Some(url) => Ok(url.into()),
                        None => Err(ErrorCode::NoUrl(path_str)),
                    },
                    FieldSelect::AdditionalAttributes { field_name } => {
                        let result = get_additional_fields(entry, field_name.clone());
                        match result {
                            Some(d2) => Ok(d2),
                            None => Err(ErrorCode::MissingField(path_str, field_name.clone())),
                        }
                    }
                },
            }
        } else {
            Err(ErrorCode::MissingEntry(path_str))
        }
    }
}

impl HelperDef for KeepassHelper<'_> {
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
        let field = extract_field_type(h.hash_get("field"));
        match self.extract_entry(path, field) {
            Ok(content) => Ok(ScopedJson::from(JsonValue::from(content))),
            Err(error_code) => {
                self.errors.register_error(error_code.clone());
                Ok(ScopedJson::Derived(JsonValue::from(
                    error_code.to_hb_entry(),
                )))
            }
        }
    }
}

handlebars_helper!(stringify: |x: String| {
    format!("\"{}\"", x.replace('\"', "\\\"").replace('$', "\\$"))
});

pub fn build_handlebars(db: Database, errors: &dyn ErrorRecord) -> Handlebars<'_> {
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("keepass", Box::new(KeepassHelper { db, errors }));
    handlebars.register_helper("stringify", Box::new(stringify));

    handlebars
}

#[cfg(test)]
mod tests {
    use crate::app::errors_and_warnings::HelperErrors;

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
        let errors_and_warnings = HelperErrors::new();
        let handlebars = build_handlebars(get_db(), &errors_and_warnings);

        let template =
            "VAR_NAME=\"My name\"\nVAR_SECRET=\"{{keepass \"group1\" \"Some weird name\"}}\"";

        let result = handlebars.render_template(template, &());
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"S$c&<J)=EVm#xo{t]<ml\""));
    }

    #[test]
    fn test_handlebars_keepass_with_string_that_needs_encoding() {
        let errors_and_warnings = HelperErrors::new();
        let handlebars = build_handlebars(get_db(), &errors_and_warnings);

        let template = "VAR_NAME=\"My name\"\nVAR_SECRET={{stringify (keepass \"test1\")}}\"";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"8/k,9P`Y\\\"\\\"7)*]CNdM~,\""));
    }

    #[test]
    fn test_handlebars_keepass_unknown_variable() {
        let errors_and_warnings = HelperErrors::new();
        {
            let handlebars = build_handlebars(get_db(), &errors_and_warnings);

            let template = "VAR_SECRET=\"{{keepass \"not-found-group\"}}\"";

            let result = handlebars.render_template(template, &());
            assert!(result.is_ok());

            let rendered = result.unwrap();
            assert!(rendered.contains("VAR_SECRET=\"<Not found keepass entry>\""));
        }
        let errors = errors_and_warnings.get_errors();
        println!("{:?}", errors);
        assert_eq!(errors.len(), 1)
    }

    #[test]
    fn test_handlebars_keepass_retrieve_username() {
        let errors_and_warnings = HelperErrors::new();
        let handlebars = build_handlebars(get_db(), &errors_and_warnings);

        let template = "VAR_SECRET=\"{{keepass field=username \"test1\"}}\"";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("VAR_SECRET=\"user1\""));
    }

    #[test]
    fn test_handlebars_keepass_retrieve_other_fields() {
        let errors_and_warnings = HelperErrors::new();
        let handlebars = build_handlebars(get_db(), &errors_and_warnings);

        let template = "URL=\"{{keepass field=url \"complex\"}}\"
            ADDITIONAL=\"{{keepass field=attribute-1 \"complex\"}}\"
            ATTRIBUTE_WITH_SPACES=\"{{keepass field=\"attribute with spaces\" \"complex\"}}\"
        ";

        let result = handlebars.render_template(template, &());
        println!("{:?}", result);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("URL=\"http://complex.url\""));
        assert!(rendered.contains("ADDITIONAL=\"protected-attribute\""));
        assert!(rendered.contains("ATTRIBUTE_WITH_SPACES=\"some extra attr\""));
    }

    #[test]
    fn test_handlebars_keepass_missing_fields() {
        let errors_and_warnings = HelperErrors::new();
        {
            let handlebars = build_handlebars(get_db(), &errors_and_warnings);

            let template = "PASSWORD=\"{{keepass \"missing\"}}\"
                USERNAME=\"{{keepass field=username \"missing\"}}\"
                URL=\"{{keepass field=url \"missing\"}}\"
                ATTRIBUTE=\"{{keepass field=missing \"missing\"}}\"
            ";

            let result = handlebars.render_template(template, &());
            assert!(result.is_ok());

            let rendered = result.unwrap();
            assert!(rendered.contains("PASSWORD=\"<No password found in entry>\""));
            assert!(rendered.contains("USERNAME=\"<No username found in entry>\""));
            assert!(rendered.contains("URL=\"<No URL found in entry>\""));
            assert!(rendered.contains("ATTRIBUTE=\"<Attribute (missing) not found in entry>\""));
        }
        let errors = errors_and_warnings.get_errors();
        println!("{:?}", errors);
        assert_eq!(errors.len(), 4)
    }
}
