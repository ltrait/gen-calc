use ltrait::{async_trait::async_trait, generator::Generator};
use numbat::{Context, module_importer::BuiltinModuleImporter, resolver::CodeSource};
use std::path::PathBuf;

pub struct CalcConfig {
    /// calc accepts inputs of the following form as input to calc. Other inputs are ignored.
    /// ={{prefix}} {{formula}}
    ///
    /// Some problems (e.g. finding solutions to nth order equations) cannot be solved
    /// with numbat or converted in units with kalker, so they can be branched with prefixes.
    ///
    /// It is not possible to set both to `None`.
    ///
    /// (kalker_prefix, numbat_prefix)
    ///
    /// If `(Some('k'), None)` and you type `=k `, the formula that is continued is evaluated with
    /// kalker.
    prefix: (Option<char>, Option<char>),
    kalker_init_path: Option<PathBuf>,
    numbat_init_path: Option<PathBuf>,
}

impl CalcConfig {
    pub fn new(
        prefix: (Option<char>, Option<char>),
        kalker_init_path: Option<PathBuf>,
        numbat_init_path: Option<PathBuf>,
    ) -> Self {
        if prefix.0.is_none() && prefix.1.is_none() {
            panic!("It is not possible to set both to None.")
        }
        Self {
            prefix,
            kalker_init_path,
            numbat_init_path,
        }
    }
}

pub struct Calc {
    config: CalcConfig,
}

impl Calc {
    pub fn new(config: CalcConfig) -> Self {
        Self { config }
    }

    fn numbat(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut ctx = Context::new(BuiltinModuleImporter::default());

        let _ = ctx.interpret("use prelude", CodeSource::Internal)?;

        if let Some(ref config_path) = self.config.numbat_init_path {
            let config_content = std::fs::read_to_string(config_path)?;

            let _ = ctx.interpret(&config_content, CodeSource::File(config_path.to_path_buf()))?;
        }

        match ctx.interpret(input, CodeSource::Text) {
            Ok((statements, results)) => {
                let result_markup =
                    results.to_markup(statements.last(), ctx.dimension_registry(), true, true);
                let output = numbat::markup::plain_text_format(&result_markup, false);
                Ok(output.into())
            }
            Err(e) => Err(format!("{e}").into()),
        }
    }

    fn kalker(&self, input: &str) -> Option<String> {
        todo!()
    }
}

#[async_trait]
impl Generator for Calc {
    type Item = String;

    async fn generate(&self, input: &str) -> Vec<Self::Item> {
        // kalker
        if input.starts_with(&format!(
            "={} ",
            self.config
                .prefix
                .0
                .map(|s| format!("{s}"))
                .unwrap_or_default()
        )) {
            self.numbat(input).map(|res| vec![res]).unwrap_or_default()
        } else if input.starts_with(&format!(
            "={} ",
            self.config
                .prefix
                .1
                .map(|s| format!("{s}"))
                .unwrap_or_default()
        )) {
            self.kalker(input).map(|res| vec![res]).unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}
