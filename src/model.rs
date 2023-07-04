use crate::builders::Template;
use crate::db_entries::{Fld, ModelDbEntry, Tmpl};
use crate::{Error, Field};

const DEFAULT_LATEX_PRE: &str = r#"
\documentclass[12pt]{article}
\special{papersize=3in,5in}
\usepackage[utf8]{inputenc}
\usepackage{amssymb,amsmath}
\pagestyle{empty}
\setlength{\parindent}{0in}
\begin{document}

"#;
const DEFAULT_LATEX_POST: &str = r"\end{document}";

/// `FrontBack` or `Cloze` to determine the type of a Model.
///
/// When creating a Model, the default is `FrontBack`
#[derive(Clone, PartialEq, Eq)]
pub enum ModelType {
    FrontBack,
    Cloze,
}

/// `Model` to determine the structure of a `Note`
#[derive(Clone)]
pub struct Model {
    pub id: i64,
    name: String,
    fields: Vec<Fld>,
    templates: Vec<Tmpl>,
    css: String,
    model_type: ModelType,
    latex_pre: String,
    latex_post: String,
    sort_field_index: i64,
}

impl Model {
    /// Creates a new model with a unique(!) `ìd`, a `name`, `fields` and  `templates`
    ///
    /// Example:
    ///
    /// ```
    /// use genanki_rs::{Model, Field, Template};
    /// let model = Model::new(
    ///     1607392319,
    ///     "Simple Model",
    ///     vec![Field::new("Question"), Field::new("Answer")],
    ///     vec![Template::new("Card 1")
    ///         .qfmt("{{Question}}")
    ///         .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#)],
    /// );
    /// ```
    pub fn new(id: i64, name: &str, fields: Vec<Field>, templates: Vec<Template>) -> Self {
        Self {
            id,
            name: name.to_string(),
            fields: fields.iter().cloned().map(|f| f.into()).collect(),
            templates: templates.iter().cloned().map(|t| t.into()).collect(),
            css: "".to_string(),
            model_type: ModelType::FrontBack,
            latex_pre: DEFAULT_LATEX_PRE.to_string(),
            latex_post: DEFAULT_LATEX_POST.to_string(),
            sort_field_index: 0,
        }
    }

