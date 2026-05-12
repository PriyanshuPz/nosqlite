use anyhow::Result;
use bson::Document;

pub fn encode_document(document: &Document) -> Result<Vec<u8>> {
    Ok(document.to_vec()?)
}

pub fn decode_document(bytes: &[u8]) -> Result<Document> {
    Ok(Document::from_reader(bytes)?)
}

pub fn parse_document(input: &str) -> Result<Document> {
    let value: serde_json::Value = serde_json::from_str(input)?;
    let document = bson::ser::serialize_to_document(&value)?;
    Ok(document)
}
