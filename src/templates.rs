use tera::Tera;

fn load_tera() -> Result<Tera, tera::Error> {
    #[derive(rust_embed::RustEmbed)]
    #[folder = "src/templates/"]
    struct Templates;

    let mut tera = Tera::default();

    let mut templates = vec![];

    for template in Templates::iter() {
        templates.push((
            template.clone(),
            String::from_utf8(Templates::get(&template).unwrap().data.into_owned()).unwrap(),
        ));
    }

    tera.add_raw_templates(templates)?;

    Ok(tera)
}

lazy_static::lazy_static! {
    static ref TEMPLATES: Tera = load_tera().unwrap();
}

#[cfg(debug_assertions)]
pub fn templates() -> Tera {
    load_tera().unwrap()
}

#[cfg(not(debug_assertions))]
pub fn templates() -> &'static Tera {
    &TEMPLATES
}
