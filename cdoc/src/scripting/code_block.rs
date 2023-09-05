use crate::ast::CodeAttributes;
use crate::notebook::{CellOutput, OutputValue, StreamType};
use anyhow::anyhow;
use rhai::{CustomType, Dynamic, TypeBuilder};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct ScriptCodeBlock {
    source: String,
    reference: Option<String>,
    attr: CodeAttributes,
    tags: Option<Vec<String>>,
    outputs: Vec<Dynamic>,
    display_cell: bool,
}

impl ScriptCodeBlock {
    pub fn new(
        source: &str,
        reference: &Option<String>,
        attr: &CodeAttributes,
        tags: &Option<Vec<String>>,
        outputs: &[CellOutput],
        display_cell: bool,
    ) -> Self {
        ScriptCodeBlock {
            source: String::from(source),
            reference: reference.clone(),
            attr: attr.clone(),
            tags: tags.clone(),
            outputs: outputs.iter().map(|c| c.clone().into()).collect(),
            display_cell,
        }
    }

    pub fn apply_changes(
        self,
        source: &mut String,
        reference: &mut Option<String>,
        attr: &mut CodeAttributes,
        tags: &mut Option<Vec<String>>,
        outputs: &mut Vec<CellOutput>,
        display_cell: &mut bool,
    ) -> anyhow::Result<()> {
        *source = self.source;
        *reference = self.reference;
        *attr = self.attr;
        *tags = self.tags;
        *outputs = self
            .outputs
            .into_iter()
            .map(|c| c.try_into())
            .collect::<anyhow::Result<Vec<CellOutput>>>()?;
        *display_cell = self.display_cell;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CellOutputStream {
    name: StreamType,
    text: String,
}

#[derive(Clone)]
pub struct CellOutputData {
    execution_count: Option<i64>,
    data: Vec<OutputValue>,
    metadata: HashMap<String, Value>,
}

#[derive(Clone)]
pub struct CellOutputError {
    ename: String,
    evalue: String,
    traceback: Vec<String>,
}

impl From<CellOutput> for Dynamic {
    fn from(value: CellOutput) -> Self {
        match value {
            CellOutput::Stream { name, text } => Dynamic::from(CellOutputStream { name, text }),
            CellOutput::Data {
                execution_count,
                data,
                metadata,
            } => Dynamic::from(CellOutputData {
                execution_count,
                data,
                metadata,
            }),
            CellOutput::Error {
                ename,
                evalue,
                traceback,
            } => Dynamic::from(CellOutputError {
                ename,
                evalue,
                traceback,
            }),
        }
    }
}

impl From<CellOutputStream> for CellOutput {
    fn from(value: CellOutputStream) -> Self {
        CellOutput::Stream {
            name: value.name,
            text: value.text,
        }
    }
}

impl From<CellOutputData> for CellOutput {
    fn from(value: CellOutputData) -> Self {
        CellOutput::Data {
            execution_count: value.execution_count,
            data: value.data,
            metadata: value.metadata,
        }
    }
}

impl From<CellOutputError> for CellOutput {
    fn from(value: CellOutputError) -> Self {
        CellOutput::Error {
            ename: value.ename,
            evalue: value.evalue,
            traceback: value.traceback,
        }
    }
}

impl TryFrom<Dynamic> for CellOutput {
    type Error = anyhow::Error;

    fn try_from(value: Dynamic) -> Result<Self, <CellOutput as TryFrom<Dynamic>>::Error> {
        match value.type_name() {
            "CellOutputStream" => Ok(value.cast::<CellOutputStream>().into()),
            "CellOutputData" => Ok(value.cast::<CellOutputData>().into()),
            "CellOutputError" => Ok(value.cast::<CellOutputError>().into()),
            _ => Err(anyhow!(format!(
                "invalid cell output type {}",
                value.type_name()
            ))),
        }
    }
}

impl CustomType for ScriptCodeBlock {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("CodeBlock")
            .with_get_set(
                "source",
                |s: &mut Self| s.source.clone(),
                |s: &mut Self, v: String| s.source = v,
            )
            .with_get_set(
                "reference",
                |s: &mut Self| s.reference.clone(),
                |s: &mut Self, v: Option<String>| s.reference = v,
            )
            .with_get_set(
                "attr",
                |s: &mut Self| s.attr.clone(),
                |s: &mut Self, v: CodeAttributes| s.attr = v,
            )
            .with_get_set(
                "tags",
                |s: &mut Self| s.tags.clone(),
                |s: &mut Self, v: Option<Vec<String>>| s.tags = v,
            )
            .with_get_set(
                "outputs",
                |s: &mut Self| s.outputs.clone(),
                |s: &mut Self, v: Vec<Dynamic>| s.outputs = v,
            )
            .with_get_set(
                "display_cell",
                |s: &mut Self| s.display_cell,
                |s: &mut Self, v: bool| s.display_cell = v,
            );
    }
}

impl CustomType for CellOutputStream {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Stream")
            .with_get_set(
                "name",
                |s: &mut Self| s.name.clone(),
                |s: &mut Self, v: StreamType| s.name = v,
            )
            .with_get_set(
                "text",
                |s: &mut Self| s.text.clone(),
                |s: &mut Self, v: String| s.text = v,
            );
    }
}

impl CustomType for CellOutputData {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Data")
            .with_get_set(
                "execution_count",
                |s: &mut Self| s.execution_count,
                |s: &mut Self, v: Option<i64>| s.execution_count = v,
            )
            .with_get_set(
                "data",
                |s: &mut Self| s.data.clone(),
                |s: &mut Self, v: Vec<OutputValue>| s.data = v,
            )
            .with_get_set(
                "metadata",
                |s: &mut Self| s.metadata.clone(),
                |s: &mut Self, v: HashMap<String, Value>| s.metadata = v,
            );
    }
}

impl CustomType for CellOutputError {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Error")
            .with_get_set(
                "ename",
                |s: &mut Self| s.ename.clone(),
                |s: &mut Self, v: String| s.ename = v,
            )
            .with_get_set(
                "evalue",
                |s: &mut Self| s.evalue.clone(),
                |s: &mut Self, v: String| s.evalue = v,
            )
            .with_get_set(
                "traceback",
                |s: &mut Self| s.traceback.clone(),
                |s: &mut Self, v: Vec<String>| s.traceback = v,
            );
    }
}
