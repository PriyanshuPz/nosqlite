# nosqlite

trying to build a small embedded nosql database from scratch in rust.

main idea is simple:
sqlite like single file database but for document/nosql style storage.

this project is mostly for learning database internals and low level systems stuff.

things i want to learn while building this:
- pager
- page layouts
- b+ tree
- binary storage
- write ahead logging
- indexes
- query engine
- recovery

not trying to make next mongodb or some distributed cloud thing.


## AI disclosure

for this project i am not using any in editor AI tools or autocomplete AI.
no copilot.
no AI code generation inside editor.
using zed editor normally and writing code manually.

AI tools like ChatGPT or Google Gemini can still be used sometimes for theory, explanations, debugging help, searching things

main reason is i want to actually learn how databases work internally instead of vibe coding random storage engine.


## resources

will keep adding useful stuff here while building.

- [SQLite Database System Design and Implementation Book by Sibsankar Haldar](https://books.google.co.in/books?id=OEJ1CQAAQBAJ)
- https://ratatui.rs/recipes/apps
- https://bsonspec.org/spec.html
- https://doc.rust-lang.org/book/ch16-03-shared-state.html
