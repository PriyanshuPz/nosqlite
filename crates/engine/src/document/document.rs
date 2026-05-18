pub type Document = bson::Document;
pub type DocId = bson::oid::ObjectId;

// +-----------------------+---------------------+-------------------+------------------------+
// | num_docs (u32) | next_page_ptr (u64) | Name (\0) | Addr  | Name (\0) | Addr ...   |
// | 4 Bytes        | 8 Bytes             | Var-len + 8 Bytes | Var-len + 8 Bytes      |
// +-----------------------+---------------------+-------------------+------------------------+
// |<---------- Header (12 Bytes) ------->|<------------------ Data Payload ---------->|
