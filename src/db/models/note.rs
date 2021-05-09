use super::schema::notes;

#[derive(Queryable, Insertable, Debug, PartialEq, Eq)]
pub struct Note {
    pub id: Option<String>,
    pub parent_id: String,
    pub title: String,
    pub body: String,
}