    /// Creates a new model with a unique(!) `ìd`, a `name`, `fields` and  `templates` and custom parameters:
    /// * `css`: Custom css to be applied to the cards
    /// * `model_type`: `Cloze` or `FrontBack`, default is `FrontBack`
    /// * `latex_pre`: Custom latex declarations at the beginning of a card.
    /// * `latex_post`: Custom latex declarations at the end of a card.
    /// * `sort_field_index`: Custom sort field index
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_options(
        id: i64,
        name: &str,
        fields: Vec<Field>,
        templates: Vec<Template>,
        css: Option<&str>,
        model_type: Option<ModelType>,
        latex_pre: Option<&str>,
        latex_post: Option<&str>,
        sort_field_index: Option<i64>,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            fields: fields.iter().cloned().map(|f| f.into()).collect(),
            templates: templates.iter().cloned().map(|t| t.into()).collect(),
            css: css.unwrap_or("").to_string(),
            model_type: model_type.unwrap_or(ModelType::FrontBack),
            latex_pre: latex_pre.unwrap_or(DEFAULT_LATEX_PRE).to_string(),
            latex_post: latex_post.unwrap_or(DEFAULT_LATEX_POST).to_string(),
            sort_field_index: sort_field_index.unwrap_or(0),
        }
    }

    /// Adds an additional field to the model
    pub fn with_field(mut self, field: Field) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Adds an additional template to the model
    pub fn with_template(mut self, template: Template) -> Self {
        self.templates.push(template.into());
        self
    }

    /// Sets the custom CSS for this model
    pub fn css(self, css: impl ToString) -> Self {
        Self {
            css: css.to_string(),
            ..self
        }
    }

    /// Change the type of the model
    pub fn model_type(self, model_type: ModelType) -> Self {
        Self { model_type, ..self }
    }

    /// Sets the model's latex_pre field
    pub fn latex_pre(self, latex_pre: impl ToString) -> Self {
        Self {
            latex_pre: latex_pre.to_string(),
            ..self
        }
    }

    /// Sets the model's latex_post field
    pub fn latex_post(self, latex_post: impl ToString) -> Self {
        Self {
            latex_post: latex_post.to_string(),
            ..self
        }
    }

    /// Sets the index of the field used for sorting with this model
    pub fn sort_field_index(self, sort_field_index: i64) -> Self {
        Self {
            sort_field_index,
            ..self
        }
    }

    pub(super) fn req(&self) -> Result<Vec<(usize, String, Vec<usize>)>, Error> {
        let sentinel = "SeNtInEl".to_string();
        let field_names: Vec<String> = self.fields.iter().map(|field| field.name.clone()).collect();
        let field_values = field_names
            .iter()
            .map(|field| (field.as_str(), format!("{}{}", &field, &sentinel)));
        let mut req = Vec::new();
        for template_ord in 0..self.templates.len() {
            req.push((
                template_ord,
                "all".to_string(),
                (0..field_values.len()).collect(),
            ));
        }
        Ok(req)
    }

    pub(super) fn fields(&self) -> Vec<Fld> {
        self.fields.clone()
    }
    pub(super) fn templates(&self) -> Vec<Tmpl> {
        self.templates.clone()
    }
    pub(super) fn get_model_type(&self) -> ModelType {
        self.model_type.clone()
    }
    pub(super) fn to_model_db_entry(
        &mut self,
        timestamp: f64,
        deck_id: i64,
    ) -> Result<ModelDbEntry, Error> {
        self.templates
            .iter_mut()
            .enumerate()
            .for_each(|(i, template)| {
                template.ord = i as i64;
            });
        self.fields.iter_mut().enumerate().for_each(|(i, field)| {
            field.ord = i as i64;
        });
        let model_type = match self.model_type {
            ModelType::FrontBack => 0,
            ModelType::Cloze => 1,
        };
        Ok(ModelDbEntry {
            vers: vec![],
            name: self.name.clone(),
            tags: vec![],
            did: deck_id,
            usn: -1,
            req: self.req()?,
            flds: self.fields.clone(),
            sortf: self.sort_field_index,
            tmpls: self.templates.clone(),
            model_db_entry_mod: timestamp as i64,
            latex_post: self.latex_post.clone(),
            model_db_entry_type: model_type,
            id: self.id.to_string(),
            css: self.css.clone(),
            latex_pre: self.latex_pre.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Deck, Note};
    use std::collections::HashSet;
    use tempfile::NamedTempFile;

    fn css() -> String {
        r#".card {
 font-family: arial;
 font-size: 20px;
 text-align: center;
 color: black;
 background-color: white;
}

.cloze {
 font-weight: bold;
 color: blue;
}
.nightMode .cloze {
 color: lightblue;
}
"#
        .to_owned()
    }

    fn cloze_model() -> Model {
        Model::new_with_options(
            998877661,
            "Cloze Model",
            vec![Field::new("Text"), Field::new("Extra")],
            vec![Template::new("My Cloze Card")
                .qfmt("{{Text}}")
                .afmt("{{Text}}<br>{{Extra}}")],
            Some(&css()),
            Some(ModelType::Cloze),
            None,
            None,
            None,
        )
    }

    fn multi_field_cloze_model() -> Model {
        Model::new_with_options(
            1047194615,
            "Multi Field Cloze Model",
            vec![Field::new("Text1"), Field::new("Text2")],
            vec![Template::new("Cloze")
                .qfmt("{{cloze:Text1}} and {{cloze:Text2}}")
                .afmt("{{cloze:Text1}} and {{cloze:Text2}}")],
            Some(&css()),
            Some(ModelType::Cloze),
            None,
            None,
            None,
        )
    }

    #[test]
    fn cloze_multi_field() {
        let fields = vec![
            "{{c1::Berlin}} is the capital of {{c2::Germany}}",
            "{{c3::Paris}} is the capital of {{c4::France}}",
        ];
        let note = Note::new(multi_field_cloze_model(), fields).unwrap();
        let mut sorted = note
            .cards()
            .iter()
            .map(|card| card.ord())
            .collect::<Vec<i64>>();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
    }

    #[test]
    fn build_all_fields() {
        // A simple test to make sure we can call all the setters on the builder.
        //
        // It doesn't actually verify any behavior, it's basically just a smoke test.
        Model::new(12345, "test model", vec![Field::new("front")], vec![])
            .with_template(Template::new("template"))
            .css(css())
            .latex_post("")
            .latex_pre("")
            .sort_field_index(1)
            .model_type(ModelType::FrontBack);
    }
}
